use super::run_wasi_http;

use anyhow::Result;

#[tokio::test(flavor = "multi_thread")]
async fn form() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost")
        .body(body::full("key1=Hello&key2=World"))?;

    let resp = run_wasi_http(test_programs_artifacts::SERVER_FORM_COMPONENT, req).await??;
    let body = resp.into_body().to_bytes();
    let body = std::str::from_utf8(&body)?;
    assert_eq!(body, "Hello World");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn json() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost")
        .body(body::full("{\"data\": \"Hello World\"}"))?;

    let resp = run_wasi_http(test_programs_artifacts::SERVER_JSON_COMPONENT, req).await??;
    let body = resp.into_body().to_bytes();
    let body = std::str::from_utf8(&body)?;
    assert_eq!(body, "Hello World");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn multipart_form() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost")
        .header("Content-Type", "multipart/form-data; boundary=boundary")
        .body(body::full("--boundary\r\nContent-Disposition: form-data; name=form\r\n\r\nHello \r\n--boundary\r\nContent-Disposition: form-data; name=file; filename=file.txt\r\nContent-Type: text/plain\r\n\r\nWorld\r\n--boundary--"))?;

    let resp = run_wasi_http(
        test_programs_artifacts::SERVER_MULTIPART_FORM_COMPONENT,
        req,
    )
    .await??;
    let body = resp.into_body().to_bytes();
    let body = std::str::from_utf8(&body)?;
    assert_eq!(body, "Hello World");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost?name=ia")
        .body(body::empty())?;

    let resp = run_wasi_http(test_programs_artifacts::SERVER_QUERY_COMPONENT, req).await??;
    let body = resp.into_body().to_bytes();
    let body = std::str::from_utf8(&body)?;
    assert_eq!(body, "Hello, ia!");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn status_code() -> Result<()> {
    let req = hyper::Request::builder()
        .uri("http://localhost")
        .body(body::empty())?;

    let resp = run_wasi_http(test_programs_artifacts::SERVER_STATUS_CODE_COMPONENT, req).await??;
    assert_eq!(resp.status(), 400);

    Ok(())
}

mod body {
    use http_body_util::{BodyExt, Full};
    use hyper::body::Bytes;
    use wasmtime_wasi_http::body::HyperIncomingBody;

    pub fn full(bytes: &'static str) -> HyperIncomingBody {
        HyperIncomingBody::new(Full::new(Bytes::from(bytes)).map_err(|_| unreachable!()))
    }

    pub fn empty() -> HyperIncomingBody {
        HyperIncomingBody::default()
    }
}
