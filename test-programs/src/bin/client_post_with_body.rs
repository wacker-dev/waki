use serde::Deserialize;
use std::time::Duration;
use waki::Client;

#[derive(Deserialize)]
struct Data {
    data: String,
}

fn main() {
    // Make sure the final body is larger than 1024*1024, but we cannot allocate
    // so much memory directly in the wasm program, so we use the `repeat`
    // method to increase the body size.
    const LEN: usize = 1024;
    const REPEAT: usize = 1025;
    let buffer = [0; LEN];
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .body(buffer.repeat(REPEAT))
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.data.len(), LEN * REPEAT);
}
