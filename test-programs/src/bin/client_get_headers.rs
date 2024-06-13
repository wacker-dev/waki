use serde::Deserialize;
use std::collections::HashMap;
use waki::{header::CONTENT_TYPE, Client};

#[derive(Deserialize)]
struct Data {
    headers: HashMap<String, String>,
}

fn main() {
    let resp = Client::new()
        .get("https://httpbin.org/get")
        .header("Test", "test")
        .headers([("A", "b"), ("C", "d")])
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);
    assert_eq!(
        resp.header(CONTENT_TYPE).unwrap().to_str().unwrap(),
        "application/json"
    );

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.headers.get("Test").unwrap(), "test");
    assert_eq!(data.headers.get("A").unwrap(), "b");
    assert_eq!(data.headers.get("C").unwrap(), "d");
}
