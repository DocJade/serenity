#![cfg(feature = "3ds")]

// re-implementations of reqwest types

use std::borrow::Cow;

use http::header::{HeaderName, HeaderValue};
pub use url::Url as Url;

#[derive(Debug)]
pub struct Client;
impl Client {
    pub fn new() -> Self { Self }
    pub fn request(&self, method: http::Method, url: Url) -> ReqwestRequestBuilder {
        ReqwestRequestBuilder {
            method,
            url,
            headers: http::HeaderMap::new(),
            body: None,
        }
    }
}


pub struct Form; 

impl Form {
    pub fn new() -> Self { Self }
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Form.html#method.part
    pub fn part<T>(self, name: T, part: Part) -> Form
    where
        T: Into<Cow<'static, str>>,
    {
        unimplemented!("3DS STUB: FORM PART")
    }
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Form.html#method.text
    pub fn text<T, U>(self, name: T, value: U) -> Form
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        unimplemented!("3DS STUB: FORM TEXT")
    }
}

pub struct Part;

impl Part {
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.bytes
    pub fn bytes<T>(value: T) -> Part
    where
        T: Into<Cow<'static, [u8]>>,
    {
        unimplemented!("3DS STUB: PART BYTES")
    }
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.file_name
    pub fn file_name<T>(self, filename: T) -> Part
    where
        T: Into<Cow<'static, str>>,
    {
        unimplemented!("3DS STUB: PART FILE_NAME")
    }
    // https://docs.rs/reqwest/latest/reqwest/multipart/struct.Part.html#method.mime_str
    pub fn mime_str(self, mime: &str) -> crate::Result<Part> {
        unimplemented!("3DS STUB: PART MIME_STR")
    }
}

pub struct Body;

impl Body {
    // todo
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        unimplemented!("3DS STUB: BODY FROM BYTES")
    }
}


// fake rewest builder
pub struct ReqwestRequestBuilder {
    method: http::Method,
    url: Url,
    headers: http::HeaderMap,
    body: Option<Vec<u8>>,
}
impl ReqwestRequestBuilder {
    pub fn header<K, V>(mut self, key: K, value: V) -> Self 
    where 
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
    {
        if let (Ok(k), Ok(v)) = (HeaderName::try_from(key), HeaderValue::try_from(value)) {
            self.headers.insert(k, v);
        }
        self
    }
    // make bodies out of bytes
    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.body
        unimplemented!("3DS STUB: BODY")
    }
    // TODO: due to how the 3ds does multithreaded this is probably sync, how do we not block?
    pub async fn send(self) -> Result<minreq::Response, minreq::Error> {
        // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.send
        unimplemented!("3DS STUB: SEND")
    }

    pub fn multipart(self, _form: Form) -> Self {
        // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.multipart
        unimplemented!("3DS STUB: MULTIPART")
    }

    pub fn headers(self, headers: http::HeaderMap) -> Self {
        // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.headers
        unimplemented!("3DS STUB: HEADERS")
    }
}