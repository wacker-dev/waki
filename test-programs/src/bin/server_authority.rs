use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    let authority = req.authority();

    match authority {
        Some(authority) => Response::builder()
            .body(format!("Hello, {}!", authority.as_str()))
            .build(),
        None => Response::builder().body("Hello!").build(),
    }
}

// required since this file is built as a `bin`
fn main() {}
