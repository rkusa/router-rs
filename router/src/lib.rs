extern crate hyper;

use std::collections::HashMap;
use hyper::Method;

mod tree;
use tree::Tree;
pub use tree::Params;

pub struct Router<T> {
    routes: HashMap<Method, Tree<T>>,
    // TODO:
    // - HEAD can execute GET
    // - Trailing slash handling
}

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name(&mut self, path: &str, handler: T)
        {
            self.route($method, path, handler);
        }
    };
}

impl<T> Router<T> {
    pub fn new() -> Self {
        let mut routes = HashMap::with_capacity(2);
        routes.insert(Method::Get, Tree::new());
        routes.insert(Method::Post, Tree::new());
        Router { routes: routes }
    }

    pub fn route(&mut self, method: Method, path: &str, handler: T) {
        if !self.routes.contains_key(&method) {
            let tree = Tree::new();
            self.routes.insert(method.clone(), tree);
        }
        let tree = self.routes.get_mut(&method).unwrap();
        tree.add_path(path, handler);
    }

    method!(options, Method::Options);
    method!(get, Method::Get);
    method!(post, Method::Post);
    method!(put, Method::Put);
    method!(delete, Method::Delete);
    method!(head, Method::Head);
    method!(patch, Method::Patch);

    pub fn resolve(&self, method: &Method, path: &str) -> Option<(&T, Params)> {
        let path = path.to_lowercase();
        self.routes.get(method).and_then(
            |tree| tree.find(path.as_str()),
        )
    }
}
