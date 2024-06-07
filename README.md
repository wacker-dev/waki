# waki

HTTP client and server library for WASI.

Send a request:

```rust
let resp = Client::new()
    .post("https://httpbin.org/post")
    .connect_timeout(Duration::from_secs(5))
    .send()?;

println!("status code: {}", resp.status_code());
```

Writing an HTTP component:

```rust
use waki::{handler, ErrorCode, Request, Response};

#[handler]
fn hello(req: Request) -> Result<Response, ErrorCode> {
    Response::builder().body(b"Hello, WASI!").build()
}
```
