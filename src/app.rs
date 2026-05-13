use std::sync::Arc;

use crate::{
    extract::StateExt,
    handler::{into_handler, Handler},
    middleware::Middleware,
    response::IntoResponse,
    router::Router,
    Request,
};

use hyper::Method;

pub struct App<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    pub(crate) router: Router,
    pub(crate) middlewares: Vec<Arc<dyn Middleware>>,
    pub(crate) state: Arc<S>,
}

impl App<()> {
    pub fn new_stateless() -> Self {
        Self {
            router: Router::new(),
            middlewares: Vec::new(),
            state: Arc::new(()),
        }
    }
}

impl<S> App<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Self {
            router: Router::new(),
            middlewares: Vec::new(),
            state: Arc::new(state),
        }
    }

    pub fn middleware<M: Middleware>(mut self, mw: M) -> Self {
        self.middlewares.push(Arc::new(mw));
        self
    }

    pub fn route<F, Fut, R>(mut self, method: Method, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        let handler: Handler = into_handler(f);
        self.router.add(method, path, handler);
        self
    }

    pub fn get<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::GET, path, f)
    }

    pub fn post<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::POST, path, f)
    }

    pub fn put<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::PUT, path, f)
    }

    pub fn patch<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::PATCH, path, f)
    }

    pub fn delete<F, Fut, R>(self, path: &str, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.route(Method::DELETE, path, f)
    }

    pub(crate) fn state_ext(&self) -> StateExt<S> {
        StateExt(self.state.clone())
    }

    pub async fn listen(self, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        crate::server::serve(self, addr).await
    }
}
