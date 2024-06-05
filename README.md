# waki

An HTTP library for building Web apps with WASI API.

```rust
use waki::{handler, Request, Response};

#[handler]
fn hello(req: Request) -> Response {
    Response::new().body(b"Hello, WASI!")
}
```
