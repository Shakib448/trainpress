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
    let body = req.into_body();
    let bytes = body
        .collect()
        .await
        .map_err(|e| AppError::Body(e.to_string()))?
        .to_bytes();
    Ok(bytes)
}
