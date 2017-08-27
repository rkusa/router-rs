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
use web::{Next, WebResult, Middleware, FnMiddleware};

pub struct Router {
    routes: HashMap<Method, Tree<Box<Middleware>>>,
    // TODO:
    // - HEAD can execute GET
    // - Trailing slash handling
}

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name<F>(&mut self, path: &str, handler: F)
            where F: 'static + Fn(Request, Response, Context) -> WebResult + Send + Sync
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
        where F: 'static + Fn(Request, Response, Context) -> WebResult + Send + Sync,
    {
        if !self.routes.contains_key(&method) {
            let tree = Tree::new();
            self.routes.insert(method.clone(), tree);
        }
        let tree = self.routes.get_mut(&method).unwrap();
        tree.add_path(path, Box::new(FnMiddleware::new(handler)));
    }

    method!(options, Method::Options);
    method!(get, Method::Get);
    method!(post, Method::Post);
    method!(put, Method::Put);
    method!(delete, Method::Delete);
    method!(head, Method::Head);
    method!(patch, Method::Patch);

    pub fn resolve(&self, method: &Method, path: &str) -> Option<(&Box<Middleware>, Params)> {
        let path = path.to_lowercase();
        self.routes.get(method).and_then(|tree| {
            tree.find(path.as_str())
        })
    }
}

impl Middleware for Router {
    fn handle(&self, req: Request, res: Response, ctx: Context) -> WebResult {
        if let Some((mw, params)) = self.resolve(req.method(), req.uri().path()) {
            let ctx = ctx::with_value(ctx, params);
            mw.handle(req, res, ctx)
        } else {
            Ok(Next(req, res, ctx))
        }
    }
}
