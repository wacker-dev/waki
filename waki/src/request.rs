use crate::{
    bindings::wasi::http::{
        outgoing_handler,
        types::{IncomingRequest, OutgoingBody, OutgoingRequest, RequestOptions},
    },
    body::{write_to_outgoing_body, Body},
    header::HeaderMap,
    ErrorCode, Method, Response,
};

use anyhow::{anyhow, Error, Result};
use http::{
    uri::{Authority, Parts, PathAndQuery},
    Uri,
};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::Duration;

pub struct RequestBuilder {
    // all errors generated while building the request will be deferred and returned when `send` the request.
    pub(crate) inner: Result<Request>,
}

impl RequestBuilder {
    #[inline]
    pub fn new(method: Method, uri: &str) -> Self {
        Self {
            inner: uri.parse::<Uri>().map_or_else(
                |e| Err(Error::new(e)),
                |uri| Ok(Request::new(method, uri.into_parts())),
            ),
        }
    }

    /// Modify the query string of the Request URI.
    ///
    /// ```
    /// # use anyhow::Result;
    /// # use waki::Client;
    /// # fn run() -> Result<()> {
    /// let resp = Client::new().get("https://httpbin.org/get")
    ///     .query([("a", "b"), ("c", "d")])
    ///     .send()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query<K, V, I>(mut self, args: I) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
    {
        let mut err = None;
        if let Ok(ref mut req) = self.inner {
            let (path, query) = match &req.uri.path_and_query {
                Some(path_and_query) => (
                    path_and_query.path(),
                    path_and_query.query().unwrap_or_default(),
                ),
                None => ("", ""),
            };
            let mut serializer = form_urlencoded::Serializer::new(query.to_string());
            serializer.extend_pairs(args);
            match PathAndQuery::try_from(format!("{}?{}", path, serializer.finish())) {
                Ok(path_and_query) => req.uri.path_and_query = Some(path_and_query),
                Err(e) => err = Some(e.into()),
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
    #[inline]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        if let Ok(ref mut req) = self.inner {
            req.connect_timeout = Some(timeout.as_nanos() as u64);
        }
        self
    }

    /// Build the Request.
    #[inline]
    pub fn build(self) -> Result<Request> {
        self.inner
    }

    /// Send the Request, returning a [`Response`].
    #[inline]
    pub fn send(self) -> Result<Response> {
        match self.inner {
            Ok(req) => req.send(),
            Err(e) => Err(e),
        }
    }
}

pub struct Request {
    method: Method,
    uri: Parts,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Body,
    connect_timeout: Option<u64>,
}

impl TryFrom<IncomingRequest> for Request {
    type Error = ErrorCode;

    fn try_from(req: IncomingRequest) -> std::result::Result<Self, Self::Error> {
        let method = req.method();

        let mut parts = Parts::default();
        if let Some(scheme) = req.scheme() {
            parts.scheme = Some(
                scheme
                    .try_into()
                    .map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
            );
        }
        if let Some(authority) = req.authority() {
            parts.authority = Some(
                authority
                    .try_into()
                    .map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
            );
        }
        if let Some(path_with_query) = req.path_with_query() {
            parts.path_and_query = Some(
                path_with_query
                    .try_into()
                    .map_err(|_| ErrorCode::HttpRequestUriInvalid)?,
            );
        }

        let headers = req
            .headers_map()
            .map_err(|e| ErrorCode::InternalError(Some(e.to_string())))?;
        // The consume() method can only be called once
        let incoming_body = req.consume().unwrap();
        drop(req);

        Ok(Self {
            method,
            uri: parts,
            headers,
            body: Body::Stream(incoming_body.into()),
            connect_timeout: None,
        })
    }
}

impl Request {
    #[inline]
    pub fn new(method: Method, uri: Parts) -> Self {
        Self {
            method,
            uri,
            headers: HeaderMap::new(),
            body: Body::Bytes(vec![]),
            connect_timeout: None,
        }
    }

    #[inline]
    pub fn builder(method: Method, uri: &str) -> RequestBuilder {
        RequestBuilder::new(method, uri)
    }

    /// Get the HTTP method of the request.
    #[inline]
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// Get the path of the request.
    #[inline]
    pub fn path(&self) -> &str {
        match &self.uri.path_and_query {
            Some(path_and_query) => path_and_query.path(),
            None => "",
        }
    }

    /// Get the query string of the request.
    pub fn query(&self) -> HashMap<String, String> {
        match &self.uri.path_and_query {
            Some(path_and_query) => {
                let query_pairs =
                    form_urlencoded::parse(path_and_query.query().unwrap_or_default().as_bytes());
                query_pairs.into_owned().collect()
            }
            None => HashMap::default(),
        }
    }

    /// Get the authority of the request.
    #[inline]
    pub fn authority(&self) -> &Option<Authority> {
        &self.uri.authority
    }

    fn send(self) -> Result<Response> {
        let req = OutgoingRequest::new(self.headers.try_into()?);
        req.set_method(&self.method)
            .map_err(|()| anyhow!("failed to set method"))?;
        if let Some(scheme) = self.uri.scheme {
            req.set_scheme(Some(&scheme.as_str().into()))
                .map_err(|()| anyhow!("failed to set scheme"))?;
        }
        if let Some(authority) = self.uri.authority {
            req.set_authority(Some(authority.as_str()))
                .map_err(|()| anyhow!("failed to set authority"))?;
        }
        if let Some(path_and_query) = self.uri.path_and_query {
            req.set_path_with_query(Some(path_and_query.as_str()))
                .map_err(|()| anyhow!("failed to set path_with_query"))?;
        }

        let options = RequestOptions::new();
        options
            .set_connect_timeout(self.connect_timeout)
            .map_err(|()| anyhow!("failed to set connect_timeout"))?;

        let outgoing_body = req
            .body()
            .map_err(|_| anyhow!("outgoing request write failed"))?;
        let body = self.body.bytes()?;
        write_to_outgoing_body(&outgoing_body, body.as_slice())?;
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
