use serde::Serialize;
/// Example demonstrating custom middleware implementation in TrainPress
use trainpress::{
    App, BoxFuture, Json, Request, Response,
    middleware::{Logger, Middleware, Next, RequestId},
};

/// Timing middleware - measures request processing time
struct TimingMiddleware;

impl Middleware for TimingMiddleware {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            let start = std::time::Instant::now();
            let path = req.uri().path().to_string();

            let response = next.run(req).await;

            let duration = start.elapsed();
            println!("⏱️  {} took {:?}", path, duration);

            response
        })
    }
}

/// Custom header middleware - adds custom headers to responses
struct CustomHeaderMiddleware {
    header_name: String,
    header_value: String,
}

impl Middleware for CustomHeaderMiddleware {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
        let name = self.header_name.clone();
        let value = self.header_value.clone();

        Box::pin(async move {
            let mut response = next.run(req).await;

            // Add custom header to response
            if let (Ok(header_name), Ok(header_value)) = (
                name.parse::<hyper::header::HeaderName>(),
                value.parse::<hyper::header::HeaderValue>(),
            ) {
                response.headers_mut().insert(header_name, header_value);
            }

            response
        })
    }
}

/// Request counter middleware - counts total requests
struct RequestCounterMiddleware {
    counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Middleware for RequestCounterMiddleware {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
        let counter = self.counter.clone();

        Box::pin(async move {
            let count = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            println!("📊 Request #{}", count + 1);

            next.run(req).await
        })
    }
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    middleware_count: usize,
}

async fn hello(_req: Request) -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Hello! This response passed through multiple middleware layers".to_string(),
        middleware_count: 5, // We have 5 middleware in the chain
    })
}

async fn info(_req: Request) -> &'static str {
    "This API demonstrates middleware composition:\n\
     1. RequestId - Adds request ID\n\
     2. Logger - Logs requests\n\
     3. TimingMiddleware - Measures duration\n\
     4. CustomHeaderMiddleware - Adds X-Powered-By header\n\
     5. RequestCounterMiddleware - Counts total requests\n"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Create request counter
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    // Build app with multiple middleware layers
    let app = App::new_stateless()
        // Built-in middleware
        .middleware(RequestId)
        .middleware(Logger)
        // Custom middleware
        .middleware(TimingMiddleware)
        .middleware(CustomHeaderMiddleware {
            header_name: "X-Powered-By".to_string(),
            header_value: "TrainPress/Hyper".to_string(),
        })
        .middleware(RequestCounterMiddleware { counter })
        // Routes
        .get("/", hello)
        .get("/info", info);

    println!("🚀 Middleware example running on http://127.0.0.1:3000");
    println!("📝 Try:");
    println!("   curl -v http://127.0.0.1:3000");
    println!("   curl http://127.0.0.1:3000/info");
    println!();
    println!("Watch the terminal for middleware output!");

    app.listen("127.0.0.1:3000").await
}
