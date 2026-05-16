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

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[test]
    fn test_not_found_status() {
        let err = AppError::NotFound;
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_bad_request_status() {
        let err = AppError::BadRequest("test".into());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_unauthorized_status() {
        let err = AppError::Unauthorized;
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_json_error_status() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = AppError::Json(json_err);
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_query_error_status() {
        let query_err = serde_urlencoded::from_str::<()>("invalid=&").unwrap_err();
        let err = AppError::Query(query_err);
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_body_error_status() {
        let err = AppError::Body("body error".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_path_param_error_status() {
        let err = AppError::PathParam("missing param".into());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_missing_state_status() {
        let err = AppError::MissingState;
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_internal_error_status() {
        let err = AppError::Internal("something broke".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let err = AppError::NotFound;
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["error"], "not found");
    }

    #[tokio::test]
    async fn test_bad_request_response() {
        let err = AppError::BadRequest("invalid input".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert!(body["error"].as_str().unwrap().contains("invalid input"));
    }

    #[tokio::test]
    async fn test_unauthorized_response() {
        let err = AppError::Unauthorized;
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["error"], "unauthorized");
    }

    #[test]
    fn test_error_display() {
        assert_eq!(AppError::NotFound.to_string(), "not found");
        assert_eq!(
            AppError::BadRequest("test".into()).to_string(),
            "bad request: test"
        );
        assert_eq!(AppError::Unauthorized.to_string(), "unauthorized");
        assert_eq!(
            AppError::PathParam("id missing".into()).to_string(),
            "path param error: id missing"
        );
        assert_eq!(
            AppError::MissingState.to_string(),
            "state not found in request extensions"
        );
        assert_eq!(
            AppError::Internal("boom".into()).to_string(),
            "internal: boom"
        );
    }
}
