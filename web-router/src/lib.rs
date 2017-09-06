extern crate ctx;
extern crate hyper;
extern crate web;
extern crate router;

use hyper::{Request, Response, Method};
use web::{Next, Middleware, WebFuture, Context};

pub struct Router(router::Router<Box<Middleware>>);

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name<M>(&mut self, path: &str, handler: M)
        where
        M: Middleware + 'static,
        {
            self.route($method, path, handler);
        }
    };
}

impl Router {
    pub fn new() -> Self {
        Router(router::Router::new())
    }

    pub fn route<M>(&mut self, method: Method, path: &str, handler: M)
    where
        M: Middleware + 'static,
    {
        self.0.route(method, path, Box::new(handler));
    }

    method!(options, Method::Options);
    method!(get, Method::Get);
    method!(post, Method::Post);
    method!(put, Method::Put);
    method!(delete, Method::Delete);
    method!(head, Method::Head);
    method!(patch, Method::Patch);
}

impl Middleware for Router {
    fn handle(&self, req: Request, res: Response, ctx: Context, next: Next) -> WebFuture {
        if let Some((mw, params)) = self.0.resolve(req.method(), req.uri().path()) {
            let ctx = ctx::with_value(ctx, params);
            mw.handle(req, res, ctx, next)
        } else {
            next(req, res, ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate futures;

    use ctx::background;
    use hyper::{Method, Uri};
    use hyper::{Request, Response, StatusCode};
    use Router;
    use self::futures::{Future, Stream};
    use std::str::FromStr;
    use web::{App, Next, done};

    #[test]
    fn middleware() {
        let mut router = Router::new();
        router.get("/foo", |_, mut res: Response, _, _| {
            res.set_body("Hello World!");
            done(res)
        });

        let mut app = App::new(|| background());
        app.add(router);

        let req = Request::new(Method::Get, Uri::from_str("http://localhost/foo").unwrap());
        let res = app.build()
            .execute(req, Response::default(), background(), |_, _, _| {
                done(Response::default().with_status(StatusCode::NotFound))
            })
            .wait()
            .unwrap();
        let body = String::from_utf8(res.body().concat2().wait().unwrap().to_vec()).unwrap();
        assert_eq!(body, "Hello World!");
    }
}
