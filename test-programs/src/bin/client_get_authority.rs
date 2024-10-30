use waki::Client;

fn main() {
    let req = Client::new()
        .get("https://httpbin.org/get?a=b")
        .query(&[("c", "d")])
        .build()
        .unwrap();

    match req.authority() {
        Some(authority) => assert_eq!(authority.as_str(), "httpbin.org"),
        None => assert!(false, "Authority isn't set on client-request"),
    }
}
