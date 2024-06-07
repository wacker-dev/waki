use std::process::Command;

fn wasmtime() -> Command {
    let mut wasmtime = Command::new("wasmtime");
    wasmtime.arg("-S").arg("http");
    wasmtime
}

#[test]
fn get_chunk() {
    let mut cmd = wasmtime();
    let status = cmd
        .arg(test_programs_artifacts::GET_CHUNK_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn get_with_query() {
    let mut cmd = wasmtime();
    let status = cmd
        .arg(test_programs_artifacts::GET_WITH_QUERY_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn post_with_form_data() {
    let mut cmd = wasmtime();
    let status = cmd
        .arg(test_programs_artifacts::POST_WITH_FORM_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn post_with_json_data() {
    let mut cmd = wasmtime();
    let status = cmd
        .arg(test_programs_artifacts::POST_WITH_JSON_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn post_with_multipart_form_data() {
    let mut cmd = wasmtime();
    let status = cmd
        .arg(test_programs_artifacts::POST_WITH_MULTIPART_FORM_DATA_COMPONENT)
        .status()
        .unwrap();
    assert!(status.success());
}
