extern crate http;

use http::Method;
use std::collections::HashMap;

mod tree;
pub use tree::Params;
use tree::Tree;

pub struct Router<'a, T> {
    routes: HashMap<Method, Tree<'a, T>>,
    // TODO:
    // - HEAD can execute GET
    // - Trailing slash handling
}

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name(&mut self, path: &'a str, handler: T)
        {
            self.route($method, path, handler);
        }
    };
}

impl<'a, T> Router<'a, T> {
    pub fn new() -> Self {
        let mut routes = HashMap::with_capacity(2);
        routes.insert(Method::GET, Tree::new());
        routes.insert(Method::POST, Tree::new());
        Router { routes: routes }
    }

    pub fn route(&mut self, method: Method, path: &'a str, handler: T) {
        if !self.routes.contains_key(&method) {
            let tree = Tree::new();
            self.routes.insert(method.clone(), tree);
        }
        let tree = self.routes.get_mut(&method).unwrap();
        tree.add_path(path, handler);
    }

    method!(options, Method::OPTIONS);
    method!(get, Method::GET);
    method!(post, Method::POST);
    method!(put, Method::PUT);
    method!(delete, Method::DELETE);
    method!(head, Method::HEAD);
    method!(patch, Method::PATCH);

    pub fn resolve(&self, method: &Method, path: &str) -> Option<(&T, Params)> {
        // let path = path.to_lowercase();
        self.routes.get(method).and_then(|tree| tree.find(path))
    }
}
