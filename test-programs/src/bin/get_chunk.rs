use waki::Client;

fn main() {
    let resp = Client::new()
        .get("https://httpbin.org/range/20")
        .query(&[("duration", "5"), ("chunk_size", "10")])
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    while let Some(chunk) = resp.chunk(1024).unwrap() {
        assert_eq!(String::from_utf8(chunk).unwrap().len(), 10);
    }
}
