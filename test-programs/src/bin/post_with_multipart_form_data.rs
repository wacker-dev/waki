use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use waki::Client;

#[derive(Deserialize)]
struct Data {
    form: HashMap<String, String>,
    files: HashMap<String, String>,
}

fn main() {
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
    assert_eq!(resp.status_code(), 200);

    let data = resp.json::<Data>().unwrap();
    assert_eq!(data.form.get("field1").unwrap(), "value1");
    assert_eq!(data.files.get("field2").unwrap(), "hello");
}
