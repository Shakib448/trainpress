# TrainPress

**A production-ready web framework powered by Hyper - one of the fastest and most reliable HTTP implementations in any programming language.**

<div align="center">

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

TrainPress brings you **blazing-fast HTTP performance** with minimal abstraction. Built directly on Hyper 1.x, it delivers the raw speed and reliability that powers some of the world's most demanding web services - including AWS, Cloudflare, and Discord.

</div>

---

## Table of Contents

- [Why TrainPress?](#why-trainpress)
- [Performance](#performance)
- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage Examples](#usage-examples)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [Architecture](#architecture)
- [Design Philosophy](#design-philosophy)
- [Contributing](#contributing)

---

## Why TrainPress?

TrainPress is designed for developers who need **production-grade performance** without sacrificing developer experience. By building directly on Hyper - the battle-tested HTTP library trusted by industry leaders - you get:

### **Uncompromising Speed**
- **Zero-copy parsing** - Minimal memory allocations
- **O(log n) routing** - Lightning-fast path matching via matchit
- **Work-stealing scheduler** - Optimal CPU utilization with Tokio
- **HTTP/1 & HTTP/2** - Automatic protocol negotiation

### **Production Reliability**
- **Memory safety** - Rust's ownership model prevents entire classes of bugs
- **Graceful shutdown** - Proper connection draining on SIGTERM/CTRL+C
- **Type-safe extractors** - Compile-time guarantees for request handling
- **Structured logging** - Tracing-based observability built-in

### **Educational Value**
- **Readable codebase** - Learn how modern web frameworks work
- **Minimal abstraction** - Clear path from your code to Hyper
- **Well-documented** - Examples and explanations throughout

---

## Performance

### Why Hyper?

**Hyper** consistently ranks among the fastest HTTP servers across all languages in the [TechEmpower benchmarks](https://www.techempower.com/benchmarks/). TrainPress gives you direct access to this performance:

| Feature | Benefit |
|---------|---------|
| **Zero-allocation parsing** | Minimal memory overhead per request |
| **Async I/O** | Handle thousands of concurrent connections |
| **HTTP/2 multiplexing** | Efficient connection reuse |
| **Optimized buffer handling** | Reduced syscalls and memory copies |

### Real-World Usage

Hyper powers some of the most performance-critical services in production:
- **AWS** - Lambda and API Gateway
- **Cloudflare** - Edge computing platform
- **Discord** - Real-time messaging infrastructure
- **Countless** high-traffic microservices

TrainPress preserves this performance while providing an ergonomic,type-safe API.

---

## Features

### Core Functionality

✨ **High-Performance Routing**
- Trie-based path matching using `matchit` (same as Axum)
- O(log n) lookup complexity
- Zero-copy parameter extraction

🌐 **Full HTTP Method Support**
- GET, POST, PUT, PATCH, DELETE
- Method-specific handler registration
- Type-safe path parameters

📦 **Request Extraction**
- **Path parameters** - `Path<T>` extractor with automatic parsing
- **Query strings** - Automatic deserialization via serde
- **JSON bodies** - Built-in JSON request/response handling
- **Shared state** - Thread-safe application state injection

### Middleware System

🔗 **Composable Pipeline**
- **Onion pattern** - Request flows through middleware layers
- **Built-in middleware**:
  - `Logger` - Request/response logging with timing
  - `RequestId` - Request ID injection and propagation
- **Custom middleware** - Easy trait implementation

### Production Features

⚡ **Performance**
- Built on Hyper 1.x - Industry-leading HTTP performance
- Tokio async runtime - Efficient concurrent request handling
- Minimal abstraction overhead

🛡️ **Reliability**
- Graceful shutdown with connection draining
- Structured logging via `tracing`
- Automatic HTTP status code mapping
- Type-safe error handling

📊 **Observability**
- Request/response logging
- Timing metrics
- Request ID tracking
- Integration-ready with tracing ecosystem

---

## Quick Start

### Prerequisites

- **Rust 1.75+** (2024 edition)
- **Cargo**

### Try the Examples

TrainPress includes several examples to get you started:

```bash
# Clone the repository
git clone https://github.com/Shakib448/trainpress
cd trainpress

# Run hello world example
cargo run --example hello_world

# Run full CRUD API with state management
cargo run --example user_crud

# Run custom middleware demonstration
cargo run --example middleware_example
```

### Installation

Add TrainPress to your `Cargo.toml`:

```toml
[dependencies]
trainpress = { path = "." }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Hello World

```rust
use trainpress::{App, Json};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    text: String,
}

async fn hello(_req: trainpress::Request) -> Json<Message> {
    Json(Message {
        text: "Hello from TrainPress!".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new_stateless()
        .get("/", hello);

    println!("🚀 Server running on http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000").await
}
```

Save this as `examples/my_app.rs` and run:

```bash
cargo run --example my_app
```

Visit `http://127.0.0.1:3000` to see your response!

---

## Usage Examples

### Path Parameters

Extract typed parameters from URL paths:

```rust
use trainpress::{Request, Json, AppError, extract::RequestExt};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn get_user(req: Request) -> Result<Json<User>, AppError> {
    let id: u64 = req.path_param("id")?;

    // Fetch user from database...
    let user = User {
        id,
        name: format!("User {}", id),
    };

    Ok(Json(user))
}

// Register the route
app.get("/users/{id}", get_user);
```

### Query Parameters

Parse query strings with automatic deserialization:

```rust
use trainpress::{Request, Json, extract::RequestExt};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    limit: usize,
}

async fn list_items(req: Request) -> Result<Json<Vec<String>>, AppError> {
    let params: Pagination = req.query()?;

    // Use params.page and params.limit for pagination
    let items = fetch_paginated_items(params.page, params.limit).await;

    Ok(Json(items))
}

app.get("/items", list_items);
```

### JSON Request Bodies

Handle JSON payloads with type safety:

```rust
use trainpress::{Request, Json, AppError, extract::json_body};
use serde::{Deserialize, Serialize};
use hyper::StatusCode;

#[derive(Deserialize)]
struct CreatePost {
    title: String,
    content: String,
}

#[derive(Serialize)]
struct Post {
    id: u64,
    title: String,
    content: String,
}

async fn create_post(req: Request) -> Result<(StatusCode, Json<Post>), AppError> {
    let payload: CreatePost = json_body(req).await?;

    let post = Post {
        id: 1,
        title: payload.title,
        content: payload.content,
    };

    Ok((StatusCode::CREATED, Json(post)))
}

app.post("/posts", create_post);
```

### Shared Application State

Inject thread-safe state into handlers:

```rust
use trainpress::{App, Request, Json, extract::RequestExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    counter: Arc<Mutex<u64>>,
}

async fn increment(req: Request) -> Result<Json<u64>, AppError> {
    let state: AppState = req.state()?;

    let mut counter = state.counter.lock().await;
    *counter += 1;

    Ok(Json(*counter))
}

let state = AppState {
    counter: Arc::new(Mutex::new(0)),
};

let app = App::new(state)
    .get("/increment", increment);
```

### Custom Middleware

Implement custom middleware for cross-cutting concerns:

```rust
use trainpress::{Middleware, Request, Response, middleware::Next, BoxFuture};

struct AuthMiddleware {
    secret_token: String,
}

impl Middleware for AuthMiddleware {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
        let secret = self.secret_token.clone();

        Box::pin(async move {
            // Check authorization header
            let auth_header = req.headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok());

            if auth_header != Some(&secret) {
                use trainpress::response::IntoResponse;
                return trainpress::AppError::Unauthorized.into_response();
            }

            next.run(req).await
        })
    }
}

app.middleware(AuthMiddleware {
    secret_token: "my-secret-token".to_string(),
});
```

### Complete Application Example

```rust
use trainpress::{App, Request, Json, AppError};
use trainpress::middleware::{Logger, RequestId};
use trainpress::extract::{RequestExt, json_body};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<User>>>,
}

#[derive(Clone, Serialize, Deserialize)]
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

async fn health(_req: Request) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn list_users(req: Request) -> Result<Json<Vec<User>>, AppError> {
    let state: AppState = req.state()?;
    let users = state.users.lock().await;
    Ok(Json(users.clone()))
}

async fn get_user(req: Request) -> Result<Json<User>, AppError> {
    let id: u64 = req.path_param("id")?;
    let state: AppState = req.state()?;

    let users = state.users.lock().await;
    users.iter()
        .find(|u| u.id == id)
        .cloned()
        .map(Json)
        .ok_or(AppError::NotFound)
}

async fn create_user(req: Request) -> Result<Json<User>, AppError> {
    let state: AppState = req.state()?;
    let payload: CreateUser = json_body(req).await?;

    let mut users = state.users.lock().await;
    let id = (users.len() + 1) as u64;

    let user = User {
        id,
        name: payload.name,
        email: payload.email,
    };

    users.push(user.clone());
    Ok(Json(user))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create shared state
    let state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    // Build application
    let app = App::new(state)
        .middleware(RequestId)
        .middleware(Logger)
        .get("/health", health)
        .get("/users", list_users)
        .get("/users/{id}", get_user)
        .post("/users", create_user);

    println!("🚀 Server running on http://0.0.0.0:3000");
    app.listen("0.0.0.0:3000").await
}
```

---

## Testing

TrainPress includes comprehensive test coverage across all modules.

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage

- **Unit Tests**: 48 tests covering all core modules
  - Router path matching and parameter extraction
  - Request extractors (path params, query, JSON body, state)
  - Error handling and HTTP status mapping
  - Response type conversions
  - Middleware composition

- **Integration Tests**: 17 tests covering application assembly
  - Route registration
  - Middleware stacking
  - State management
  - Builder pattern API

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use trainpress::{App, Json, AppError};

    #[tokio::test]
    async fn test_handler() {
        async fn handler(_req: trainpress::Request) -> Json<&'static str> {
            Json("test")
        }

        let app = App::new_stateless().get("/test", handler);

        assert!(app.router.find(&hyper::Method::GET, "/test").is_some());
    }
}
```

---

## Benchmarks

TrainPress includes comprehensive benchmarks using [Criterion.rs](https://github.com/bheisler/criterion.rs) to measure and track performance across releases.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench router_bench
cargo bench --bench json_bench
cargo bench --bench handler_bench

# Run specific benchmark
cargo bench --bench router_bench -- simple_route_match

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

### Benchmark Suites

#### 1. Router Benchmarks (`router_bench`)

Tests the performance of the routing system powered by `matchit`:

- **Simple route matching** - Basic path lookup (`/`)
- **Single parameter extraction** - Routes with one param (`/users/{id}`)
- **Multiple parameters** - Routes with multiple params (`/users/{user_id}/posts/{post_id}`)
- **Deep nested routes** - Complex paths (`/api/v1/users/{id}/posts/{post_id}/comments/{comment_id}`)
- **Many routes lookup** - Performance with 100+ registered routes
- **Route not found** - Lookup performance for non-existent routes
- **Method routing** - Performance across different HTTP methods (GET, POST, PUT, DELETE, PATCH)

#### 2. JSON Benchmarks (`json_bench`)

Measures JSON serialization/deserialization performance:

- **Small payload** - Simple objects (id, name)
- **Medium payload** - Objects with multiple fields and nested data
- **Large payload** - Complex nested structures with arrays
- **Array serialization** - Performance with 10, 100, and 1000 items
- **Serialization** - Converting Rust structs to JSON strings
- **Deserialization** - Parsing JSON strings to Rust structs

#### 3. Handler Benchmarks (`handler_bench`)

Tests handler and application builder performance:

- **Handler conversion** - Converting async functions to handler types
- **JSON handler conversion** - Handler conversion with JSON responses
- **State handler conversion** - Handler conversion with state access
- **Router with handler** - Combined routing and handler lookup
- **Many routes** - Building apps with 50+ routes
- **App builder pattern** - Fluent API performance
- **State initialization** - App creation with shared state

### Benchmark Results

Benchmark reports are generated in `target/criterion/` with:
- **HTML reports** - Visual performance graphs
- **Statistical analysis** - Mean, median, std deviation
- **Regression detection** - Automatic performance regression alerts
- **Historical comparison** - Track performance across commits

### Interpreting Results

```
router/simple_route_match
                        time:   [12.345 ns 12.567 ns 12.789 ns]
                        change: [-5.2% -3.1% -1.0%] (p = 0.01 < 0.05)
                        Performance has improved.
```

- **time**: Measured execution time (lower is better)
- **change**: Performance change vs baseline
- **Performance improved/regressed**: Statistical significance indicator

### Performance Goals

TrainPress aims to maintain:
- **O(log n) routing** - Logarithmic path lookup complexity
- **Sub-microsecond routing** - Route matching in < 1µs for typical apps
- **Zero-copy where possible** - Minimal allocations per request
- **Linear scaling** - Performance scales with route count efficiently

### Writing Custom Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use trainpress::{App, Request, Json};

fn my_benchmarks(c: &mut Criterion) {
    c.bench_function("my_handler", |b| {
        async fn handler(_req: Request) -> Json<&'static str> {
            Json("benchmark")
        }

        b.iter(|| {
            black_box(into_handler(handler))
        });
    });
}

criterion_group!(benches, my_benchmarks);
criterion_main!(benches);
```

---

## Architecture

### Project Structure

```
trainpress/
├── Cargo.toml              # Package configuration
├── README.md               # Documentation
├── CONTRIBUTING.md         # Contribution guidelines
│
├── src/                    # Library source code
│   ├── lib.rs             # Public API and type aliases
│   ├── app.rs             # Application builder and state management
│   ├── router.rs          # Trie-based path routing (matchit)
│   ├── handler.rs         # Handler type definitions and conversion
│   ├── middleware.rs      # Middleware trait and built-in implementations
│   ├── server.rs          # TCP listener, connection handling, graceful shutdown
│   ├── response.rs        # IntoResponse trait implementations
│   ├── extract.rs         # Request extractors (Path, Query, State, Body)
│   └── error.rs           # Error types and HTTP status mapping
│
├── examples/               # Example applications
│   ├── hello_world.rs     # Simple hello world server
│   ├── user_crud.rs       # Full CRUD API with state
│   └── middleware_example.rs  # Custom middleware demo
│
└── tests/                  # Integration tests
    └── integration_tests.rs   # Full application tests
```

### Request Flow

```
Incoming TCP Connection
        ↓
    Hyper HTTP Parser
        ↓
    Router (matchit trie)
        ↓
    Middleware Chain (onion pattern)
        ↓
    Handler Execution
        ↓
    Response Conversion (IntoResponse)
        ↓
    Middleware Response Processing
        ↓
    Hyper HTTP Serialization
        ↓
    TCP Socket
```

### Technology Stack

| Component | Library | Purpose |
|-----------|---------|---------|
| **HTTP** | Hyper 1.x | Protocol implementation |
| **Async Runtime** | Tokio | Work-stealing scheduler |
| **Routing** | matchit | O(log n) path matching |
| **Serialization** | Serde | JSON handling |
| **Logging** | Tracing | Structured observability |

---

## Design Philosophy

TrainPress is built on these core principles:

### 1. **Performance First**
Direct Hyper integration means zero abstraction overhead. You get raw HTTP performance.

### 2. **Type Safety**
Rust's type system prevents bugs at compile time. Extractors, handlers, and responses are all type-checked.

### 3. **Ergonomic API**
Builder pattern, middleware composition, and automatic conversions make common tasks simple.

### 4. **Educational**
Small, readable codebase (~1000 lines). Learn by reading the source.

### 5. **Production Ready**
- Graceful shutdown
- Structured logging
- Error handling
- HTTP/1 & HTTP/2 support

### Current Limitations

To maintain focus on core concepts, TrainPress currently excludes:
- WebSocket support
- Server-Sent Events (SSE)
- Multipart form handling
- Built-in compression middleware
- Tower integration

These features can be added as needed. The codebase is designed to be extended.

---

## Performance Benchmarks

While formal benchmarks are ongoing, TrainPress inherits Hyper's performance characteristics:

- **Requests/sec**: Comparable to other Hyper-based frameworks
- **Latency**: Sub-millisecond for simple routes
- **Memory**: Minimal per-connection overhead
- **Concurrency**: Scales to thousands of connections

The thin abstraction layer adds negligible overhead to raw Hyper performance.

---

## Evolution Path

Want to extend TrainPress? Consider implementing:

- **Tower Integration** - Middleware ecosystem compatibility
- **WebSocket Support** - Full-duplex communication
- **Nested Routers** - Modular application structure
- **Compression Middleware** - Gzip/Brotli/Zstandard
- **Static File Serving** - Optimized asset delivery
- **Testing Utilities** - Request testing without server
- **OpenTelemetry** - Distributed tracing integration

See the [full evolution path](docs/EVOLUTION.md) for detailed guidance.

---

## Contributing

Contributions are welcome! This project serves both as a production tool and an educational resource.

### Ways to Contribute

- **Report bugs** - File issues with reproduction steps
- **Suggest features** - Open discussions for new capabilities
- **Improve documentation** - Clarify examples and explanations
- **Submit PRs** - Bug fixes and feature implementations

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Shakib448/trainpress
cd trainpress

# Run tests
cargo test

# Run the example server
cargo run

# Run with logging
RUST_LOG=debug cargo run
```

---

## License

This project is open source and available under the MIT License.

---

## Acknowledgments

- **Inspired by** [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework design
- **Built on** [Hyper](https://github.com/hyperium/hyper) - Fast, safe, and correct HTTP implementation
- **Routing by** [matchit](https://github.com/ibraheemdev/matchit) - High-performance path matching
- **Powered by** [Tokio](https://tokio.rs) - Asynchronous runtime for Rust

---

<div align="center">

**Built with Rust 🦀 for learning and production use**

[Documentation](#) • [Examples](#usage-examples) • [Contributing](#contributing)

</div>
