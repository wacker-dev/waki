use crate::bindings::wasi::http::types::Scheme;

impl From<&str> for Scheme {
    fn from(s: &str) -> Self {
        match s {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => Scheme::Other(other.to_string()),
        }
    }
}

impl TryInto<http::uri::Scheme> for Scheme {
    type Error = http::uri::InvalidUri;

    fn try_into(self) -> Result<http::uri::Scheme, Self::Error> {
        match self {
            Scheme::Http => Ok(http::uri::Scheme::HTTP),
            Scheme::Https => Ok(http::uri::Scheme::HTTPS),
            Scheme::Other(s) => s.as_str().try_into(),
        }
    }
}
