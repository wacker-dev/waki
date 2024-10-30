use super::run_wasi;

#[tokio::test(flavor = "multi_thread")]
async fn get_authority() {
    run_wasi(test_programs_artifacts::CLIENT_GET_AUTHORITY_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn get_chunk() {
    run_wasi(test_programs_artifacts::CLIENT_GET_CHUNK_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn get_headers() {
    run_wasi(test_programs_artifacts::CLIENT_GET_HEADERS_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn get_with_query() {
    run_wasi(test_programs_artifacts::CLIENT_GET_WITH_QUERY_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn post_with_body() {
    run_wasi(test_programs_artifacts::CLIENT_POST_WITH_BODY_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn post_with_form_data() {
    run_wasi(test_programs_artifacts::CLIENT_POST_WITH_FORM_DATA_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn post_with_json_data() {
    run_wasi(test_programs_artifacts::CLIENT_POST_WITH_JSON_DATA_COMPONENT)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn post_with_multipart_form_data() {
    run_wasi(test_programs_artifacts::CLIENT_POST_WITH_MULTIPART_FORM_DATA_COMPONENT)
        .await
        .unwrap();
}
