use crate::bindings::wasi::http::types::{
    Headers, OutgoingBody, OutgoingResponse, ResponseOutparam,
};

use std::collections::HashMap;

pub struct Response {
    headers: HashMap<String, String>,
    status_code: u16,
    body: Option<Vec<u8>>,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl Response {
    pub fn new() -> Self {
        Self {
            body: None,
            headers: HashMap::new(),
            status_code: 200,
        }
    }

    /// Add a header to the response.
    ///
    /// ```
    /// use waki::{handler, Request, Response};
    ///
    /// #[handler]
    /// fn hello(req: Request) -> Response {
    ///     Response::new().header("Content-Type", "application/json")
    /// }
    /// ```
    pub fn header<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a set of headers to the response.
    ///
    /// ```
    /// use waki::{handler, Request, Response};
    ///
    /// #[handler]
    /// fn hello(req: Request) -> Response {
    ///     Response::new().headers([("Content-Type", "application/json"), ("Accept", "*/*")])
    /// }
    /// ```
    pub fn headers<S, I>(mut self, headers: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, S)>,
    {
        self.headers
            .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Set the status code for the response.
    ///
    /// Default value: 200.
    ///
    /// ```
    /// use waki::{handler, Request, Response};
    ///
    /// #[handler]
    /// fn hello(req: Request) -> Response {
    ///     Response::new().status_code(500).body(b"error")
    /// }
    /// ```
    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    /// Set the response body.
    ///
    /// ```
    /// use waki::{handler, Request, Response};
    ///
    /// #[handler]
    /// fn hello(req: Request) -> Response {
    ///     Response::new().body(b"Hello, WASI!")
    /// }
    /// ```
    pub fn body(mut self, data: &[u8]) -> Self {
        self.body = Some(data.into());
        self
    }
}

pub fn handle_response(response_out: ResponseOutparam, response: Response) {
    let entries = response
        .headers
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect::<Vec<_>>();
    let headers = Headers::from_list(&entries).unwrap();

    let outgoing_response = OutgoingResponse::new(headers);
    outgoing_response
        .set_status_code(response.status_code)
        .unwrap();
    let outgoing_body = outgoing_response.body().unwrap();
    ResponseOutparam::set(response_out, Ok(outgoing_response));

    if let Some(body) = response.body {
        let out = outgoing_body.write().unwrap();
        out.blocking_write_and_flush(&body).unwrap();
    }

    OutgoingBody::finish(outgoing_body, None).unwrap();
}
