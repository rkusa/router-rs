extern crate futures;
extern crate http;
extern crate hyper;
extern crate web;
extern crate web_router;

use futures::Future;
use hyper::StatusCode;
use web::*;
use web_router::{AsParams, Params, Router};

struct State {
    params: Option<Params>,
}

struct Error;

fn main() {
    let mut app = App::new();
    let mut router: Router<State, Error> = Router::new();
    router.get("/foobar", foobar);
    router.get("/foocar", |_, mut res: Response, _| res.body("foocar"));
    router.get("/user/:id", |_, mut res: Response, state: State| {
        let params = state.params().unwrap();
        let id = params.get("id").unwrap();
        res.body(format!("id = {}", id))
    });
    app.add(router);

    let app = app.build();
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = hyper::Server::bind(&addr)
        .serve(move || Ok::<_, ::std::io::Error>(app.serve(|| State::new())));
    println!("Listening on http://{}", server.local_addr());
    hyper::rt::run(server.map_err(|e| {
        eprintln!("Server Error: {}", e);
    }));
}

fn foobar(_: Request, mut res: Response, _: State) -> impl IntoResponse<Error> {
    res.body("foobar")
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

impl From<http::Error> for Error {
    fn from(_: http::Error) -> Self {
        Error
    }
}

impl Into<HttpError> for Error {
    fn into(self) -> HttpError {
        HttpError::Status(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
