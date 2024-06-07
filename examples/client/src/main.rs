use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use waki::Client;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Data {
    origin: String,
    url: String,
}

fn main() {
    // get with query
    let resp = Client::new()
        .get("https://httpbin.org/get?a=b")
        .headers([("Content-Type", "application/json"), ("Accept", "*/*")])
        .send()
        .unwrap();
    println!(
        "GET https://httpbin.org/get, status code: {}, body:\n{}",
        resp.status_code(),
        String::from_utf8(resp.body().unwrap()).unwrap()
    );

    // get with json response
    let resp = Client::new().get("https://httpbin.org/get").send().unwrap();
    let status = resp.status_code();
    let json_data = resp.json::<Data>().unwrap();
    println!(
        "GET https://httpbin.org/get, status code: {}, body:\n{:?}\n",
        status, json_data,
    );

    // play with the response chunk
    let resp = Client::new()
        .get("https://httpbin.org/range/20")
        .query(&[("duration", "5"), ("chunk_size", "10")])
        .send()
        .unwrap();
    println!(
        "GET https://httpbin.org/range, status code: {}, body:",
        resp.status_code()
    );
    while let Some(chunk) = resp.chunk(1024).unwrap() {
        println!("{}", String::from_utf8(chunk).unwrap());
    }
    println!();

    // post with json data
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .json(&HashMap::from([("data", "hello")]))
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    println!(
        "POST https://httpbin.org/post, status code: {}, body:\n{}",
        resp.status_code(),
        String::from_utf8(resp.body().unwrap()).unwrap()
    );

    // post with form data
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .form(&[("a", "b"), ("c", "")])
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    println!(
        "POST https://httpbin.org/post, status code: {}, body:\n{}",
        resp.status_code(),
        String::from_utf8(resp.body().unwrap()).unwrap()
    );

    // post with file form data
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .header("Content-Type", "multipart/form-data; boundary=boundary")
        .body(
            "--boundary
Content-Disposition: form-data; name=field1

value1
--boundary
Content-Disposition: form-data; name=field2; filename=file.txt
Content-Type: text/plain

hello
--boundary--"
                .as_bytes(),
        )
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    println!(
        "POST https://httpbin.org/post, status code: {}, body:\n{}",
        resp.status_code(),
        String::from_utf8(resp.body().unwrap()).unwrap()
    );
}
