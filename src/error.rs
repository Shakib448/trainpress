use crate::{full, Response};
use hyper::StatusCode;
use thiserror::Error;

use crate::response::IntoResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid query: {0}")]
    Query(#[from] serde_urlencoded::de::Error),

    #[error("body error: {0}")]
    Body(String),

    #[error("path param error: {0}")]
    PathParam(String),

    #[error("state not found in request extensions")]
    MissingState,

    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_)
            | AppError::Json(_)
            | AppError::Query(_)
            | AppError::PathParam(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Body(_) | AppError::MissingState | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        if status.is_server_error() {
            tracing::error!(error = %self, "internal error");
        }

        let body = serde_json::json!({
            "error" : self.to_string(),
        });

        let bytes = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());

        hyper::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(full(bytes))
            .expect("Valid response - static fields")
    }
}
