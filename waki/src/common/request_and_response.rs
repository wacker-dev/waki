#[cfg(feature = "multipart")]
use crate::multipart::{parser::parse, Form, Part};
use crate::{
    body::Body,
    header::{AsHeaderName, HeaderMap, HeaderValue, IntoHeaderName, CONTENT_TYPE},
    Request, RequestBuilder, Response, ResponseBuilder,
};
use anyhow::{anyhow, Error, Result};
use serde::Serialize;
use std::collections::HashMap;

macro_rules! impl_common_get_methods {
    ($($t:ty),+ $(,)?) => ($(
        impl $t {
            /// Get the header.
            pub fn header<K: AsHeaderName>(&self, key: K) -> Option<&HeaderValue> {
                self.headers.get(key)
            }

            /// Get headers.
            pub fn headers(&self) -> &HeaderMap {
                &self.headers
            }

            /// Get a chunk of the body.
            ///
            /// It will block until at least one byte can be read or the stream is closed.
            ///
            /// NOTE: This method is only for incoming requests/responses, if you call it on an
            /// outgoing request/response it will always return None.
            pub fn chunk(&self, len: u64) -> Result<Option<Vec<u8>>> {
                self.body.chunk(len)
            }

            /// Get the full body.
            ///
            /// It will block until the stream is closed.
            pub fn body(self) -> Result<Vec<u8>> {
                self.body.bytes()
            }

            /// Deserialize the body as JSON.
            ///
            /// # Optional
            ///
            /// This requires the `json` feature enabled.
            ///
            /// ```
            /// # use anyhow::Result;
            /// # use serde::Deserialize;
            /// # use waki::Response;
            /// # fn run() -> Result<()> {
            /// # let r = Response::new();
            /// #[derive(Deserialize)]
            /// struct Data {
            ///     origin: String,
            ///     url: String,
            /// }
            ///
            /// let json_data = r.json::<Data>()?;
            /// # Ok(())
            /// # }
            /// ```
            #[cfg(feature = "json")]
            pub fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
                Ok(serde_json::from_slice(self.body()?.as_ref())?)
            }

            /// Parse the body as form data.
            pub fn form(self) -> Result<HashMap<String, String>> {
                Ok(serde_urlencoded::from_bytes(self.body()?.as_ref())?)
            }

            /// Parse the body as multipart/form-data.
            ///
            /// # Optional
            ///
            /// This requires the `multipart` feature enabled.
            #[cfg(feature = "multipart")]
            pub fn multipart(self) -> Result<HashMap<String, Part>> {
                match self.headers.get(CONTENT_TYPE) {
                    Some(header) => {
                        let mime = header.to_str()?.parse::<mime::Mime>()?;
                        let boundary = match mime.get_param(mime::BOUNDARY) {
                            Some(v) => v.as_str(),
                            None => {
                                return Err(anyhow!(
                                    "unable to find the boundary value in the Content-Type header"
                                ))
                            }
                        };
                        parse(self.body()?.as_ref(), boundary)
                    }
                    None => Err(anyhow!(
                        "parse body as multipart failed, unable to find the Content-Type header"
                    )),
                }
            }
        }
    )+)
}

impl_common_get_methods!(Request, Response);

macro_rules! impl_common_set_methods {
    ($($t:ty),+ $(,)?) => ($(
        impl $t {
            /// Add a header.
            ///
            /// ```
            /// # use waki::ResponseBuilder;
            /// # fn run() {
            /// # let r = ResponseBuilder::new();
            /// r.header("Content-Type", "application/json");
            /// # }
            /// ```
            pub fn header<K, V>(mut self, key: K, value: V) -> Self
            where
                K: IntoHeaderName,
                V: TryInto<HeaderValue>,
                <V as TryInto<HeaderValue>>::Error: Into<Error>,
            {
                let mut err = None;
                if let Ok(ref mut inner) = self.inner {
                    match value.try_into().map_err(|e| e.into()) {
                        Ok(v) => {
                            inner.headers.insert(key, v);
                        }
                        Err(e) => err = Some(e),
                    };
                }
                if let Some(e) = err {
                    self.inner = Err(e);
                }
                self
            }

            /// Add a set of headers.
            ///
            /// ```
            /// # use waki::ResponseBuilder;
            /// # fn run() {
            /// # let r = ResponseBuilder::new();
            /// r.headers([("Content-Type", "application/json"), ("Accept", "*/*")]);
            /// # }
            /// ```
            pub fn headers<K, V, I>(mut self, headers: I) -> Self
            where
                K: IntoHeaderName,
                V: TryInto<HeaderValue>,
                <V as TryInto<HeaderValue>>::Error: Into<Error>,
                I: IntoIterator<Item = (K, V)>,
            {
                let mut err = None;
                if let Ok(ref mut inner) = self.inner {
                    for (key, value) in headers.into_iter() {
                        match value.try_into().map_err(|e| e.into()) {
                            Ok(v) => {
                                inner.headers.insert(key, v);
                            }
                            Err(e) => {
                                err = Some(e);
                                break;
                            }
                        };
                    }
                }
                if let Some(e) = err {
                    self.inner = Err(e);
                }
                self
            }

            /// Set the body.
            ///
            /// ```
            /// # use waki::ResponseBuilder;
            /// # fn run() {
            /// # let r = ResponseBuilder::new();
            /// r.body("hello");
            /// # }
            /// ```
            pub fn body<V: Into<Vec<u8>>>(mut self, body: V) -> Self {
                if let Ok(ref mut inner) = self.inner {
                    inner.body = Body::Bytes(body.into());
                }
                self
            }

            /// Set a JSON body.
            ///
            /// # Optional
            ///
            /// This requires the `json` feature enabled.
            ///
            /// ```
            /// # use std::collections::HashMap;
            /// # use waki::ResponseBuilder;
            /// # fn run() {
            /// # let r = ResponseBuilder::new();
            /// r.json(&HashMap::from([("data", "hello")]));
            /// # }
            /// ```
            #[cfg(feature = "json")]
            pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
                let mut err = None;
                if let Ok(ref mut inner) = self.inner {
                    inner.headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
                    match serde_json::to_vec(json) {
                        Ok(data) => inner.body = Body::Bytes(data),
                        Err(e) => err = Some(e.into()),
                    }
                }
                if let Some(e) = err {
                    self.inner = Err(e);
                }
                self
            }

            /// Set a form body.
            ///
            /// ```
            /// # use waki::ResponseBuilder;
            /// # fn run() {
            /// # let r = ResponseBuilder::new();
            /// r.form(&[("a", "b"), ("c", "d")]);
            /// # }
            /// ```
            pub fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
                let mut err = None;
                if let Ok(ref mut inner) = self.inner {
                    inner.headers.insert(
                        CONTENT_TYPE,
                        "application/x-www-form-urlencoded".parse().unwrap(),
                    );
                    match serde_urlencoded::to_string(form) {
                        Ok(data) => inner.body = Body::Bytes(data.into()),
                        Err(e) => err = Some(e.into()),
                    }
                }
                if let Some(e) = err {
                    self.inner = Err(e);
                }
                self
            }

            /// Set a multipart/form-data body.
            ///
            /// # Optional
            ///
            /// This requires the `multipart` feature enabled.
            ///
            /// ```
            /// # use anyhow::Result;
            /// # use waki::ResponseBuilder;
            /// # fn run() -> Result<()> {
            /// # let r = ResponseBuilder::new();
            /// let form = waki::multipart::Form::new()
            ///     // Add a text field
            ///     .text("key", "value")
            ///     // And a file
            ///     .file("file", "/path/to/file.txt")?
            ///     // And a custom part
            ///     .part(
            ///         waki::multipart::Part::new("key2", "value2")
            ///             .filename("file.txt")
            ///             .mime_str("text/plain")?,
            ///     );
            ///
            /// r.multipart(form);
            /// # Ok(())
            /// # }
            /// ```
            #[cfg(feature = "multipart")]
            pub fn multipart(mut self, form: Form) -> Self {
                if let Ok(ref mut inner) = self.inner {
                    inner.headers.insert(
                        CONTENT_TYPE,
                        format!("multipart/form-data; boundary={}", form.boundary())
                            .parse()
                            .unwrap(),
                    );
                    inner.body = Body::Bytes(form.build());
                }
                self
            }
        }
    )+)
}

impl_common_set_methods!(RequestBuilder, ResponseBuilder);
