use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(_: Request) -> Result<Response, ErrorCode> {
    let buffer = [0; 5000];
    Response::builder().body(buffer).build()
}

// required since this file is built as a `bin`
fn main() {}
