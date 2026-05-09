use http::{Method, StatusCode};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as AutoBuilder;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

type ResponseBody = Full<Bytes>;
type BoxFuture = Pin<Box<dyn Future<Output = Response<ResponseBody>> + Send>>;
type Handler = Arc<dyn Fn(Request<Incoming>) -> BoxFuture + Send + Sync>;

struct App {
    routes: HashMap<(Method, String), Handler>,
}

impl App {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResponseBody>> + Send + 'static,
    {
        let handler = Arc::new(move |req| Box::pin(handler(req)) as BoxFuture);
        self.routes.insert((Method::GET, path.to_string()), handler);
    }

    fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response<ResponseBody>> + Send + 'static,
    {
        let handler = Arc::new(move |req| Box::pin(handler(req)) as BoxFuture);
        self.routes.insert((Method::GET, path.to_string()), handler);
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

fn text(body: &str) -> Response<ResponseBody> {
    Response::new(Full::new(Bytes::from(body.to_string())))
}

fn not_found() -> Response<ResponseBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("404 Not Found")))
        .unwrap()
}

async fn hello(_req: Request<Incoming>) -> Response<ResponseBody> {
    text("Hello from Rust Express-like server")
}

async fn users(_req: Request<Incoming>) -> Response<ResponseBody> {
    text("Users route")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = App::new();

    app.get("/", hello);
    app.get("/users", users);
    app.post("/post", users);

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
