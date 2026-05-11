use std::sync::Arc;

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
