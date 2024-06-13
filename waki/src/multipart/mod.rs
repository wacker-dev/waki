mod constants;
pub(crate) mod parser;

use anyhow::Result;
use mime::Mime;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Form {
    parts: Vec<Part>,
    boundary: String,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    pub fn new() -> Self {
        Self {
            parts: vec![],
            boundary: format!("--FormBoundary{}", generate_random_string(10)),
        }
    }

    pub(crate) fn boundary(&self) -> &str {
        &self.boundary
    }

    pub fn text<S, V>(mut self, key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.parts.push(Part::new(key, value));
        self
    }

    pub fn file<S, P>(mut self, key: S, path: P) -> Result<Self>
    where
        S: Into<String>,
        P: AsRef<Path>,
    {
        self.parts.push(Part::file(key, path)?);
        Ok(self)
    }

    pub fn part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut buf = vec![];
        for part in self.parts {
            buf.extend_from_slice(
                format!(
                    "{}{}{}Content-Disposition: form-data; name={}",
                    constants::BOUNDARY_EXT,
                    self.boundary,
                    constants::CRLF,
                    part.key
                )
                .as_bytes(),
            );
            if let Some(filename) = part.filename {
                buf.extend_from_slice(format!("; filename=\"{}\"", filename).as_bytes());
            }
            if let Some(mime) = part.mime {
                buf.extend_from_slice(
                    format!("{}Content-Type: {}", constants::CRLF, mime).as_bytes(),
                );
            }
            for (k, v) in part.headers {
                buf.extend_from_slice(format!("{}{}: {}", constants::CRLF, k, v).as_bytes());
            }

            buf.extend_from_slice(constants::CRLF_CRLF.as_bytes());
            buf.extend_from_slice(&part.value);
            buf.extend_from_slice(constants::CRLF.as_bytes());
        }
        buf.extend_from_slice(
            format!(
                "{}{}{}",
                constants::BOUNDARY_EXT,
                self.boundary,
                constants::BOUNDARY_EXT,
            )
            .as_bytes(),
        );
        buf
    }
}

fn generate_random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub struct Part {
    pub key: String,
    pub value: Vec<u8>,
    pub filename: Option<String>,
    pub mime: Option<Mime>,
    pub headers: HashMap<String, String>,
}

impl Part {
    pub fn new<S, V>(key: S, value: V) -> Self
    where
        S: Into<String>,
        V: Into<Vec<u8>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
            filename: None,
            mime: None,
            headers: HashMap::new(),
        }
    }

    pub fn file<S, P>(key: S, path: P) -> Result<Self>
    where
        S: Into<String>,
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer)?;
        let part = Part::new(key, buffer).mime(mime);

        match path
            .file_name()
            .map(|filename| filename.to_string_lossy().to_string())
        {
            Some(name) => Ok(part.filename(name)),
            None => Ok(part),
        }
    }

    pub fn mime(mut self, mime: Mime) -> Self {
        self.mime = Some(mime);
        self
    }

    pub fn mime_str(mut self, mime: &str) -> Result<Self> {
        self.mime = Some(mime.parse()?);
        Ok(self)
    }

    pub fn filename<S: Into<String>>(mut self, name: S) -> Self {
        self.filename = Some(name.into());
        self
    }

    pub fn headers<S, I>(mut self, headers: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, S)>,
    {
        self.headers
            .extend(headers.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }
}
