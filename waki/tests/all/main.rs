mod client;
mod server;

use anyhow::{Context, Result};
use http_body_util::Collected;
use hyper::body::Bytes;
use wasmtime::{
    component::{Component, Linker, ResourceTable},
    Config, Engine, Store,
};
use wasmtime_wasi::{bindings::Command, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{
    bindings::http::types::ErrorCode, body::HyperIncomingBody, proxy::Proxy, WasiHttpCtx,
    WasiHttpView,
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

fn new_component(component_filename: &str) -> Result<(Store<Ctx>, Component, Linker<Ctx>)> {
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

    let store = Store::new(&engine, ctx);
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::proxy::add_only_http_to_linker(&mut linker)?;
    Ok((store, component, linker))
}

// ref: https://github.com/bytecodealliance/wasmtime/blob/af59c4d568d487b7efbb49d7d31a861e7c3933a6/crates/wasi-http/tests/all/main.rs#L129
pub async fn run_wasi_http(
    component_filename: &str,
    req: hyper::Request<HyperIncomingBody>,
) -> Result<Result<hyper::Response<Collected<Bytes>>, ErrorCode>> {
    let (mut store, component, linker) = new_component(component_filename)?;

    let (proxy, _) = Proxy::instantiate_async(&mut store, &component, &linker).await?;

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

pub async fn run_wasi(component_filename: &str) -> Result<()> {
    let (mut store, component, linker) = new_component(component_filename)?;

    let (command, _) = Command::instantiate_async(&mut store, &component, &linker).await?;
    command
        .wasi_cli_run()
        .call_run(&mut store)
        .await?
        .map_err(|()| anyhow::anyhow!("run returned a failure"))
}
