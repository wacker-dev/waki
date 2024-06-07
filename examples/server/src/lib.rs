use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    let query = req.query();
    Response::builder()
        .body(
            format!(
                "Hello, {}!",
                query.get("name").unwrap_or(&"WASI".to_string())
            )
            .as_bytes(),
        )
        .build()
}
