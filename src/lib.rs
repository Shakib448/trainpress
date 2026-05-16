pub mod app;
pub mod error;
pub mod extract;
pub mod handler;
pub mod middleware;
pub mod response;
pub mod router;
pub mod server;

use std::convert::Infallible;
use std::pin::Pin;

pub use app::App;
pub use error::AppError;
pub use handler::Handler;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
pub use response::{IntoResponse, Json};

pub type Body = BoxBody<Bytes, Infallible>;
pub type Request = hyper::Request<hyper::body::Incoming>;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type Response = hyper::Response<BoxBody<Bytes, Infallible>>;

pub fn full(body: impl Into<Bytes>) -> Body {
    Full::new(body.into()).boxed()
}
