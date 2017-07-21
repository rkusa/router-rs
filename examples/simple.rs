extern crate web;
extern crate router;

use web::*;
use router::Router;

fn main() {
    let mut app = App::new();
    let mut router = Router::new();
    router.get("/foobar", |req, mut res, ctx| {
        res.set_body("foobar");
        Done(res)
    });
    router.get("/foocar", |req, mut res, ctx| {
        res.set_body("foocar");
        Done(res)
    });
    router.get("/user/:id", |req, mut res, ctx| {
        res.set_body("/user/:id");
        Done(res)
    });
    app.attach(router.middleware());

    let addr = "127.0.0.1:3000".parse().unwrap();
    // let addr = ([127, 0, 0, 1], 3000).into();

    let server = app.server(&addr).unwrap();
    println!("Listening on http://{} with 1 thread.",
             server.local_addr().unwrap());
    server.run().unwrap();
}