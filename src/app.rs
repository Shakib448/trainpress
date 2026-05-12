use std::sync::Arc;


use crate::{
    handler::{into_handler, Handler},
    middleware::Middleware,
    router::Router,
    response::IntoResponse,
    Request,
}

pub struct Ap<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) route: Router,
    pub(crate) middlewares: Vec<Arc<dyn Middleware>>,
    pub(crate) state: Arc<S>,
}
