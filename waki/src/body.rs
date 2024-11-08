use crate::bindings::wasi::{
    http::types::{IncomingBody, InputStream},
    io::streams::StreamError,
};

use anyhow::{anyhow, Result};

pub struct IncomingBodyStream {
    // input-stream resource is a child: it must be dropped before the parent incoming-body is dropped
    input_stream: InputStream,
    _incoming_body: IncomingBody,
}

impl From<IncomingBody> for IncomingBodyStream {
    #[inline]
    fn from(body: IncomingBody) -> Self {
        Self {
            // The stream() method can only be called once
            input_stream: body.stream().unwrap(),
            _incoming_body: body,
        }
    }
}

impl InputStream {
    pub fn chunk(&self, len: u64) -> Result<Option<Vec<u8>>> {
        match self.blocking_read(len) {
            Ok(c) => Ok(Some(c)),
            Err(StreamError::Closed) => Ok(None),
            Err(e) => Err(anyhow!("input_stream read failed: {e:?}"))?,
        }
    }
}

pub enum Body {
    Bytes(Vec<u8>),
    Stream(IncomingBodyStream),
}

impl Body {
    #[inline]
    pub fn chunk(&self, len: u64) -> Result<Option<Vec<u8>>> {
        match &self {
            Body::Bytes(_) => Ok(None),
            Body::Stream(s) => s.input_stream.chunk(len),
        }
    }

    pub fn bytes(self) -> Result<Vec<u8>> {
        match self {
            Body::Bytes(data) => Ok(data),
            Body::Stream(s) => {
                let mut body = Vec::new();
                while let Some(mut chunk) = s.input_stream.chunk(1024 * 1024)? {
                    body.append(&mut chunk);
                }
                Ok(body)
            }
        }
    }
}
