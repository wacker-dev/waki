use crate::{
    bindings::wasi::http::types::{
        IncomingResponse, OutgoingBody, OutgoingResponse, ResponseOutparam,
    },
    body::Body,
    header::HeaderMap,
    ErrorCode,
};

use anyhow::{Error, Result};

pub struct ResponseBuilder {
    // all errors generated while building the response will be deferred.
    pub(crate) inner: Result<Response>,
}

impl Default for ResponseBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Ok(Response::new()),
        }
    }

    /// Set the status code for the response.
    ///
    /// Default value: 200.
    #[inline]
    pub fn status_code(mut self, status_code: u16) -> Self {
        if let Ok(ref mut resp) = self.inner {
            resp.status_code = status_code;
        }
        self
    }

    /// Build the Response.
    #[inline]
    pub fn build(self) -> Result<Response, ErrorCode> {
        match self.inner {
            Ok(inner) => Ok(inner),
            Err(e) => Err(ErrorCode::InternalError(Some(e.to_string()))),
        }
    }
}

pub struct Response {
    pub(crate) headers: HeaderMap,
    pub(crate) body: Body,
    status_code: u16,
}

impl Default for Response {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<IncomingResponse> for Response {
    type Error = Error;

    fn try_from(incoming_response: IncomingResponse) -> std::result::Result<Self, Self::Error> {
        let status_code = incoming_response.status();
        let headers = incoming_response.headers_map()?;
        // The consume() method can only be called once
        let incoming_body = incoming_response.consume().unwrap();
        drop(incoming_response);

        Ok(Self {
            headers,
            status_code,
            body: Body::Stream(incoming_body.into()),
        })
    }
}

impl Response {
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            status_code: 200,
            body: Body::Bytes(vec![]),
        }
    }

    #[inline]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    #[inline]
    /// Get the status code of the response.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}

pub fn handle_response(response_out: ResponseOutparam, response: Response) {
    let outgoing_response = OutgoingResponse::new(response.headers.try_into().unwrap());
    outgoing_response
        .set_status_code(response.status_code)
        .unwrap();
    let outgoing_body = outgoing_response.body().unwrap();
    ResponseOutparam::set(response_out, Ok(outgoing_response));

    let body = response.body.bytes().unwrap();
    if !body.is_empty() {
        let out = outgoing_body.write().unwrap();
        // `blocking-write-and-flush` writes up to 4096 bytes
        let chunks = body.chunks(4096);
        for chunk in chunks {
            out.blocking_write_and_flush(chunk).unwrap();
        }
    }

    OutgoingBody::finish(outgoing_body, None).unwrap();
}
