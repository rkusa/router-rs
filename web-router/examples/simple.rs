extern crate ctx;
extern crate hyper;
extern crate web;
extern crate web_router;

use ctx::background;
use web::*;
use web_router::{Router, RouterFuture};
use hyper::server::Http;

fn main() {
    let mut app = App::new(|| background());
    let mut router = Router::new();
    router.get("/foobar", foobar);
    router.get("/foocar", |_, mut res: Response, _| {
        res.set_body("foocar");
        Ok(res)
    });
    router.get("/user/:id", |_, mut res: Response, _| {
        res.set_body("/user/:id");
        Ok(res)
    });
    app.add(router);

    let app = app.build();
    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Http::new()
        .bind(&addr, move || Ok(app.clone()))
        .expect("unable to listen");
    println!(
        "Listening on http://{} with 1 thread.",
        server.local_addr().unwrap()
    );
    server.run().expect("error running server");
}

fn foobar(_: Request, mut res: Response, _: Context) -> RouterFuture<HttpError> {
    res.set_body("foobar");
    done(res.into())
}
