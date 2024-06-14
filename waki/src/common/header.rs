use crate::{
    bindings::wasi::http::types::{HeaderError, Headers, IncomingRequest, IncomingResponse},
    header::HeaderMap,
};
use anyhow::Result;

macro_rules! impl_header {
    ($($t:ty),+ $(,)?) => ($(
        impl $t {
            pub fn headers_map(&self) -> Result<HeaderMap> {
                let headers_handle = self.headers();
                headers_handle
                    .entries()
                    .into_iter()
                    .map(|(key, value)| Ok((key.try_into()?, value.try_into()?)))
                    .collect::<Result<_, _>>()
            }
        }
    )+)
}

impl_header!(IncomingRequest, IncomingResponse);

impl TryFrom<HeaderMap> for Headers {
    type Error = HeaderError;

    fn try_from(headers: HeaderMap) -> Result<Self, Self::Error> {
        let entries = headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.as_bytes().into()))
            .collect::<Vec<_>>();
        Headers::from_list(&entries)
    }
}
