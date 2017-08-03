extern crate ctx;
extern crate web;
extern crate router;
extern crate hyper;

use ctx::background;
use web::*;
use router::Router;
use hyper::server::Http;

fn main() {
    let mut app = App::new(|| background());
    let mut router = Router::new();
    router.get("/foobar", foobar);
    router.get("/foocar", |_req, mut res, _ctx| {
        res.set_body("foocar");
        Ok(res.into())
    });
    router.get("/user/:id", |_req, mut res, _ctx| {
        res.set_body("/user/:id");
        Ok(res.into())
    });
    app.attach(router);

    let addr = "127.0.0.1:3000".parse().unwrap();
    // let addr = ([127, 0, 0, 1], 3000).into();
    let server = Http::new().bind(&addr, move || Ok(app.clone())).expect("unable to listen");
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().expect("error running server");
}

fn foobar(_req: Request, mut res: Response, _ctx: Context) -> WebResult {
    res.set_body("foobar");
    Ok(res.into())
}