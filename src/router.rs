use crate::handler::Handler;
use hyper::Method;
use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::into_handler;
    use crate::Request;

    fn dummy_handler() -> Handler {
        into_handler(|_req: Request| async { "test" })
    }

    #[test]
    fn test_router_add_and_find() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users", handler.clone());

        let matched = router.find(&Method::GET, "/users");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().params.len(), 0);
    }

    #[test]
    fn test_router_path_params() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users/{id}", handler.clone());

        let matched = router.find(&Method::GET, "/users/123").unwrap();
        assert_eq!(matched.params.get("id").unwrap(), "123");
    }

    #[test]
    fn test_router_multiple_params() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users/{user_id}/posts/{post_id}", handler.clone());

        let matched = router.find(&Method::GET, "/users/42/posts/99").unwrap();
        assert_eq!(matched.params.get("user_id").unwrap(), "42");
        assert_eq!(matched.params.get("post_id").unwrap(), "99");
    }

    #[test]
    fn test_router_method_not_found() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users", handler);

        let matched = router.find(&Method::POST, "/users");
        assert!(matched.is_none());
    }

    #[test]
    fn test_router_path_not_found() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users", handler);

        let matched = router.find(&Method::GET, "/posts");
        assert!(matched.is_none());
    }

    #[test]
    fn test_router_multiple_methods_same_path() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users", handler.clone());
        router.add(Method::POST, "/users", handler.clone());
        router.add(Method::DELETE, "/users", handler.clone());

        assert!(router.find(&Method::GET, "/users").is_some());
        assert!(router.find(&Method::POST, "/users").is_some());
        assert!(router.find(&Method::DELETE, "/users").is_some());
        assert!(router.find(&Method::PUT, "/users").is_none());
    }

    #[test]
    #[should_panic(expected = "route conflict")]
    fn test_router_duplicate_route_panics() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/users", handler.clone());
        router.add(Method::GET, "/users", handler.clone()); // Should panic
    }

    #[test]
    fn test_router_wildcard_patterns() {
        let mut router = Router::new();
        let handler = dummy_handler();
        router.add(Method::GET, "/files/{*path}", handler);

        let matched = router.find(&Method::GET, "/files/a/b/c.txt").unwrap();
        assert_eq!(matched.params.get("path").unwrap(), "a/b/c.txt");
    }
}
