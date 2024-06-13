use crate::{
    bindings::wasi::http::{
        outgoing_handler,
        types::{IncomingRequest, OutgoingBody, OutgoingRequest, RequestOptions, Scheme},
    },
    body::Body,
    header::HeaderMap,
    ErrorCode, Method, Response,
};

use anyhow::{anyhow, Error, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

pub struct RequestBuilder {
    // all errors generated while building the request will be deferred and returned when `send` the request.
    pub(crate) inner: Result<Request>,
}

impl RequestBuilder {
    pub fn new(method: Method, url: &str) -> Self {
        Self {
            inner: Url::parse(url)
                .map_or_else(|e| Err(Error::new(e)), |url| Ok(Request::new(method, url))),
        }
    }

    /// Modify the query string of the Request URL.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use waki::Client;
    /// # fn run() -> Result<()> {
    /// let resp = Client::new().get("https://httpbin.org/get")
    ///     .query(&[("a", "b"), ("c", "d")])
    ///     .send()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        let mut err = None;
        if let Ok(ref mut req) = self.inner {
            let mut pairs = req.url.query_pairs_mut();
            let serializer = serde_urlencoded::Serializer::new(&mut pairs);
            if let Err(e) = query.serialize(serializer) {
                err = Some(e.into());
            }
        }
        if let Some(e) = err {
            self.inner = Err(e);
        }
        self
    }

    /// Set the timeout for the initial connect to the HTTP Server.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use std::time::Duration;
    /// # use waki::Client;
    /// # fn run() -> Result<()> {
    /// let resp = Client::new().post("https://httpbin.org/post")
    ///     .connect_timeout(Duration::from_secs(5))
    ///     .send()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        if let Ok(ref mut req) = self.inner {
            req.connect_timeout = Some(timeout.as_nanos() as u64);
        }
        self
    }

    /// Build the Request.
    pub fn build(self) -> Result<Request> {
        self.inner
    }

    /// Send the Request, returning a [`Response`].
    pub fn send(self) -> Result<Response> {
        match self.inner {
            Ok(req) => req.send(),
            Err(e) => Err(e),
        }
    }
}

pub struct Request {
    method: Method,
    url: Url,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Body,
    connect_timeout: Option<u64>,
}

impl TryFrom<IncomingRequest> for Request {
    type Error = ErrorCode;

    fn try_from(req: IncomingRequest) -> std::result::Result<Self, Self::Error> {
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

        let headers = req
            .headers_map()
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
        // The consume() method can only be called once
        let incoming_body = req.consume().unwrap();
        drop(req);

        Ok(Self {
            method,
            url,
            headers,
            body: Body::Stream(incoming_body.into()),
            connect_timeout: None,
        })
    }
}

impl Request {
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: Body::Bytes(vec![]),
            connect_timeout: None,
        }
    }

    pub fn builder(method: Method, url: &str) -> RequestBuilder {
        RequestBuilder::new(method, url)
    }

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

    fn send(self) -> Result<Response> {
        let req = OutgoingRequest::new(self.headers.try_into()?);
        req.set_method(&self.method)
            .map_err(|()| anyhow!("failed to set method"))?;

        let scheme = match self.url.scheme() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => Scheme::Other(other.to_string()),
        };
        req.set_scheme(Some(&scheme))
            .map_err(|()| anyhow!("failed to set scheme"))?;

        req.set_authority(Some(self.url.authority()))
            .map_err(|()| anyhow!("failed to set authority"))?;

        let path = match self.url.query() {
            Some(query) => format!("{}?{query}", self.url.path()),
            None => self.url.path().to_string(),
        };
        req.set_path_with_query(Some(&path))
            .map_err(|()| anyhow!("failed to set path_with_query"))?;

        let options = RequestOptions::new();
        options
            .set_connect_timeout(self.connect_timeout)
            .map_err(|()| anyhow!("failed to set connect_timeout"))?;

        let outgoing_body = req
            .body()
            .map_err(|_| anyhow!("outgoing request write failed"))?;
        let body = self.body.bytes()?;
        if !body.is_empty() {
            let request_body = outgoing_body
                .write()
                .map_err(|_| anyhow!("outgoing request write failed"))?;
            request_body.blocking_write_and_flush(&body)?;
        }
        OutgoingBody::finish(outgoing_body, None)?;

        let future_response = outgoing_handler::handle(req, Some(options))?;
        let incoming_response = match future_response.get() {
            Some(result) => result.map_err(|()| anyhow!("response already taken"))?,
            None => {
                let pollable = future_response.subscribe();
                pollable.block();

                future_response
                    .get()
                    .expect("incoming response available")
                    .map_err(|()| anyhow!("response already taken"))?
            }
        }?;
        drop(future_response);

        incoming_response.try_into()
    }
}
