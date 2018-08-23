extern crate futures;
extern crate http;
extern crate router;
extern crate web;

use http::Method;
pub use router::Params;
use web::{IntoResponse, Middleware, Next, Request, Response, ResponseFuture};

pub trait Handler<S>: Send + Sync {
    fn handle(&self, Request, Response, S) -> ResponseFuture;
}

impl<S, F, B> Handler<S> for F
where
    F: Send + Sync + Fn(Request, Response, S) -> B,
    B: IntoResponse,
{
    fn handle(&self, req: Request, res: Response, state: S) -> ResponseFuture {
        let fut = (self)(req, res, state).into_response();
        Box::new(fut)
    }
}

pub struct Router<'a, S>(router::Router<'a, Box<Handler<S>>>);

macro_rules! method {
    ( $name:ident, $method:expr ) => {
        pub fn $name<H>(&mut self, path: &'a str, handler: H)
    where
        H: Handler<S> + 'static,
        {
            self.route($method, path, handler);
        }
    };
}

impl<'a, S> Router<'a, S> {
    pub fn new() -> Self {
        Router::default()
    }

    pub fn route<H>(&mut self, method: Method, path: &'a str, handler: H)
    where
        H: Handler<S> + 'static,
    {
        self.0.route(method, path, Box::new(handler));
    }

    method!(options, Method::OPTIONS);
    method!(get, Method::GET);
    method!(post, Method::POST);
    method!(put, Method::PUT);
    method!(delete, Method::DELETE);
    method!(head, Method::HEAD);
    method!(patch, Method::PATCH);
}

impl<'a, S> Default for Router<'a, S> {
    fn default() -> Self {
        Router(router::Router::default())
    }
}

pub trait AsParams {
    fn with_params(self, Params) -> Self;
    fn params(&self) -> Option<&Params>;
}

impl<'a, S> Middleware<S> for Router<'a, S>
where
    S: AsParams,
{
    fn handle(&self, req: Request, res: Response, state: S, next: Next<S>) -> ResponseFuture {
        if let Some((mw, params)) = self.0.resolve(req.method(), req.uri().path()) {
            let state = state.with_params(params);
            let fut = mw.handle(req, res, state);
            Box::new(fut)
        } else {
            next(req, res, state)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate futures;
    extern crate hyper;

    use self::futures::{Future, Stream};
    use self::hyper::Body;
    use http::{self, StatusCode};
    use web::{App, HttpError, IntoResponse, Response};
    use {AsParams, Params, Router};

    struct State {
        params: Option<Params>,
    }

    impl State {
        fn new() -> Self {
            State { params: None }
        }
    }

    impl AsParams for State {
        fn with_params(mut self, params: Params) -> Self {
            self.params = Some(params);
            self
        }

        fn params(&self) -> Option<&Params> {
            self.params.as_ref()
        }
    }

    #[test]
    fn middleware() {
        let mut router: Router<State> = Router::new();
        router.get("/foo", |_, mut res: Response, _| res.body("Hello World!"));

        let mut app = App::new();
        app.add(router);

        let req = http::Request::get("http://localhost/foo")
            .body(Body::empty())
            .unwrap();
        let res = app
            .build()
            .execute(req, Response::new(), State::new(), |_, _, _| {
                let mut res = Response::new();
                res.status(StatusCode::NOT_FOUND);
                Ok::<_, HttpError>(res).into_response()
            })
            .wait()
            .unwrap();
        let body = String::from_utf8(res.into_body().concat2().wait().unwrap().to_vec()).unwrap();
        assert_eq!(body, "Hello World!");
    }

    #[test]
    fn param_case_sensitivity() {
        let mut router: Router<State> = Router::new();
        router.get("/test/:name", |_, mut res: Response, state: State| {
            let params = state.params().unwrap();
            res.body(params.get("name").unwrap().clone())
        });

        let mut app = App::new();
        app.add(router);

        let req = http::Request::get("http://localhost/test/FooBar")
            .body(Body::empty())
            .unwrap();
        let res = app
            .build()
            .execute(req, Response::default(), State::new(), |_, _, _| {
                let mut res = Response::new();
                res.status(StatusCode::NOT_FOUND);
                Ok::<_, HttpError>(res).into_response()
            })
            .wait()
            .unwrap();
        let body = String::from_utf8(res.into_body().concat2().wait().unwrap().to_vec()).unwrap();
        assert_eq!(body, "FooBar");
    }
}
