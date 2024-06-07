mod common;

use crate::common::run_wasi_http;

use anyhow::Result;
use wasmtime_wasi_http::body::HyperIncomingBody;

#[tokio::test]
async fn server_hello_query() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost?name=ia")
        .body(HyperIncomingBody::default())?;

    match run_wasi_http(test_programs_artifacts::SERVER_HELLO_COMPONENT, req).await? {
        Ok(resp) => {
            let body = resp.into_body().to_bytes();
            let body = std::str::from_utf8(&body)?;
            assert_eq!(body, "Hello, ia!")
        }
        Err(e) => panic!("Error given in response: {e:?}"),
    };
    Ok(())
}
