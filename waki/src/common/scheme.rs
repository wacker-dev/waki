use crate::bindings::wasi::http::types::Scheme;
use std::fmt::{Display, Formatter, Result};

impl From<&str> for Scheme {
    fn from(s: &str) -> Self {
        match s {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => Scheme::Other(other.to_string()),
        }
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Other(s) => s,
        })
    }
}
