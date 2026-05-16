/// A complete CRUD API example demonstrating TrainPress features:
/// - Shared state management
/// - Path parameters
/// - Query string parsing
/// - JSON request/response bodies
/// - Error handling
/// - Middleware (Logger, RequestId)
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use trainpress::{
    App, AppError, Json, Request,
    extract::{RequestExt, json_body},
    middleware::{Logger, RequestId},
};

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<User>>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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

#[derive(Deserialize, Default)]
struct ListQuery {
    limit: Option<usize>,
}

async fn root(_req: Request) -> &'static str {
    "🦀 Welcome to TrainPress User CRUD API!\n\n\
     Available endpoints:\n\
     - GET    /health       - Health check\n\
     - GET    /users        - List all users (optional ?limit=N)\n\
     - GET    /users/{id}   - Get user by ID\n\
     - POST   /users        - Create new user (JSON body)\n\
     - DELETE /users/{id}   - Delete user by ID\n"
}

async fn health(_req: Request) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "user-crud-api",
        "framework": "TrainPress"
    }))
}

async fn list_users(req: Request) -> Result<Json<Vec<User>>, AppError> {
    // Extract query parameters
    let query: ListQuery = req.query()?;

    // Get shared state
    let state: AppState = req.state()?;

    let users = state.users.lock().await;
    let limit = query.limit.unwrap_or(users.len());
    let result: Vec<User> = users.iter().take(limit).cloned().collect();

    Ok(Json(result))
}

async fn get_user(req: Request) -> Result<Json<User>, AppError> {
    // Extract path parameter
    let id: u64 = req.path_param("id")?;

    // Get shared state
    let state: AppState = req.state()?;

    let users = state.users.lock().await;
    users
        .iter()
        .find(|u| u.id == id)
        .cloned()
        .map(Json)
        .ok_or(AppError::NotFound)
}

async fn create_user(req: Request) -> Result<Json<User>, AppError> {
    // Get shared state
    let state: AppState = req.state()?;

    // Parse JSON body
    let body: CreateUser = json_body(req).await?;

    // Validation
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }
    if body.email.trim().is_empty() {
        return Err(AppError::BadRequest("email cannot be empty".into()));
    }

    // Create user
    let mut users = state.users.lock().await;
    let id = users.len() as u64 + 1;
    let user = User {
        id,
        name: body.name,
        email: body.email,
    };
    users.push(user.clone());

    Ok(Json(user))
}

async fn delete_user(req: Request) -> Result<&'static str, AppError> {
    // Extract path parameter
    let id: u64 = req.path_param("id")?;

    // Get shared state
    let state: AppState = req.state()?;

    let mut users = state.users.lock().await;
    let before = users.len();
    users.retain(|u| u.id != id);

    if users.len() == before {
        return Err(AppError::NotFound);
    }

    Ok("User deleted successfully")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,user_crud=debug".into()),
        )
        .init();

    // Create shared application state
    let state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    // Get port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Build the application with routes and middleware
    let app = App::new(state)
        .middleware(RequestId) // Add request ID to each request
        .middleware(Logger) // Log requests and responses
        .get("/", root)
        .get("/health", health)
        .get("/users", list_users)
        .get("/users/{id}", get_user)
        .post("/users", create_user)
        .delete("/users/{id}", delete_user);

    println!("🚀 User CRUD API running on http://{}", addr);
    println!("📝 Visit http://{} for API documentation", addr);
    println!();
    println!("Example curl commands:");
    println!("  # List users");
    println!("  curl http://{}/users", addr);
    println!();
    println!("  # Create user");
    println!("  curl -X POST http://{}/users \\", addr);
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}'");
    println!();

    app.listen(&addr).await
}
