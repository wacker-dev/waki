use anyhow::{Context, Result};
use http_body_util::Collected;
use hyper::body::Bytes;
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, Store,
};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{
    bindings::http::types::ErrorCode, body::HyperIncomingBody, WasiHttpCtx, WasiHttpView,
};

struct Ctx {
    table: ResourceTable,
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for Ctx {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// ref: https://github.com/bytecodealliance/wasmtime/blob/af59c4d568d487b7efbb49d7d31a861e7c3933a6/crates/wasi-http/tests/all/main.rs#L129
pub async fn run_wasi_http(
    component_filename: &str,
    req: hyper::Request<HyperIncomingBody>,
) -> Result<Result<hyper::Response<Collected<Bytes>>, ErrorCode>> {
    let mut config = Config::new();
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.wasm_component_model(true);
    config.async_support(true);

    let engine = Engine::new(&config)?;
    let component = Component::from_file(&engine, component_filename)?;

    let ctx = Ctx {
        table: ResourceTable::new(),
        wasi: WasiCtxBuilder::new().build(),
        http: WasiHttpCtx::new(),
    };

    let mut store = Store::new(&engine, ctx);
    let mut linker = Linker::new(&engine);
    wasmtime_wasi_http::proxy::add_to_linker(&mut linker)?;
    let (proxy, _) =
        wasmtime_wasi_http::proxy::Proxy::instantiate_async(&mut store, &component, &linker)
            .await?;

    let req = store.data_mut().new_incoming_request(req)?;

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let out = store.data_mut().new_response_outparam(sender)?;

    let handle = wasmtime_wasi::runtime::spawn(async move {
        proxy
            .wasi_http_incoming_handler()
            .call_handle(&mut store, req, out)
            .await?;

        Ok::<_, anyhow::Error>(())
    });

    let resp = match receiver.await {
        Ok(Ok(resp)) => {
            use http_body_util::BodyExt;
            let (parts, body) = resp.into_parts();
            let collected = BodyExt::collect(body).await?;
            Some(Ok(hyper::Response::from_parts(parts, collected)))
        }
        Ok(Err(e)) => Some(Err(e)),

        // Fall through below to the `resp.expect(...)` which will hopefully
        // return a more specific error from `handle.await`.
        Err(_) => None,
    };

    // Now that the response has been processed, we can wait on the wasm to
    // finish without deadlocking.
    handle.await.context("Component execution")?;

    Ok(resp.expect("wasm never called set-response-outparam"))
}
