use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(_: Request) -> Result<Response, ErrorCode> {
    Response::builder().status_code(400).build()
}

// required since this file is built as a `bin`
fn main() {}
