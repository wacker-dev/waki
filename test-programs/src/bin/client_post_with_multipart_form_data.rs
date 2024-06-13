use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use waki::{
    multipart::{Form, Part},
    Client,
};

#[derive(Deserialize)]
struct Data {
    form: HashMap<String, String>,
    files: HashMap<String, String>,
}

fn main() {
    let resp = Client::new()
        .post("https://httpbin.org/post")
        .multipart(
            Form::new()
                .text("field1", "value1")
                .file("field2", "file.txt")
                .unwrap()
                .part(
                    Part::new("field3", "hello")
                        .filename("file.txt")
                        .mime(mime::TEXT_PLAIN),
                ),
        )
        .connect_timeout(Duration::from_secs(5))
        .send()
        .unwrap();
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.form.get("field1").unwrap(), "value1");
    assert_eq!(data.files.get("field2").unwrap(), "hello\n");
    assert_eq!(data.files.get("field3").unwrap(), "hello");
}
