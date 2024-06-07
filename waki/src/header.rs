use crate::bindings::wasi::http::types::{HeaderError, Headers, IncomingRequest, IncomingResponse};

use std::collections::HashMap;

macro_rules! impl_header {
    ($($t:ty),+ $(,)?) => ($(
        impl $t {
            pub fn headers_map(&self) -> HashMap<String, String> {
                let headers_handle = self.headers();
                headers_handle
                    .entries()
                    .into_iter()
                    .map(|(key, value)| (key, String::from_utf8_lossy(&value).to_string()))
                    .collect()
            }
        }
    )+)
}

impl_header!(IncomingRequest, IncomingResponse);

impl TryFrom<HashMap<String, String>> for Headers {
    type Error = HeaderError;

    fn try_from(headers: HashMap<String, String>) -> Result<Self, Self::Error> {
        let entries = headers
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<Vec<_>>();
        Headers::from_list(&entries)
    }
}
