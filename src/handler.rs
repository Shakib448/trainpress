use std::sync::Arc;

use crate::{BoxFuture, IntoResponse, Request, Response};

pub type Handler = Arc<dyn Fn(Request) -> BoxFuture<Response> + Send + Sync + 'static>;

pub fn into_handler<F, Fut, R>(f: F) -> Handler
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + 'static,
{
    let f = Arc::new(f);
    Arc::new(move |req| {
        let f = f.clone();
        Box::pin(async move {
            let result = f(req).await;
            result.into_response()
        })
    })
}
