use super::{constants, Part};

use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes, BytesMut};
use httparse::Status;
use std::collections::HashMap;

struct Buffer {
    buf: BytesMut,
}

impl Buffer {
    fn new(data: &[u8]) -> Self {
        Self { buf: data.into() }
    }

    fn peek_exact(&mut self, size: usize) -> Option<&[u8]> {
        self.buf.get(..size)
    }

    fn read_until(&mut self, pattern: &[u8]) -> Option<Bytes> {
        memchr::memmem::find(&self.buf, pattern)
            .map(|idx| self.buf.split_to(idx + pattern.len()).freeze())
    }

    fn read_to(&mut self, pattern: &[u8]) -> Option<Bytes> {
        memchr::memmem::find(&self.buf, pattern).map(|idx| self.buf.split_to(idx).freeze())
    }

    fn advance(&mut self, n: usize) {
        self.buf.advance(n)
    }
}

pub fn parse(body: &[u8], boundary: &str) -> Result<HashMap<String, Part>> {
    let mut buffer = Buffer::new(body);
    let boundary = format!("{}{}", constants::BOUNDARY_EXT, boundary);

    // Finding the first boundary
    if buffer
        .read_until(format!("{}{}", boundary, constants::CRLF).as_bytes())
        .is_none()
    {
        return Err(anyhow!("incomplete multipart data, missing boundary"));
    };

    let mut parts = HashMap::new();

    loop {
        // Finding headers
        let header_bytes = match buffer.read_until(constants::CRLF_CRLF.as_bytes()) {
            Some(bytes) => bytes,
            None => return Err(anyhow!("incomplete multipart data, missing headers")),
        };

        let mut part = Part::new("", vec![]);
        let mut headers = [httparse::EMPTY_HEADER; constants::MAX_HEADERS];
        part.headers = match httparse::parse_headers(&header_bytes, &mut headers)? {
            Status::Complete((_, raw_headers)) => {
                let mut headers_map = HashMap::new();
                for header in raw_headers {
                    let (k, v) = (
                        header.name.to_string(),
                        String::from_utf8(header.value.to_vec())?,
                    );
                    if k.to_uppercase() == "Content-Disposition".to_uppercase() {
                        // can't parse it without a /
                        let mime = format!("multipart/{}", v).parse::<mime::Mime>()?;
                        part.key = match mime.get_param("name") {
                            Some(name) => name.to_string(),
                            None => {
                                return Err(anyhow!(
                                    "missing name field in the Content-Disposition header"
                                ))
                            }
                        };
                        part.filename = mime.get_param("filename").map(|v| v.to_string());
                    };
                    if k.to_uppercase() == "Content-Type".to_uppercase() {
                        part.mime = Some(v.parse()?)
                    }
                    headers_map.insert(k, v);
                }
                headers_map
            }
            Status::Partial => return Err(anyhow!("failed to parse field complete headers")),
        };

        // Finding field data
        part.value = match buffer.read_to(format!("{}{}", constants::CRLF, boundary).as_bytes()) {
            Some(bytes) => bytes.to_vec(),
            None => return Err(anyhow!("incomplete multipart data, missing field data")),
        };

        // Determine end of stream
        if buffer.read_until(boundary.as_bytes()).is_none() {
            return Err(anyhow!("incomplete multipart data, missing boundary"));
        };
        let next_bytes = match buffer.peek_exact(constants::BOUNDARY_EXT.len()) {
            Some(bytes) => bytes,
            None => return Err(anyhow!("incomplete multipart data")),
        };

        parts.insert(part.key.clone(), part);

        if next_bytes == constants::BOUNDARY_EXT.as_bytes() {
            return Ok(parts);
        }
        // discard \r\n.
        buffer.advance(constants::CRLF.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<()> {
        let data = b"--boundary\r\nContent-Disposition: form-data; name=field1\r\n\r\nvalue1\r\n--boundary\r\nContent-Disposition: form-data; name=field2; filename=file.txt\r\nContent-Type: text/plain\r\n\r\nhello\r\n--boundary--";

        let parts = parse(data, "boundary")?;
        let field1 = parts.get("field1").unwrap();
        assert_eq!(field1.key, "field1");
        assert_eq!(field1.value, b"value1");
        assert_eq!(field1.filename, None);
        assert_eq!(field1.mime, None);
        assert_eq!(field1.headers.len(), 1);

        let field2 = parts.get("field2").unwrap();
        assert_eq!(field2.key, "field2");
        assert_eq!(field2.value, b"hello");
        assert_eq!(field2.filename, Some("file.txt".into()));
        assert_eq!(field2.mime, Some(mime::TEXT_PLAIN));
        assert_eq!(field2.headers.len(), 2);
        Ok(())
    }
}
