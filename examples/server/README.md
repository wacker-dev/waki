# http-server

This example is built using the [WASI Preview 2 API](https://github.com/WebAssembly/wasi-http)
with the [waki](https://github.com/wacker-dev/waki) library.

## Build

First, install [cargo component](https://github.com/bytecodealliance/cargo-component):

```
cargo install cargo-component@0.15.0
```

Then execute the following command to compile it into a WASM program:

```
$ cargo component build
```

Or use `--release` option to build it in release mode:

```
$ cargo component build --release
```

## Run

After compilation, you can use [wasmtime](https://github.com/bytecodealliance/wasmtime) to run it:

```
$ wasmtime serve target/wasm32-wasip1/debug/http_server.wasm
Serving HTTP on http://0.0.0.0:8080/
```

```
$ curl http://localhost:8080/
Hello, WASI!

$ curl http://localhost:8080/?name=ia
Hello, ia!
```
