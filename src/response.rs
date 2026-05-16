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

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_string_into_response() {
        let response = "Hello, World!".to_string().into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.as_ref(), b"Hello, World!");
    }

    #[tokio::test]
    async fn test_str_into_response() {
        let response = "Static string".into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.as_ref(), b"Static string");
    }

    #[tokio::test]
    async fn test_status_code_into_response() {
        let response = StatusCode::CREATED.into_response();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.len(), 0);
    }

    #[tokio::test]
    async fn test_tuple_status_and_string() {
        let response = (StatusCode::CREATED, "Resource created").into_response();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.as_ref(), b"Resource created");
    }

    #[tokio::test]
    async fn test_tuple_status_and_json() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
        };
        let response = (StatusCode::CREATED, Json(data)).into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["name"], "test");
        assert_eq!(body["count"], 42);
    }

    #[tokio::test]
    async fn test_json_into_response() {
        let data = TestData {
            name: "Alice".to_string(),
            count: 100,
        };
        let response = Json(data).into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["name"], "Alice");
        assert_eq!(body["count"], 100);
    }

    #[tokio::test]
    async fn test_unit_into_response() {
        let response = ().into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.len(), 0);
    }

    #[tokio::test]
    async fn test_result_ok_into_response() {
        let result: Result<&str, AppError> = Ok("success");
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.as_ref(), b"success");
    }

    #[tokio::test]
    async fn test_result_err_into_response() {
        let result: Result<&str, AppError> = Err(AppError::NotFound);
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["error"], "not found");
    }

    #[tokio::test]
    async fn test_json_nested_structure() {
        #[derive(Serialize)]
        struct User {
            id: u64,
            profile: Profile,
        }

        #[derive(Serialize)]
        struct Profile {
            email: String,
            tags: Vec<String>,
        }

        let user = User {
            id: 1,
            profile: Profile {
                email: "test@example.com".to_string(),
                tags: vec!["rust".to_string(), "web".to_string()],
            },
        };

        let response = Json(user).into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["id"], 1);
        assert_eq!(body["profile"]["email"], "test@example.com");
        assert_eq!(body["profile"]["tags"][0], "rust");
        assert_eq!(body["profile"]["tags"][1], "web");
    }
}
