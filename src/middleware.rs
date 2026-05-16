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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_handler() -> Handler {
        Arc::new(|_req: Request| -> BoxFuture<Response> {
            Box::pin(async move {
                use crate::response::IntoResponse;
                "test response".into_response()
            })
        })
    }

    #[test]
    fn test_middleware_trait_exists() {
        // Verify middleware trait compiles
        struct TestMiddleware;

        impl Middleware for TestMiddleware {
            fn call(&self, _req: Request, _next: Next) -> BoxFuture<Response> {
                Box::pin(async move {
                    use crate::response::IntoResponse;
                    hyper::StatusCode::OK.into_response()
                })
            }
        }

        let _mw: Box<dyn Middleware> = Box::new(TestMiddleware);
    }

    #[test]
    fn test_next_structure() {
        let handler = make_test_handler();
        let middlewares: Vec<Arc<dyn Middleware>> = vec![];

        let next = Next {
            middlewares: Arc::new(middlewares),
            handler: handler.clone(),
            index: 0,
        };

        // Verify Next can be created
        assert_eq!(next.index, 0);
    }

    #[test]
    fn test_logger_exists() {
        let _logger = Logger;
        // Logger middleware compiles and can be instantiated
    }

    #[test]
    fn test_request_id_exists() {
        let _request_id = RequestId;
        // RequestId middleware compiles and can be instantiated
    }

    #[test]
    fn test_custom_middleware_implementation() {
        struct CountingMiddleware {
            _count: u32,
        }

        impl Middleware for CountingMiddleware {
            fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
                Box::pin(async move {
                    // Custom middleware logic would go here
                    next.run(req).await
                })
            }
        }

        let mw = CountingMiddleware { _count: 0 };
        let _boxed: Arc<dyn Middleware> = Arc::new(mw);
        // Custom middleware can be created and boxed
    }

    #[test]
    fn test_middleware_vector() {
        struct Mw1;
        struct Mw2;

        impl Middleware for Mw1 {
            fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
                Box::pin(async move { next.run(req).await })
            }
        }

        impl Middleware for Mw2 {
            fn call(&self, req: Request, next: Next) -> BoxFuture<Response> {
                Box::pin(async move { next.run(req).await })
            }
        }

        let middlewares: Vec<Arc<dyn Middleware>> = vec![Arc::new(Mw1), Arc::new(Mw2)];

        assert_eq!(middlewares.len(), 2);
    }
}
