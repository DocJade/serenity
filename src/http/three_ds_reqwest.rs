#![cfg(feature = "3ds")]

// re-implementations of reqwest

use std::fmt;
use std::{borrow::Cow, future::Future};

pub use http::header::HeaderName as HeaderName;
pub use http::header::HeaderValue as HeaderValue;
pub use url::Url as Url;
pub use http::HeaderMap as HeaderMap;
pub use http::Method as Method;
pub use http::header::InvalidHeaderValue as InvalidHeaderValue;

pub type ReqwestResult<T> = std::result::Result<T, Error>;

// https://github.com/seanmonstar/reqwest/blob/04a216fc17d75b4ebe4b0829ae7bbd8279c0dcab/src/into_url.rs#L7
pub trait IntoUrl: IntoUrlSealed {}
impl IntoUrl for Url {}
impl IntoUrl for String {}
impl<'a> IntoUrl for &'a str {}
impl<'a> IntoUrl for &'a String {}
pub trait IntoUrlSealed {
    // Besides parsing as a valid `Url`, the `Url` must be a valid
    // `http::Uri`, in that it makes sense to use in a network request.
    fn into_url(self) -> crate::Result<Url>;

    fn as_str(&self) -> &str;
}
// https://github.com/seanmonstar/reqwest/blob/04a216fc17d75b4ebe4b0829ae7bbd8279c0dcab/src/into_url.rs#L40-L45
impl IntoUrlSealed for Url {
    fn into_url(self) -> crate::Result<Url> { unimplemented!("3DS STUB: IntoUrlSealed URL into_url") }
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}
impl<'a> IntoUrlSealed for &'a str {
    fn into_url(self) -> crate::Result<Url> {
        unimplemented!("3DS STUB: IntoUrlSealed &'a str into_url")
    }

    fn as_str(&self) -> &str {
        self
    }
}

impl<'a> IntoUrlSealed for &'a String {
    fn into_url(self) -> crate::Result<Url> {
        (&**self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl IntoUrlSealed for String {
    fn into_url(self) -> crate::Result<Url> {
        (&*self).into_url()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

// https://docs.rs/reqwest/latest/reqwest/struct.Error.html
#[derive(Debug)]
pub struct Error {
    inner: Box<Inner>,
}
#[derive(Debug)]
struct Inner {
    kind: Kind,
    source: Option<BoxError>,
    url: Option<Url>,
}
#[derive(Debug)]
pub(crate) enum Kind {
    Builder,
    Request,
    Redirect,
    #[cfg(not(target_arch = "wasm32"))]
    Status(StatusCode, Option<()>),
    #[cfg(target_arch = "wasm32")]
    Status(StatusCode),
    Body,
    Decode,
    Upgrade,
}
use std::error::Error as StdError;
pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;
// Needs mf std error
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "3DS Reqwest Stub Error: {:?}", self.inner.kind)
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}


#[derive(Debug, Clone)]
pub struct Client;
impl Client {
    pub fn new() -> Self { Self }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.request
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = url.into_url().map(move |url| Request::new(method, url));
        RequestBuilder::new(self.clone(), req)
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.execute
    pub fn execute(
        &self,
        request: Request,
    ) -> impl Future<Output = Result<Response, crate::Error>> {
        async {
            unimplemented!("3DS STUB: CLIENT EXECUTE")
        }
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.get
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }
}

// https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
pub struct ClientBuilder {
    config: Config,
}
impl ClientBuilder {
    // https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.new
    pub fn new() -> Self {
        // TODO: the body is huge, fill in later
        unimplemented!("3DS STUB: ClientBuilder NEW")
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.build
    pub fn build(self) -> crate::Result<Client> {
        // TODO: ditto.
        unimplemented!("3DS STUB: ClientBuilder BUILD")
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.use_rustls_tls
    pub fn use_rustls_tls(self) -> ClientBuilder {
        // TODO: TLS stuff.
        unimplemented!("3DS STUB: ClientBuilder RUSTLS_TLS")
    }
}
// todo
struct Config {

}

// https://github.com/seanmonstar/reqwest/blob/master/src/async_impl/body.rs#L373-L374C5
pub(crate) type ResponseBody =
    http_body_util::combinators::BoxBody<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug)]
pub struct HyperResponse<T> {
    pub something_idk_yet: T,
}
impl HyperResponse<ResponseBody> {
    pub fn headers(&self) -> &HeaderMap {
        unimplemented!("3DS STUB: HyperResponse HEADERS")
    }
    pub fn status(&self) -> StatusCode {
        unimplemented!("3DS STUB: HyperResponse STATUS")
    }
    fn into_body(self) -> ResponseBody { // not sure if this is the right return type.
        unimplemented!("3DS STUB: HyperResponse INTO_BODY")
    }
}

// https://github.com/seanmonstar/reqwest/blob/master/src/async_impl/response.rs#L27
#[derive(Debug)]
pub struct Response {
    pub(super) res: HyperResponse<ResponseBody>,
    // Boxed to save space (11 words to 1 word), and it's not accessed
    // frequently internally.
    url: Box<Url>,
}

impl Response {
    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.headers
    pub fn headers(&self) -> &HeaderMap {
        self.res.headers()
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.status
    pub fn status(&self) -> StatusCode {
        self.res.status()
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.url
    pub fn url(&self) -> &Url {
        &self.url
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.bytes
    // Ive changed teh error return to always give none on the optionals, no idea
    // if this will be an issue.
    pub async fn bytes(self) -> ReqwestResult<bytes::Bytes> {
        use http_body_util::BodyExt;

        BodyExt::collect(self.res.into_body())
            .await
            .map(|buf| buf.to_bytes())
            .map_err(|_| Error {inner: Box::new(Inner { kind: Kind::Decode, source: None, url: None })})
            
    }
}

// https://docs.rs/reqwest/latest/reqwest/struct.StatusCode.html
pub use http::StatusCode as StatusCode;

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

// https://docs.rs/reqwest/latest/src/reqwest/async_impl/request.rs.html#26-33
pub struct Request {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Body>,
    version: http::Version,
    extensions: http::Extensions,
}

impl Request {
    // https://docs.rs/reqwest/latest/reqwest/struct.Request.html#method.new
    pub fn new(method: Method, url: Url) -> Self {
        Request {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            version: http::Version::default(),
            extensions: http::Extensions::new(),
        }
    }
}


// https://docs.rs/reqwest/latest/src/reqwest/async_impl/request.rs.html#39-42
pub struct RequestBuilder {
    client: Client,
    request: crate::Result<Request>,
}
impl RequestBuilder {
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.header
    pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        unimplemented!("3DS STUB: REQUESTBUILDER HEADER")
    }
    // make bodies out of bytes
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.body
    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        unimplemented!("3DS STUB: REQUESTBUILDER BODY")
    }
    // TODO: due to how the 3ds does multithreaded this is probably sync, how do we not block?
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.send
    pub fn send(self) -> impl Future<Output = Result<Response, crate::Error>> {
        async {
            unimplemented!("3DS STUB: REQUESTBUILDER SEND");
        }
    }

    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.multipart
    pub fn multipart(self, _form: Form) -> Self {
        unimplemented!("3DS STUB: REQUESTBUILDER MULTIPART")
    }

    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.headers
    pub fn headers(self, headers: HeaderMap) -> Self {
        unimplemented!("3DS STUB: REQUESTBUILDER HEADERS")
    }
    
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.build
    pub fn build(self) -> crate::Result<Request> {
        self.request
    }
    pub fn new(client: Client, request: crate::Result<Request>) -> RequestBuilder {
        unimplemented!("3DS STUB: REQUESTBUILDER NEW")
    }
}