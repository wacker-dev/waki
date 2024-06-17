use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    let form = req.multipart().unwrap();
    let form_part = form.get("form").unwrap();
    let file_part = form.get("file").unwrap();
    Response::builder()
        .body([form_part.value.as_slice(), file_part.value.as_slice()].concat())
        .build()
}

// required since this file is built as a `bin`
fn main() {}
