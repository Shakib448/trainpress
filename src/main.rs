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
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Deserialize, Default)]
struct ListQuery {
    limit: Option<usize>,
}

async fn root(_req: Request) -> &'static str {
    "🦀 Welcome to tiny-server!\n\nTry: GET /users, POST /users, GET /health"
}

async fn health(_req: Request) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "tiny-server",
    }))
}

async fn list_users(req: Request) -> Result<Json<Vec<User>>, AppError> {
    let query: ListQuery = req.query()?;
    let state: AppState = req.state()?;

    let users = state.users.lock().await;
    let limit = query.limit.unwrap_or(users.len());
    let result: Vec<User> = users.iter().take(limit).cloned().collect();

    Ok(Json(result))
}

async fn get_user(req: Request) -> Result<Json<User>, AppError> {
    let id: u64 = req.path_param("id")?;
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
    let state: AppState = req.state()?;
    let body: CreateUser = json_body(req).await?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".into()));
    }

    let mut users = state.users.lock().await;
    let id = users.len() as u64 + 1;
    let user = User {
        id,
        name: body.name,
    };
    users.push(user.clone());

    Ok(Json(user))
}

async fn delete_user(req: Request) -> Result<&'static str, AppError> {
    let id: u64 = req.path_param("id")?;
    let state: AppState = req.state()?;

    let mut users = state.users.lock().await;
    let before = users.len();
    users.retain(|u| u.id != id);

    if users.len() == before {
        return Err(AppError::NotFound);
    }
    Ok("deleted")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tiny_server=debug".into()),
        )
        .init();

    let state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    // ---- Address: PORT env var থেকে নিই, default 3000 ----
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = App::new(state)
        .middleware(RequestId)
        .middleware(Logger)
        .get("/", root)
        .get("/health", health)
        .get("/users", list_users)
        .get("/users/{id}", get_user)
        .post("/users", create_user)
        .delete("/users/{id}", delete_user);

    app.listen(&addr).await?;

    Ok(())
}
