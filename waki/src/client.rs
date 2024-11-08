use crate::{Method, RequestBuilder};

#[derive(Default)]
pub struct Client {}

impl Client {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::Get, url)
    }

    #[inline]
    pub fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::Post, url)
    }

    #[inline]
    pub fn put(&self, url: &str) -> RequestBuilder {
        self.request(Method::Put, url)
    }

    #[inline]
    pub fn patch(&self, url: &str) -> RequestBuilder {
        self.request(Method::Patch, url)
    }

    #[inline]
    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.request(Method::Delete, url)
    }

    #[inline]
    pub fn head(&self, url: &str) -> RequestBuilder {
        self.request(Method::Head, url)
    }

    #[inline]
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        RequestBuilder::new(method, url)
    }
}
