use crate::{
    bindings::wasi::{
        http::types::{IncomingBody, IncomingRequest, InputStream, Scheme},
        io::streams::StreamError,
    },
    Method,
};

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use url::Url;

pub struct Request {
    url: Url,
    method: Method,
    headers: HashMap<String, String>,
    // input-stream resource is a child: it must be dropped before the parent incoming-body is dropped
    input_stream: InputStream,
    _incoming_body: IncomingBody,
}

impl From<IncomingRequest> for Request {
    fn from(req: IncomingRequest) -> Self {
        let scheme = match req.scheme().unwrap_or(Scheme::Http) {
            Scheme::Http => "http".into(),
            Scheme::Https => "https".into(),
            Scheme::Other(s) => s,
        };
        let method = req.method();
        let url = Url::parse(&format!(
            "{}://{}{}",
            scheme,
            req.authority().unwrap_or("localhost".into()),
            req.path_with_query().unwrap_or("/".into())
        ))
        .unwrap();

        let headers_handle = req.headers();
        let headers: HashMap<String, String> = headers_handle
            .entries()
            .into_iter()
            .map(|(key, value)| (key, String::from_utf8_lossy(&value).to_string()))
            .collect();
        drop(headers_handle);

        // The consume() method can only be called once
        let incoming_body = req.consume().unwrap();
        drop(req);

        // The stream() method can only be called once
        let input_stream = incoming_body.stream().unwrap();
        Self {
            url,
            method,
            headers,
            input_stream,
            _incoming_body: incoming_body,
        }
    }
}

impl Request {
    /// Get the full URL of the request.
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    /// Get the HTTP method of the request.
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// Get the path of the request.
    pub fn path(&self) -> String {
        self.url.path().to_string()
    }

    /// Get the query string of the request.
    pub fn query(&self) -> HashMap<String, String> {
        let query_pairs = self.url.query_pairs();
        query_pairs.into_owned().collect()
    }

    /// Get the headers of the request.
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// Get a chunk of the request body.
    ///
    /// It will block until at least one byte can be read or the stream is closed.
    pub fn chunk(&self, len: u64) -> Result<Option<Vec<u8>>> {
        match self.input_stream.blocking_read(len) {
            Ok(c) => Ok(Some(c)),
            Err(StreamError::Closed) => Ok(None),
            Err(e) => Err(anyhow!("input_stream read failed: {e:?}"))?,
        }
    }

    /// Get the full request body.
    ///
    /// It will block until the stream is closed.
    pub fn body(self) -> Result<Vec<u8>> {
        let mut body = Vec::new();
        while let Some(mut chunk) = self.chunk(1024 * 1024)? {
            body.append(&mut chunk);
        }
        Ok(body)
    }
}
