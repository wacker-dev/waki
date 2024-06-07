use crate::{Method, RequestBuilder};

#[derive(Default)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::Get, url)
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::Post, url)
    }

    pub fn put(&self, url: &str) -> RequestBuilder {
        self.request(Method::Put, url)
    }

    pub fn patch(&self, url: &str) -> RequestBuilder {
        self.request(Method::Patch, url)
    }

    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.request(Method::Delete, url)
    }

    pub fn head(&self, url: &str) -> RequestBuilder {
        self.request(Method::Head, url)
    }

    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        RequestBuilder::new(method, url)
    }
}
