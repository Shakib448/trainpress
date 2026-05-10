use http::{Method, StatusCode};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

type ResponseBody = Full<Bytes>;
type BoxFuture = Pin<Box<dyn Future<Output = Response<ResponseBody>> + Send>>;
type Handler = Arc<dyn Fn(Request<Incoming>) -> BoxFuture + Send + Sync>;

trait IntoResponse {
    fn into_response(self) -> Response<ResponseBody>;
}
impl IntoResponse for Response<ResponseBody> {
    fn into_response(self) -> Response<ResponseBody> {
        self
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response<ResponseBody> {
        Response::builder()
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from(self)))
            .unwrap()
    }
}

struct App {
    routes: HashMap<(Method, String), Handler>,
}
struct Json<T>(T);

impl<T> IntoResponse for Json<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response<ResponseBody> {
        let body = serde_json::to_string(&self.0).unwrap();
        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap()
    }
}

impl App {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn route<F, Fut, R>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        let handler = Arc::new(handler);

        let handler_wrapper: Handler = Arc::new(move |req| {
            let handler = Arc::clone(&handler);

            Box::pin(async move { handler(req).await.into_response() })
        });

        self.routes
            .insert((method, path.to_string()), handler_wrapper);
    }

    fn get<F, Fut, R>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::GET, path, handler);
    }

    fn post<F, Fut, R>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::POST, path, handler);
    }

    fn put<F, Fut, R>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::PUT, path, handler);
    }

    fn patch<F, Fut, R>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,

        Fut: Future<Output = R> + Send + 'static,

        R: IntoResponse + 'static,
    {
        self.route(Method::PATCH, path, handler);
    }

    fn delete<F, Fut, R>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::DELETE, path, handler);
    }

    async fn listen(self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = addr.parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;

        let routes = Arc::new(self.routes);

        loop {
            let (stream, _) = listener.accept().await?;

            let io = TokioIo::new(stream);

            let routes = routes.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let routes = routes.clone();

                    async move {
                        let key = (req.method().clone(), req.uri().path().to_string());

                        if let Some(handler) = routes.get(&key) {
                            let response = handler(req).await;

                            Ok::<_, Infallible>(response)
                        } else {
                            Ok::<_, Infallible>(not_found())
                        }
                    }
                });

                let builder = AutoBuilder::new(TokioExecutor::new());

                if let Err(err) = builder.serve_connection(io, service).await {
                    eprintln!("connection error: {:?}", err);
                }
            });
        }
    }
}

async fn hello(_req: Request<Incoming>) -> &'static str {
    "hello world"
}

#[derive(Serialize)]
struct User {
    name: String,
}
async fn user(_req: Request<Incoming>) -> Json<User> {
    Json(User {
        name: "Shakib".to_string(),
    })
}

fn text(body: &str) -> Response<ResponseBody> {
    Response::new(Full::new(Bytes::from(body.to_string())))
}

fn not_found() -> Response<ResponseBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("404 Not Found")))
        .unwrap()
}

async fn users(_req: Request<Incoming>) -> Response<ResponseBody> {
    text("Users route")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();

    app.get("/", hello);
    app.get("/users", user);
    app.post("/post", users);
    app.patch("/patch", users);
    app.put("/put", users);
    app.delete("/delete", users);

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
