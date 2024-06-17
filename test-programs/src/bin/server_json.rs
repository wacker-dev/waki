use std::collections::HashMap;
use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    let json = req.json::<HashMap<String, String>>().unwrap();
    Response::builder()
        .body(json.get("data").unwrap().to_string())
        .build()
}

// required since this file is built as a `bin`
fn main() {}
