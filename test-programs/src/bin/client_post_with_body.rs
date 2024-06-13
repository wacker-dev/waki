use serde::Deserialize;
use std::time::Duration;
use waki::Client;

#[derive(Deserialize)]
struct Data {
    data: String,
}

fn main() {
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .body("hello")
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.data, "hello");
}
