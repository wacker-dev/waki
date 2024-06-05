//! # waki
//!
//! An HTTP library for building Web apps with WASI API.
//!
//! ```
//! use waki::{handler, Request, Response};
//!
//! #[handler]
//! fn hello(req: Request) -> Response {
//!     Response::new().body(b"Hello, WASI!")
//! }
//! ```

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
pub use self::{bindings::wasi::http::types::Method, request::Request, response::Response};

/// Export the annotated function as entrypoint of the WASI HTTP component.
///
/// The function needs to have one [`Request`] parameter and one [`Response`] return value.
///
/// For example:
///
/// ```
/// use waki::{handler, Request, Response};
///
/// #[handler]
/// fn hello(req: Request) -> Response {
///     Response::new().body(b"Hello, WASI!")
/// }
/// ```
pub use waki_macros::handler;
