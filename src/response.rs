use hyper::body::Bytes;
use hyper::StatusCode;

use crate::error::AppError;
use crate::{full, Response};

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        plain_text(self, StatusCode::OK)
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        plain_text(self.to_string(), StatusCode::OK)
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(self)
            .body(full(Bytes::new()))
            .expect("valid response")
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Response {
        let mut resp = self.1.into_response();
        *resp.status_mut() = self.0;
        resp
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(full(Bytes::new()))
            .expect("valid response")
    }
}

pub struct Json<T>(pub T);

impl<T: serde::Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self.0) {
            Ok(bytes) => hyper::Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(full(bytes))
                .expect("valid response"),
            Err(e) => AppError::Internal(format!("json serialize: {e}")).into_response(),
        }
    }
}

fn plain_text(body: String, status: StatusCode) -> Response {
    hyper::Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(full(body))
        .expect("valid response")
}
