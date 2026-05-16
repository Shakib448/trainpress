use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use trainpress::{
    App, AppError, Json, Request,
    middleware::{Logger, RequestId},
};

// Test models
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

// Test state
#[derive(Clone)]
struct TestState {
    users: Arc<Mutex<Vec<User>>>,
    counter: Arc<Mutex<u64>>,
}

impl TestState {
    fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

// Test handlers
async fn get_root(_req: Request) -> &'static str {
    "Welcome to TrainPress!"
}

async fn health_check(_req: Request) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0"
    }))
}

async fn list_users(_req: Request) -> Json<Vec<User>> {
    Json(Vec::new())
}

async fn get_user(_req: Request) -> Result<Json<User>, AppError> {
    Err(AppError::NotFound)
}

async fn create_user(_req: Request) -> Result<Json<User>, AppError> {
    Ok(Json(User {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    }))
}

async fn update_user(_req: Request) -> Result<Json<User>, AppError> {
    Err(AppError::NotFound)
}

async fn delete_user(_req: Request) -> Result<&'static str, AppError> {
    Err(AppError::NotFound)
}

async fn echo_json(_req: Request) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"echo": "test"})))
}

// Helper function to create test app
fn create_test_app(state: TestState) -> App<TestState> {
    App::new(state)
        .middleware(RequestId)
        .middleware(Logger)
        .get("/", get_root)
        .get("/health", health_check)
        .get("/users", list_users)
        .get("/users/{id}", get_user)
        .post("/users", create_user)
        .put("/users/{id}", update_user)
        .delete("/users/{id}", delete_user)
        .post("/echo", echo_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation_with_state() {
        let state = TestState::new();
        let app = create_test_app(state);
        assert!(app.router.find(&hyper::Method::GET, "/").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/health").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/users").is_some());
    }

    #[test]
    fn test_app_stateless_creation() {
        let app = App::new_stateless()
            .get("/", get_root)
            .get("/health", health_check);

        assert!(app.router.find(&hyper::Method::GET, "/").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/health").is_some());
    }

    #[test]
    fn test_all_routes_registered() {
        let state = TestState::new();
        let app = create_test_app(state);

        // Check all routes are registered
        assert!(app.router.find(&hyper::Method::GET, "/").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/health").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/users").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/users/123").is_some());
        assert!(app.router.find(&hyper::Method::POST, "/users").is_some());
        assert!(app.router.find(&hyper::Method::PUT, "/users/123").is_some());
        assert!(
            app.router
                .find(&hyper::Method::DELETE, "/users/123")
                .is_some()
        );
        assert!(app.router.find(&hyper::Method::POST, "/echo").is_some());
    }

    #[test]
    fn test_middleware_registration() {
        let state = TestState::new();
        let app = create_test_app(state);

        // We added 2 middlewares: RequestId and Logger
        assert_eq!(app.middlewares.len(), 2);
    }

    #[test]
    fn test_app_not_found_routes() {
        let state = TestState::new();
        let app = create_test_app(state);

        // Routes that don't exist
        assert!(
            app.router
                .find(&hyper::Method::GET, "/nonexistent")
                .is_none()
        );
        assert!(
            app.router
                .find(&hyper::Method::PATCH, "/users/123")
                .is_none()
        );
        assert!(app.router.find(&hyper::Method::OPTIONS, "/").is_none());
    }

    #[test]
    fn test_state_initialization() {
        let state = TestState::new();
        let counter_val = state.counter.blocking_lock();
        assert_eq!(*counter_val, 0);
    }

    #[test]
    fn test_multiple_apps_independent_state() {
        let state1 = TestState::new();
        let state2 = TestState::new();

        let _app1 = create_test_app(state1.clone());
        let _app2 = create_test_app(state2.clone());

        // States should be independent
        let mut counter1 = state1.counter.blocking_lock();
        let mut counter2 = state2.counter.blocking_lock();

        *counter1 = 10;
        *counter2 = 20;

        assert_eq!(*counter1, 10);
        assert_eq!(*counter2, 20);
    }

    #[test]
    fn test_app_builder_pattern_chaining() {
        let state = TestState::new();

        // Test builder pattern chaining
        let app = App::new(state)
            .get("/test1", get_root)
            .post("/test2", get_root)
            .put("/test3", get_root)
            .patch("/test4", get_root)
            .delete("/test5", get_root);

        assert!(app.router.find(&hyper::Method::GET, "/test1").is_some());
        assert!(app.router.find(&hyper::Method::POST, "/test2").is_some());
        assert!(app.router.find(&hyper::Method::PUT, "/test3").is_some());
        assert!(app.router.find(&hyper::Method::PATCH, "/test4").is_some());
        assert!(app.router.find(&hyper::Method::DELETE, "/test5").is_some());
    }

    #[test]
    fn test_route_with_path_parameters() {
        let state = TestState::new();
        let app = create_test_app(state);

        // Test path parameter routes
        let matched = app.router.find(&hyper::Method::GET, "/users/123");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().params.get("id").unwrap(), "123");

        let matched = app.router.find(&hyper::Method::DELETE, "/users/456");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().params.get("id").unwrap(), "456");
    }

    #[test]
    fn test_multiple_http_methods_same_path() {
        let state = TestState::new();
        let app = create_test_app(state);

        // Same path "/users" with different methods
        assert!(app.router.find(&hyper::Method::GET, "/users").is_some());
        assert!(app.router.find(&hyper::Method::POST, "/users").is_some());

        // Same path "/users/123" with different methods
        assert!(app.router.find(&hyper::Method::GET, "/users/123").is_some());
        assert!(app.router.find(&hyper::Method::PUT, "/users/123").is_some());
        assert!(
            app.router
                .find(&hyper::Method::DELETE, "/users/123")
                .is_some()
        );
    }

    #[test]
    fn test_app_with_middleware_chain() {
        struct CustomMiddleware;
        impl trainpress::middleware::Middleware for CustomMiddleware {
            fn call(
                &self,
                req: Request,
                next: trainpress::middleware::Next,
            ) -> trainpress::BoxFuture<trainpress::Response> {
                Box::pin(async move { next.run(req).await })
            }
        }

        let state = TestState::new();
        let app = App::new(state)
            .middleware(RequestId)
            .middleware(Logger)
            .middleware(CustomMiddleware)
            .get("/test", get_root);

        assert_eq!(app.middlewares.len(), 3);
        assert!(app.router.find(&hyper::Method::GET, "/test").is_some());
    }

    #[test]
    fn test_json_response_type() {
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let json_response = Json(user.clone());
        // Json wrapper compiles and can be created
    }

    #[test]
    fn test_app_error_types() {
        let _not_found = AppError::NotFound;
        let _bad_request = AppError::BadRequest("test error".to_string());
        let _unauthorized = AppError::Unauthorized;
        let _internal = AppError::Internal("internal error".to_string());

        // All error types compile
    }

    #[test]
    fn test_nested_routes() {
        let state = TestState::new();

        // Test creating an app with nested-looking routes
        let app = App::new(state)
            .get("/api/v1/users", list_users)
            .get("/api/v1/users/{id}", get_user)
            .post("/api/v1/users", create_user);

        assert!(
            app.router
                .find(&hyper::Method::GET, "/api/v1/users")
                .is_some()
        );
        assert!(
            app.router
                .find(&hyper::Method::GET, "/api/v1/users/123")
                .is_some()
        );
        assert!(
            app.router
                .find(&hyper::Method::POST, "/api/v1/users")
                .is_some()
        );
    }

    #[test]
    fn test_app_with_no_middleware() {
        let state = TestState::new();
        let app = App::new(state)
            .get("/", get_root)
            .get("/health", health_check);

        assert_eq!(app.middlewares.len(), 0);
        assert!(app.router.find(&hyper::Method::GET, "/").is_some());
    }

    #[test]
    fn test_result_handlers() {
        // Test that handlers can return Result types
        async fn success_handler(_req: Request) -> Result<Json<User>, AppError> {
            Ok(Json(User {
                id: 1,
                name: "Test".to_string(),
                email: "test@example.com".to_string(),
            }))
        }

        async fn error_handler(_req: Request) -> Result<Json<User>, AppError> {
            Err(AppError::NotFound)
        }

        let state = TestState::new();
        let app = App::new(state)
            .get("/success", success_handler)
            .get("/error", error_handler);

        assert!(app.router.find(&hyper::Method::GET, "/success").is_some());
        assert!(app.router.find(&hyper::Method::GET, "/error").is_some());
    }

    #[test]
    fn test_wildcard_routes() {
        let state = TestState::new();

        async fn files_handler(_req: Request) -> &'static str {
            "files"
        }

        let app = App::new(state).get("/files/{*path}", files_handler);

        let matched = app.router.find(&hyper::Method::GET, "/files/a/b/c.txt");
        assert!(matched.is_some());
        let params = matched.unwrap().params;
        assert_eq!(params.get("path").unwrap(), "a/b/c.txt");
    }
}
