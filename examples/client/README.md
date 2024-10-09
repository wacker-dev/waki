# http-client

This example is built using the [WASI Preview 2 API](https://github.com/WebAssembly/wasi-http)
with the [waki](https://github.com/wacker-dev/waki) library.

## Build

Requires Rust 1.82+.

```
$ rustup target add wasm32-wasip2
```

Use the following command to compile it into a WASM program:

```
$ cargo build
```

Or use `--release` option to build it in release mode:

```
$ cargo build --release
```

## Run

After compilation, you can use [wasmtime](https://github.com/bytecodealliance/wasmtime) to run it:

```
$ wasmtime -S http target/wasm32-wasip1/debug/http-client.wasm

GET https://httpbin.org/get, status code: 200, body:
{
  "args": {
    "a": "b"
  },
  "headers": {
    "Accept": "*/*",
    "Content-Type": "application/json",
    "Host": "httpbin.org",
    "X-Amzn-Trace-Id": "..."
  },
  "origin": "...",
  "url": "https://httpbin.org/get?a=b"
}

GET https://httpbin.org/get, status code: 200, body:
Data { origin: "117.172.222.76", url: "https://httpbin.org/get" }

GET https://httpbin.org/range, status code: 200, body:
abcdefghij
klmnopqrst

POST https://httpbin.org/post, status code: 200, body:
{
  "args": {},
  "data": "{\"data\":\"hello\"}",
  "files": {},
  "form": {},
  "headers": {
    "Content-Length": "16",
    "Content-Type": "application/json",
    "Host": "httpbin.org",
    "X-Amzn-Trace-Id": "..."
  },
  "json": {
    "data": "hello"
  },
  "origin": "...",
  "url": "https://httpbin.org/post"
}

POST https://httpbin.org/post, status code: 200, body:
{
  "args": {},
  "data": "",
  "files": {},
  "form": {
    "a": "b",
    "c": ""
  },
  "headers": {
    "Content-Length": "6",
    "Content-Type": "application/x-www-form-urlencoded",
    "Host": "httpbin.org",
    "X-Amzn-Trace-Id": "..."
  },
  "json": null,
  "origin": "...",
  "url": "https://httpbin.org/post"
}

POST https://httpbin.org/post, status code: 200, body:
{
  "args": {},
  "data": "",
  "files": {
    "field2": "hello"
  },
  "form": {
    "field1": "value1"
  },
  "headers": {
    "Content-Length": "181",
    "Content-Type": "multipart/form-data; boundary=boundary",
    "Host": "httpbin.org",
    "X-Amzn-Trace-Id": "..."
  },
  "json": null,
  "origin": "...",
  "url": "https://httpbin.org/post"
}
```
