extern crate hyper;
extern crate web;
extern crate ctx;

mod tree;

use std::collections::HashMap;
use hyper::Method;
use hyper::server::{Request, Response};
use tree::Tree;
pub use tree::Params;
use ctx::Context;
use web::{Respond, Middleware};

pub struct Router {
    routes: HashMap<Method, Tree<Middleware>>,
    // TODO:
    // - HEAD can execute GET
    // - Trailing slash handling
}

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name<F>(&mut self, path: &str, handler: F)
            where F: Fn(Request, Response, Context) -> Respond + Send + 'static
        {
            self.route($method, path, handler);
        }
    };
}

impl Router {
    pub fn new() -> Self {
        let mut routes = HashMap::with_capacity(2);
        routes.insert(Method::Get, Tree::new());
        routes.insert(Method::Post, Tree::new());
        Router {
            routes: routes,
        }
    }

    pub fn route<F>(&mut self, method: Method, path: &str, handler: F)
        where F: Fn(Request, Response, Context) -> Respond + Send + 'static
    {
        if !self.routes.contains_key(&method) {
            let tree = Tree::new();
            self.routes.insert(method.clone(), tree);
        }
        let mut tree = self.routes.get_mut(&method).unwrap();
        tree.add_path(path, handler.into());
    }

    method!(options, Method::Options);
    method!(get, Method::Get);
    method!(post, Method::Post);
    method!(put, Method::Put);
    method!(delete, Method::Delete);
    method!(head, Method::Head);
    method!(patch, Method::Patch);

    pub fn resolve(&self, method: &Method, path: &str) -> Option<(&Middleware, Params)> {
        let path = path.to_lowercase();
        self.routes.get(method).and_then(|tree| {
            tree.find(path.as_str())
        })
    }

    pub fn middleware(self) -> Middleware {
        Box::new(move |req, res, ctx| {
            if let Some((handler, params)) = self.resolve(req.method(), req.uri().path()) {
                let ctx = ctx::with_value(ctx, params);
                handler(req, res, ctx)
            } else {
                Respond::Next(req, res, ctx)
            }
        })
    }
}
