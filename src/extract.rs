use http_body_util::BodyExt;
use hyper::body::Bytes;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use serde::de::DeserializeOwned;

use crate::error::AppError;
use crate::Request;

#[derive(Clone, Debug)]
pub(crate) struct PathParams(pub HashMap<String, String>);

pub(crate) struct StateExt<S>(pub Arc<S>);

impl<S> Clone for StateExt<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait RequestExt {
    fn path_param<T>(&self, name: &str) -> Result<T, AppError>
    where
        T: FromStr,
        T::Err: std::fmt::Display;

    fn query<T: DeserializeOwned>(&self) -> Result<T, AppError>;

    fn state<S: Clone + Send + Sync + 'static>(&self) -> Result<S, AppError>;
}

impl RequestExt for Request {
    fn path_param<T>(&self, name: &str) -> Result<T, AppError>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let params = self
            .extensions()
            .get::<PathParams>()
            .ok_or_else(|| AppError::PathParam("no path params on this request".into()))?;

        let raw = params
            .0
            .get(name)
            .ok_or_else(|| AppError::PathParam(format!("param '{}' not found", name)))?;
        raw.parse::<T>()
            .map_err(|e| AppError::PathParam(format!("param '{}': {}", name, e)))
    }

    fn query<T: DeserializeOwned>(&self) -> Result<T, AppError> {
        let q = self.uri().query().unwrap_or("");
        Ok(serde_urlencoded::from_str(q)?)
    }

    fn state<S: Clone + Send + Sync + 'static>(&self) -> Result<S, AppError> {
        let ext = self
            .extensions()
            .get::<StateExt<S>>()
            .ok_or(AppError::MissingState)?;

        Ok((*ext.0).clone())
    }
}

pub async fn json_body<T: DeserializeOwned>(req: Request) -> Result<T, AppError> {
    json_body_generic(req).await
}

pub async fn json_body_generic<T, B>(req: hyper::Request<B>) -> Result<T, AppError>
where
    T: DeserializeOwned,
    B: hyper::body::Body + Send + 'static,
    B::Error: std::fmt::Display,
{
    let body = req.into_body();

    let bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Body(e.to_string()))?
        .to_bytes();

    let value: T = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("invalid json body: {}", e)))?;

    Ok(value)
}

pub async fn body_bytes(req: Request) -> Result<Bytes, AppError> {
    body_bytes_generic(req).await
}

pub async fn body_bytes_generic<B>(req: hyper::Request<B>) -> Result<Bytes, AppError>
where
    B: hyper::body::Body + Send + 'static,
    B::Error: std::fmt::Display,
{
    let body = req.into_body();
    let bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Body(e.to_string()))?
        .to_bytes();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::Full;
    use hyper::body::Bytes;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestQuery {
        name: String,
        age: u32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestBody {
        title: String,
        count: i32,
    }

    #[derive(Clone, Debug)]
    struct TestState {
        value: String,
    }

    // Helper to create test request - note this is a different type than production Request
    type TestRequest = hyper::Request<http_body_util::combinators::BoxBody<Bytes, std::convert::Infallible>>;

    impl RequestExt for TestRequest {
        fn path_param<T>(&self, name: &str) -> Result<T, AppError>
        where
            T: FromStr,
            T::Err: std::fmt::Display,
        {
            let params = self
                .extensions()
                .get::<PathParams>()
                .ok_or_else(|| AppError::PathParam("no path params on this request".into()))?;

            let raw = params
                .0
                .get(name)
                .ok_or_else(|| AppError::PathParam(format!("param '{}' not found", name)))?;
            raw.parse::<T>()
                .map_err(|e| AppError::PathParam(format!("param '{}': {}", name, e)))
        }

        fn query<T: DeserializeOwned>(&self) -> Result<T, AppError> {
            let q = self.uri().query().unwrap_or("");
            Ok(serde_urlencoded::from_str(q)?)
        }

        fn state<S: Clone + Send + Sync + 'static>(&self) -> Result<S, AppError> {
            let ext = self
                .extensions()
                .get::<StateExt<S>>()
                .ok_or(AppError::MissingState)?;

            Ok((*ext.0).clone())
        }
    }

    #[test]
    fn test_path_param_extraction() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("name".to_string(), "alice".to_string());

        let mut req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        req.extensions_mut().insert(PathParams(params));

        let id: u64 = req.path_param("id").unwrap();
        assert_eq!(id, 123);

        let name: String = req.path_param("name").unwrap();
        assert_eq!(name, "alice");
    }

    #[test]
    fn test_path_param_missing() {
        let params = HashMap::new();
        let mut req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        req.extensions_mut().insert(PathParams(params));

        let result: Result<u64, _> = req.path_param("id");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PathParam(_)));
    }

    #[test]
    fn test_path_param_parse_error() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "not-a-number".to_string());

        let mut req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        req.extensions_mut().insert(PathParams(params));

        let result: Result<u64, _> = req.path_param("id");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::PathParam(_)));
    }

    #[test]
    fn test_query_extraction() {
        let req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test?name=bob&age=30")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let query: TestQuery = req.query().unwrap();
        assert_eq!(
            query,
            TestQuery {
                name: "bob".to_string(),
                age: 30
            }
        );
    }

    #[test]
    fn test_query_empty() {
        let req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        #[derive(Deserialize)]
        struct EmptyQuery {}

        let _query: EmptyQuery = req.query().unwrap();
        // Should succeed with empty query
    }

    #[test]
    fn test_query_parse_error() {
        let req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test?age=invalid")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let result: Result<TestQuery, _> = req.query();
        assert!(result.is_err());
    }

    #[test]
    fn test_state_extraction() {
        let state = TestState {
            value: "test-state".to_string(),
        };
        let state_ext = StateExt(Arc::new(state.clone()));

        let mut req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        req.extensions_mut().insert(state_ext);

        let extracted: TestState = req.state().unwrap();
        assert_eq!(extracted.value, "test-state");
    }

    #[test]
    fn test_state_missing() {
        let req: TestRequest = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::new()).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let result: Result<TestState, _> = req.state();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::MissingState));
    }

    #[tokio::test]
    async fn test_json_body_extraction() {
        let json_str = r#"{"title":"Test","count":42}"#;
        let req = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::from(json_str)).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let body: TestBody = json_body_generic(req).await.unwrap();
        assert_eq!(
            body,
            TestBody {
                title: "Test".to_string(),
                count: 42
            }
        );
    }

    #[tokio::test]
    async fn test_json_body_invalid() {
        let json_str = r#"{"invalid json"#;
        let req = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::from(json_str)).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let result: Result<TestBody, _> = json_body_generic(req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn test_body_bytes_extraction() {
        let data = b"raw bytes data";
        let req = hyper::Request::builder()
            .uri("http://example.com/test")
            .body(Full::new(Bytes::from(&data[..])).map_err(|_| unreachable!()).boxed())
            .unwrap();

        let bytes = body_bytes_generic(req).await.unwrap();
        assert_eq!(bytes.as_ref(), data);
    }
}
