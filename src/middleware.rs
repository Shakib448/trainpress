use std::{sync::Arc, time::Instant};

use crate::{BoxFuture, Handler, Request, Response};

pub trait Middleware: Send + Sync + 'static {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response>;
}

pub struct Next {
    pub(crate) middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    pub(crate) handler: Handler,
    pub(crate) index: usize,
}

impl Next {
    pub fn run(self, req: Request) -> BoxFuture<Response> {
        if self.index < self.middlewares.len() {
            let mw = self.middlewares[self.index].clone();
            let next = Next {
                middlewares: self.middlewares.clone(),
                handler: self.handler.clone(),
                index: self.index + 1,
            };
            mw.call(req, next)
        } else {
            (self.handler)(req)
        }
    }
}

pub struct Logger;

impl Middleware for Logger {
    fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            let method = req.method().clone();
            let path = req.uri().path().to_string();
            let start = Instant::now();

            let resp = next.run(req).await;

            let status = resp.status();
            let elapsed_ms = start.elapsed().as_millis() as u64;

            tracing::info!(
                %method,
                path = %path,
                status = status.as_u16(),
                elapsed_ms,
                "request handled"
            );
            resp
        })
    }
}

pub struct RequestId;

impl Middleware for RequestId {
    fn call(&self, mut req: Request, next: Next) -> BoxFuture<Response> {
        Box::pin(async move {
            let id = req
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    format!(
                        "{:x}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos())
                            .unwrap_or(0)
                    )
                });

            if let Ok(value) = id.parse() {
                req.headers_mut().insert("x-request-id", value);
            }

            let mut resp = next.run(req).await;

            if let Ok(value) = id.parse() {
                resp.headers_mut().insert("x-request-id", value);
            }

            resp
        })
    }
}
