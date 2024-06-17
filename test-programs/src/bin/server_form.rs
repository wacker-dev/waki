use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    let form = req.form().unwrap();
    Response::builder()
        .body(format!(
            "{} {}",
            form.get("key1").unwrap(),
            form.get("key2").unwrap()
        ))
        .build()
}

// required since this file is built as a `bin`
fn main() {}
