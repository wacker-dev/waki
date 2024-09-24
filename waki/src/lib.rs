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

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod body;
mod client;
mod common;
#[cfg(feature = "multipart")]
pub mod multipart;
mod request;
mod response;

#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "http",
        pub_export_macro: true,
        generate_all,
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

pub use http::header;
