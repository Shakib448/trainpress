mod app;
mod error;
mod handler;
mod response;
mod router;

use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use std::convert::Infallible;
use std::pin::Pin;

pub use response::{IntoResponse, Json};

pub type Body = BoxBody<Bytes, Infallible>;
pub type Request = hyper::Request<Incoming>;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type Response = hyper::Response<BoxBody<Bytes, Infallible>>;

pub fn full(body: impl Into<Bytes>) -> Body {
    Full::new(body.into()).boxed()
}
