use serde::Deserialize;
use std::collections::HashMap;
use waki::Client;

#[derive(Deserialize)]
struct Data {
    args: HashMap<String, String>,
}

fn main() {
    let resp = Client::new()
        .get("https://httpbin.org/get?a=b")
        .query(&[("c", "d")])
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.args.get("a").unwrap(), "b");
    assert_eq!(data.args.get("c").unwrap(), "d");
}
