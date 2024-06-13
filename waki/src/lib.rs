#![cfg_attr(docsrs, feature(doc_cfg))]

//! # waki
//!
//! HTTP client and server library for WASI.
//!
//! Send a request:
//!
//! ```
//! # use anyhow::Result;
//! # use std::time::Duration;
//! # use waki::Client;
//! # fn run() -> Result<()> {
//! let resp = Client::new()
//!     .post("https://httpbin.org/post")
//!     .connect_timeout(Duration::from_secs(5))
//!     .send()?;
//!
//! println!("status code: {}", resp.status_code());
//! # Ok(())
//! # }
//! ```
//!
//! Writing an HTTP component:
//!
//! ```
//! use waki::{handler, ErrorCode, Request, Response};
//!
//! #[handler]
//! fn hello(req: Request) -> Result<Response, ErrorCode> {
//!     Response::builder().body(b"Hello, WASI!").build()
//! }
//! ```

mod body;
mod client;
mod header;
#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
pub mod multipart;
mod request;
mod response;

#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "http",
        pub_export_macro: true,
    });
}

#[doc(hidden)]
pub use self::response::handle_response;
pub use self::{
    bindings::wasi::http::types::{ErrorCode, Method},
    client::Client,
    request::{Request, RequestBuilder},
    response::{Response, ResponseBuilder},
};

/// Export the annotated function as entrypoint of the WASI HTTP component.
///
/// The function needs to have one [`Request`] parameter and one Result<[`Response`], [`ErrorCode`]> return value.
///
/// For example:
///
/// ```
/// use waki::{handler, ErrorCode, Request, Response};
///
/// #[handler]
/// fn hello(req: Request) -> Result<Response, ErrorCode> {
///     Response::builder().body(b"Hello, WASI!").build()
/// }
/// ```
pub use waki_macros::handler;

use crate::body::Body;
#[cfg(feature = "multipart")]
use crate::multipart::{parser::parse, Form, Part};
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::collections::HashMap;

macro_rules! impl_common_get_methods {
    ($($t:ty),+ $(,)?) => ($(
        impl $t {
            /// Get the headers.
            pub fn headers(&self) -> &HashMap<String, String> {
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
            #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
            pub fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
                Ok(serde_json::from_slice(self.body()?.as_ref())?)
            }

            /// Parse the body as multipart/form-data.
            ///
            /// # Optional
            ///
            /// This requires the `multipart` feature enabled.
            #[cfg(feature = "multipart")]
            #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
            pub fn multipart(self) -> Result<HashMap<String, Part>> {
                match self.headers.get("Content-Type") {
                    Some(header) => {
                        let mime = header.parse::<mime::Mime>()?;
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
            pub fn header<S: Into<String>>(mut self, key: S, value: S) -> Self {
                if let Ok(ref mut inner) = self.inner {
                    inner.headers.insert(key.into(), value.into());
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
            pub fn headers<S, I>(mut self, headers: I) -> Self
            where
                S: Into<String>,
                I: IntoIterator<Item = (S, S)>,
            {
                if let Ok(ref mut inner) = self.inner {
                    inner.headers
                        .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
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
            #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
            pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
                let mut err = None;
                if let Ok(ref mut inner) = self.inner {
                    inner.headers
                        .insert("Content-Type".into(), "application/json".into());
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
                        "Content-Type".into(),
                        "application/x-www-form-urlencoded".into(),
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
            #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
            pub fn multipart(mut self, form: Form) -> Self {
                if let Ok(ref mut inner) = self.inner {
                    inner.headers.insert(
                        "Content-Type".into(),
                        format!("multipart/form-data; boundary={}", form.boundary()),
                    );
                    inner.body = Body::Bytes(form.build());
                }
                self
            }
        }
    )+)
}

impl_common_set_methods!(RequestBuilder, ResponseBuilder);
