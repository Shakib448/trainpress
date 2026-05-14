# TrainPress

A minimal, educational HTTP web framework powered by **Hyper** - one of the fastest and most reliable HTTP implementations in any language.

## Overview

TrainPress is a production-ready micro-framework that brings Hyper's exceptional performance directly to your applications with minimal abstraction. Inspired by Axum's design philosophy but with a streamlined codebase, TrainPress is perfect for learning how modern web frameworks work while delivering the raw speed and reliability that Hyper is renowned for.

## Features

### Core Functionality
- **High-Performance Routing** - Trie-based path matching using `matchit` (same as Axum)
- **RESTful API Support** - Full HTTP method support (GET, POST, PUT, PATCH, DELETE)
- **Dynamic Path Parameters** - Type-safe parameter extraction from URLs
- **Query String Parsing** - Automatic deserialization with serde
- **JSON Request/Response** - Built-in JSON body handling
- **Shared State Management** - Thread-safe application state injection

### Middleware System
- **Onion Pattern Pipeline** - Composable middleware chain
- **Built-in Middleware**:
  - Logger - Request/response logging with timing
  - RequestId - Request ID injection and propagation
- **Custom Middleware Support** - Easy to implement custom middleware

### Production Features
- **Graceful Shutdown** - Proper connection draining on CTRL+C/SIGTERM
- **Structured Logging** - Tracing-based observability
- **Error Handling** - Automatic HTTP status mapping with JSON error responses
- **HTTP/1.1 & HTTP/2** - Auto-negotiation support
- **Type Safety** - Compile-time guarantees via Rust's type system

## Technology Stack

- **Hyper** 1.x - Battle-tested, blazingly fast HTTP implementation trusted by industry leaders
- **Rust** (Edition 2024) - Memory-safe systems programming
- **Tokio** - High-performance async runtime
- **matchit** - Zero-copy path routing with O(log n) complexity
- **Serde** - Fast serialization/deserialization
- **Tracing** - Zero-cost structured logging

## Getting Started

### Prerequisites

- Rust 1.75 or higher
- Cargo

### Installation

Add TrainPress to your `Cargo.toml`:

```toml
[dependencies]
trainpress = { path = "." }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### Quick Start

```rust
use trainpress::{App, Json, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Message {
    text: String,
}

async fn hello() -> Json<Message> {
    Json(Message {
        text: "Hello, World!".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()
        .get("/", hello)
        .build()?;

    app.serve("127.0.0.1:3000").await
}
```

## Usage Examples

### Path Parameters

```rust
use trainpress::{Path, Json, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserParams {
    id: String,
}

async fn get_user(Path(params): Path<UserParams>) -> Result<Json<User>> {
    let user = find_user(&params.id).await?;
    Ok(Json(user))
}

app.get("/users/{id}", get_user);
```

### Query Strings

```rust
use trainpress::{Query, Json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: usize,
    limit: usize,
}

async fn list_users(Query(params): Query<Pagination>) -> Json<Vec<User>> {
    let users = fetch_users(params.page, params.limit).await;
    Json(users)
}

app.get("/users", list_users);
```

### JSON Request Body

```rust
use trainpress::{Json, AppError, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(Json(payload): Json<CreateUser>) -> Result<(StatusCode, Json<User>)> {
    let user = User::create(payload.name, payload.email).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

app.post("/users", create_user);
```

### Shared State

```rust
use trainpress::{App, State, Json};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: DatabasePool,
}

async fn handler(State(state): State<Arc<AppState>>) -> Json<String> {
    let result = state.db.query("SELECT 1").await;
    Json(result)
}

let state = Arc::new(AppState { db: create_pool().await });
let app = App::with_state(state)
    .get("/", handler)
    .build()?;
```

### Custom Middleware

```rust
use trainpress::{Middleware, Next, Request, Response, Result};

struct CustomMiddleware;

impl Middleware for CustomMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Result<Response> {
        // Pre-handler logic
        println!("Before request");

        let response = next.run(req).await?;

        // Post-handler logic
        println!("After request");

        Ok(response)
    }
}

app.layer(CustomMiddleware);
```

## Project Structure

```
trainpress/
├── src/
│   ├── lib.rs         # Public API exports
│   ├── app.rs         # Application builder
│   ├── router.rs      # Path routing
│   ├── handler.rs     # Handler type definitions
│   ├── middleware.rs  # Middleware system
│   ├── server.rs      # TCP listener and shutdown
│   ├── response.rs    # Response type conversions
│   ├── extract.rs     # Request extractors
│   └── error.rs       # Error types and handling
└── tiny-server/       # Example implementation
```

## Design Philosophy

TrainPress is intentionally minimal, focusing on:

- **Simplicity over features** - Easy to understand codebase
- **Educational value** - Learn web framework internals
- **Production basics** - Essential features without bloat
- **Type safety** - Leverage Rust's type system
- **Performance** - Built on battle-tested libraries

### Current Limitations

To maintain simplicity and focus on core concepts, TrainPress currently excludes certain advanced features.

## Evolution Path: Becoming Axum-Like

If you wanted to evolve TrainPress into a full-featured Axum-like framework, consider implementing these features:

### Type System & Extractors
- **Function Argument Extractors** - Extract data directly from handler function parameters using Rust's type system
- **FromRequest Trait** - Automatic conversion of request data to typed parameters
- **Multiple Extractors** - Support extracting multiple types in a single handler (e.g., `State`, `Json`, `Path` simultaneously)
- **Custom Extractors** - Allow users to define their own extractor types

### Routing & Composition
- **Nested Routers** - Mount sub-routers at specific paths for modular application structure
- **Route Groups** - Apply middleware to groups of routes
- **Fallback Handlers** - Custom 404 and method-not-allowed handlers
- **Routing Layers** - Apply transformations at the routing level

### Tower Integration
- **Tower Service Trait** - Full compatibility with Tower's service ecosystem
- **Tower Middleware** - Use any Tower middleware (timeout, rate limiting, compression, etc.)
- **Service Composition** - Stack services and middleware using Tower's combinators
- **Layer System** - Rich middleware composition via Tower layers

### Advanced Protocol Support
- **WebSocket** - Full-duplex communication for real-time applications
- **Server-Sent Events (SSE)** - Push updates to clients over HTTP
- **Streaming Bodies** - Efficient handling of large request/response bodies
- **Multipart Forms** - File upload support with streaming

### Response Types & Negotiation
- **Content Negotiation** - Automatic format selection based on Accept headers
- **Response Headers API** - Builder pattern for setting headers
- **Streaming Responses** - AsyncRead/Stream support for large responses
- **Custom Response Types** - More flexible IntoResponse implementations

### State & Dependency Injection
- **Layered State** - State at router and route levels
- **Request Extensions** - Richer extension API for request-scoped data
- **Dependency Injection** - Constructor injection for handlers

### Middleware Enhancements
- **Async Middleware Traits** - More flexible middleware API
- **Error Handling Middleware** - Catch and transform errors at different layers
- **Built-in Middleware Library** - CORS, compression, rate limiting, auth, etc.

### Testing & Development
- **Testing Utilities** - Test client for making requests to your app
- **Mock Support** - Easy testing without starting a server
- **Hot Reload** - Development-mode automatic recompilation

### Observability & Diagnostics
- **OpenTelemetry Integration** - Distributed tracing and metrics
- **Request Tracing** - Detailed request lifecycle logging
- **Health Check Endpoints** - Built-in health and readiness probes
- **Metrics Collection** - Prometheus-compatible metrics

### Performance Optimizations
- **Connection Pooling** - Efficient reuse of connections
- **Response Caching** - Built-in HTTP cache support
- **Compression Middleware** - Gzip, Brotli, Zstandard support
- **Static File Serving** - Optimized serving with ETag support

Implementing these features would transform TrainPress into a comprehensive web framework comparable to Axum, while maintaining its educational value and Hyper's performance foundation.

## Example Application

Check out the `tiny-server/` directory for a complete CRUD API implementation demonstrating:
- User management endpoints
- Database integration patterns
- Error handling strategies
- Middleware usage
- State management

## Performance

TrainPress is built directly on **Hyper**, which powers some of the most performance-critical web services in production:

### Why Hyper?

- **Industry-Leading Speed** - Consistently ranks among the fastest HTTP servers in TechEmpower benchmarks
- **Zero-Copy Parsing** - Minimal memory allocations and CPU overhead
- **HTTP/1 and HTTP/2** - Native support for modern protocols with automatic negotiation
- **Production-Proven** - Used by AWS, Cloudflare, Discord, and countless high-traffic services
- **Memory Safety** - Rust's ownership model prevents entire classes of bugs without runtime cost

### Additional Performance Features

- **matchit Router** - O(log n) path matching with zero-copy parameter extraction
- **Tokio Runtime** - Work-stealing scheduler for optimal CPU utilization
- **Minimal Abstraction** - Thin layer over Hyper preserves raw performance
- **Efficient Connection Handling** - Per-connection task spawning with low overhead

TrainPress gives you Hyper's exceptional performance with an ergonomic, type-safe API.

## Contributing

This is an educational project. Feel free to:
- Report issues
- Suggest improvements
- Submit pull requests
- Use it as a learning resource

## License

This project is open source and available for educational purposes.

## Acknowledgments

- Inspired by [Axum](https://github.com/tokio-rs/axum)
- Built on [Hyper](https://github.com/hyperium/hyper)
- Uses [matchit](https://github.com/ibraheemdev/matchit) for routing

---

Built with Rust for learning and production use.