extern crate ctx;
extern crate futures;
extern crate hyper;
extern crate router;
extern crate web;

use hyper::{Method, Request, Response};
use web::{Context, HttpError, IntoResponse, Middleware, Next, WebFuture};
use futures::{Future, IntoFuture};
pub use router::Params;

pub type RouterFuture<E> = Box<Future<Item = Response, Error = E>>;

pub trait Handler<S, E>: Send + Sync {
    fn handle(&self, Request, Response, S) -> RouterFuture<E>;
}

impl<S, E, F, B> Handler<S, E> for F
where
    E: 'static,
    F: Send + Sync + Fn(Request, Response, S) -> B,
    B: IntoRouterFuture<E> + 'static,
{
    fn handle(&self, req: Request, res: Response, ctx: S) -> RouterFuture<E> {
        Box::new((self)(req, res, ctx).into_future())
    }
}

pub trait IntoRouterFuture<E> {
    fn into_future(self) -> RouterFuture<E>;
}

impl<E, F, I> IntoRouterFuture<E> for F
where
    F: IntoFuture<Item = I, Error = E>,
    I: IntoResponse,
    <F as futures::IntoFuture>::Future: 'static,
{
    fn into_future(self) -> RouterFuture<E> {
        Box::new(self.into_future().map(|i| i.into_response()))
    }
}

pub struct Router<'a, S, E: Into<HttpError>>(router::Router<'a, Box<Handler<S, E>>>);

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name<H>(&mut self, path: &'a str, handler: H)
    where
        H: Handler<S, E> + 'static,
        {
            self.route($method, path, handler);
        }
    };
}

impl<'a, S, E> Router<'a, S, E>
where
    E: Into<HttpError>,
{
    pub fn new() -> Self {
        Router(router::Router::new())
    }

    pub fn route<H>(&mut self, method: Method, path: &'a str, handler: H)
    where
        H: Handler<S, E> + 'static,
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

pub trait AsParams {
    fn with_params(self, Params) -> Self;
    fn params(&self) -> Option<&Params>;
}

impl AsParams for Context {
    fn with_params(self, params: Params) -> Self {
        ctx::with_value(self, params)
    }

    fn params(&self) -> Option<&Params> {
        self.value_ref::<Params>()
    }
}

impl<'a, S, E> Middleware<S> for Router<'a, S, E>
where
    S: AsParams,
    E: Into<HttpError> + 'static,
{
    fn handle(&self, req: Request, res: Response, ctx: S, next: Next<S>) -> WebFuture {
        if let Some((mw, params)) = self.0.resolve(req.method(), req.uri().path()) {
            let ctx = ctx.with_params(params);
            Box::new(mw.handle(req, res, ctx).map_err(|err| err.into()))
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
    use {Router, Params};
    use self::futures::{Future, Stream};
    use std::str::FromStr;
    use web::{done, App, Context};

    #[test]
    fn middleware() {
        let mut router = Router::new();
        router.get("/foo", |_, mut res: Response, _| {
            res.set_body("Hello World!");
            done(res)
        });

        let mut app = App::new();
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

    #[test]
    fn param_case_sensitivity() {
        let mut router = Router::new();
        router.get("/test/:name", |_, mut res: Response, ctx: Context| {
            let params = ctx.value::<Params>().unwrap();
            res.set_body(params.get("name").unwrap().clone());
            done(res)
        });

        let mut app = App::new();
        app.add(router);

        let req = Request::new(Method::Get, Uri::from_str("http://localhost/test/FooBar").unwrap());
        let res = app.build()
            .execute(req, Response::default(), background(), |_, _, _| {
                done(Response::default().with_status(StatusCode::NotFound))
            })
            .wait()
            .unwrap();
        let body = String::from_utf8(res.body().concat2().wait().unwrap().to_vec()).unwrap();
        assert_eq!(body, "FooBar");
    }
}
