use serde::Deserialize;
use std::time::Duration;
use waki::Client;

#[derive(Deserialize)]
struct Data {
    data: String,
}

fn main() {
    let buffer = [0; 5000];
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .body(buffer)
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.data.len(), 5000);
}
