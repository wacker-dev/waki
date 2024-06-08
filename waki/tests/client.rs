use std::process::Command;

fn wasmtime() -> Command {
    let mut wasmtime = Command::new("wasmtime");
    wasmtime.arg("-S").arg("http");
    wasmtime
}

#[test]
fn client_get_chunk() {
    let status = wasmtime()
        .arg(test_programs_artifacts::CLIENT_GET_CHUNK_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn client_get_with_query() {
    let status = wasmtime()
        .arg(test_programs_artifacts::CLIENT_GET_WITH_QUERY_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn client_post_with_form_data() {
    let status = wasmtime()
        .arg(test_programs_artifacts::CLIENT_POST_WITH_FORM_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn client_post_with_json_data() {
    let status = wasmtime()
        .arg(test_programs_artifacts::CLIENT_POST_WITH_JSON_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn client_post_with_multipart_form_data() {
    let status = wasmtime()
        .arg(test_programs_artifacts::CLIENT_POST_WITH_MULTIPART_FORM_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}
