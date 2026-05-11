use std::collections::HashMap;

use hyper::Method;

use crate::handler::Handler;

pub struct Matched {
    pub handler: Handler,
    pub params: HashMap<String, String>,
}

pub struct Router {
    inner: HashMap<Method, matchit::Router<Handler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    pub fn add(&mut self, method: Method, path: &str, handler: Handler) {
        let trie = self
            .inner
            .entry(method.clone())
            .or_insert_with(matchit::Router::new);

        trie.insert(path, handler)
            .unwrap_or_else(|e| panic!("route conflict on {} {}: {}", method, path, e));
    }

    pub fn find(&self, method: &Method, path: &str) -> Option<Matched> {
        let trie = self.inner.get(method)?;
        let m = trie.at(path).ok()?;

        let params = m
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Some(Matched {
            handler: m.value.clone(),
            params,
        })
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
