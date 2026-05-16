use trainpress::{App, Json, Request};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    text: String,
}

async fn hello(_req: Request) -> Json<Message> {
    Json(Message {
        text: "Hello from TrainPress!".to_string(),
    })
}

async fn health(_req: Request) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "framework": "TrainPress"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app = App::new_stateless()
        .get("/", hello)
        .get("/health", health);

    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📝 Try: http://127.0.0.1:3000 or http://127.0.0.1:3000/health");

    app.listen("127.0.0.1:3000").await
}
