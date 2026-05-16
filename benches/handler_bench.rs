use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trainpress::{App, Json, Request, AppError};
use trainpress::extract::RequestExt;
use trainpress::handler::into_handler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper::Method;

#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<u64>>,
    users: Arc<Mutex<Vec<User>>>,
}

// Simple handler - no state, no extraction
async fn simple_handler(_req: Request) -> Json<&'static str> {
    Json("Hello, World!")
}

// Handler with JSON response
async fn json_handler(_req: Request) -> Json<User> {
    Json(User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

// Handler with path parameter extraction
async fn path_param_handler(req: Request) -> Result<Json<User>, AppError> {
    let id: u64 = req.path_param("id")?;

    Ok(Json(User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    }))
}

// Handler with state access
async fn state_handler(req: Request) -> Result<Json<u64>, AppError> {
    let state: AppState = req.state()?;
    let mut counter = state.counter.lock().await;
    *counter += 1;
    Ok(Json(*counter))
}

// Handler with simple response
async fn echo_handler(_req: Request) -> Json<&'static str> {
    Json("echo")
}

fn handler_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler");

    // Benchmark: Handler conversion
    group.bench_function("handler_conversion", |b| {
        b.iter(|| {
            black_box(into_handler(simple_handler))
        });
    });

    // Benchmark: JSON handler conversion
    group.bench_function("json_handler_conversion", |b| {
        b.iter(|| {
            black_box(into_handler(json_handler))
        });
    });

    // Benchmark: State handler conversion
    group.bench_function("state_handler_conversion", |b| {
        b.iter(|| {
            black_box(into_handler(state_handler))
        });
    });

    // Benchmark: Router lookup with handler
    group.bench_function("router_with_handler", |b| {
        let app = App::new_stateless()
            .get("/", simple_handler)
            .get("/users/{id}", path_param_handler);

        b.iter(|| {
            let result = app.router.find(&Method::GET, "/");
            black_box(result)
        });
    });

    // Benchmark: Multiple route registrations
    group.bench_function("app_with_many_routes", |b| {
        b.iter(|| {
            let mut app = App::new_stateless();

            for i in 0..50 {
                app = app.get(&format!("/route{}", i), simple_handler);
            }

            black_box(app)
        });
    });

    // Benchmark: App builder pattern
    group.bench_function("app_builder_pattern", |b| {
        b.iter(|| {
            let app = App::new_stateless()
                .get("/", simple_handler)
                .get("/users", json_handler)
                .get("/users/{id}", path_param_handler)
                .post("/echo", echo_handler);

            black_box(app)
        });
    });

    // Benchmark: State initialization
    group.bench_function("app_with_state_init", |b| {
        b.iter(|| {
            let state = AppState {
                counter: Arc::new(Mutex::new(0)),
                users: Arc::new(Mutex::new(Vec::new())),
            };

            let app = App::new(state)
                .get("/counter", state_handler);

            black_box(app)
        });
    });

    group.finish();
}

criterion_group!(benches, handler_benchmarks);
criterion_main!(benches);
