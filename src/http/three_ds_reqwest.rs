#![cfg(feature = "3ds")]

// re-implementations of reqwest

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::{borrow::Cow, future::Future};
use minreq;

pub use http::header::HeaderName as HeaderName;
pub use http::header::HeaderValue as HeaderValue;
use tracing::info;
pub use url::Url as Url;
pub use http::HeaderMap as HeaderMap;
pub use http::Method as Method;
pub use http::header::InvalidHeaderValue as InvalidHeaderValue;
use http_body_util::{BodyExt, Full};
use bytes::Bytes;

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
    fn into_url(self) -> crate::Result<Url> { Ok(self) }
    fn as_str(&self) -> &str { self.as_ref() }
}
impl<'a> IntoUrlSealed for &'a str {
    fn into_url(self) -> crate::Result<Url> {
        Url::parse(self).map_err(|_| crate::Error::Other("Invalid URL"))
    }
    fn as_str(&self) -> &str { self }
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
pub struct Client {
    config: Arc<Config>,
}
impl Client {
    pub fn new() -> Self {
        // defaults i guess
       
        Self {
            config: ClientBuilder::new().config.into()
        }
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.request
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = url.into_url().map(move |url| Request::new(method, url));
        RequestBuilder::new(self.clone(), req)
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.execute
    // https://docs.rs/reqwest/latest/reqwest/struct.Client.html#method.execute
    pub fn execute(&self, request: Request) -> impl Future<Output = Result<Response, crate::Error>> {
        let config = self.config.clone();
        let url_copy = request.url.clone(); // Capture URL for printing
        let url_string = request.url.to_string(); // Capture URL for printing
        async move {
            println!("execute: {} {}", request.method, url_string); // TODO: DEBUGGING
            
            // We use spawn_blocking because minreq is a blocking library.
            // On the 3DS, running this directly in the async executor often starves the 
            // background threads responsible for the SOC service.
            let res = tokio::task::spawn_blocking(move || {
                let mut min_req = minreq::Request::new(convert_http_method(request.method.clone()), request.url.as_str());
                // timeouts!
                // TODO: Timeouts spawn a thread which breaks everything, so no timeouts!
                // min_req = min_req.with_timeout(config.timeout.unwrap_or(120));
                
                // info!("Adding headers...");
                for (name, value) in &request.headers {
                    // info!("{} : {}", name.as_str(), value.to_str().unwrap_or("FAILED TO PARSE TO STRING!"));
                    min_req = min_req.with_header(name.as_str(), value.to_str().unwrap_or(""));
                }

                if let Some(body) = request.body {
                    min_req = min_req.with_body(body.bytes.clone());
                    println!("{}", String::from_utf8(body.bytes).expect("Should be valid UTF-8!"));
                } else {
                    // no body in the request?
                    // TODO: is this bad?
                    tracing::error!("No body in outgoing message!")
                }

                println!("Sending request...");

                match min_req.clone().send() {
                    Ok(ok) => Ok(ok),
                    Err(err) =>{
                        println!("minreq failed. {err:#?}");
                        return Err(crate::Error::Other("minreq failed"));
                    }
                }
            }).await;

            match res {
                Ok(Ok(min_res)) => {
                    println!("Response code: {}", min_res.status_code);
                    println!("Response body: {:#?}", min_res.as_str());
                    Ok(http_response_from_minreq_response(min_res, url_copy))
                }
                Ok(Err(e)) => Err(e),
                Err(_) => Err(crate::Error::Other("Spawn blocking task failed")),
            }
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
        Self {
            config: Config::default(),
        }
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.build
   pub fn build(self) -> crate::Result<Client> {
        // this is where a pool would be made, but we lazy
        Ok(Client {
            config: Arc::new(self.config),
        })
    }
    // https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html#method.use_rustls_tls
    pub fn use_rustls_tls(self) -> Self {
        // "yeah we are using tls i promise... stop bugging me!"
        self
    }
}
#[derive(Debug)]
struct Config {
    user_agent: Option<HeaderValue>,
    /// Timeout in seconds
    timeout: Option<u64>,
    proxy: Option<String>, 
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            // user_agent: Some(HeaderValue::from_static("3DSCord (3DS; Nintendo 3DS)")),
            user_agent: Some(HeaderValue::from_static("DiscordBot (https://github.com/serenity-rs/serenity)")),
            timeout: Some(30),
            proxy: None,
        }
    }
}

// https://github.com/seanmonstar/reqwest/blob/master/src/async_impl/body.rs#L373-L374C5
pub(crate) type ResponseBody =
    http_body_util::combinators::BoxBody<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>;
#[derive(Debug)]
pub struct HyperResponse<T> {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: T,
}
impl HyperResponse<ResponseBody> {
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn status(&self) -> StatusCode {
        self.status
    }
    fn into_body(self) -> ResponseBody { // not sure if this is the right return type.
        self.body
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

pub struct Body {
    bytes: Vec<u8>
}

impl Body {
    // todo
}

impl From<Vec<u8>> for Body {
    fn from(value: Vec<u8>) -> Self {
        Self {
            bytes: value
        }
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
    pub fn header<K, V>(mut self, key: K, value: V) -> Self 
    where 
        HeaderName: TryFrom<K>, <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>, <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        if let Ok(ref mut req) = self.request {
            if let (Ok(k), Ok(v)) = (HeaderName::try_from(key), HeaderValue::try_from(value)) {
                req.headers.insert(k, v);
            }
        }
        self
    }
    // make bodies out of bytes
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.body
    pub fn body<T: Into<Body>>(mut self, body: T) -> Self {
        if let Ok(ref mut req) = self.request {
            req.body = Some(body.into());
        }
        self
    }
    // TODO: due to how the 3ds does multithreaded this is probably sync, how do we not block?
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.send
    pub fn send(self) -> impl Future<Output = Result<Response, crate::Error>> {
        async move {
            let req = self.request.map_err(|_| crate::Error::Other("Failed to build request."))?;
            self.client.execute(req).await
        }
    }

    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.multipart
    pub fn multipart(self, _form: Form) -> Self {
        unimplemented!("3DS STUB: REQUESTBUILDER MULTIPART")
    }

    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.headers
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        if let Ok(ref mut request) = self.request {
            // extend those headers
            request.headers.extend(headers);
        }
        self
    }
    
    // https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.build
    pub fn build(self) -> crate::Result<Request> {
        self.request
    }
    pub fn new(client: Client, request: crate::Result<Request>) -> Self {
        Self { client, request }
    }
}


/// Convert http::Method to minireq::Method
fn convert_http_method(method: http::Method) -> minreq::Method {
    match method {
        Method::GET => {
            minreq::Method::Get
        }
        Method::POST => {
            minreq::Method::Post
        }
        Method::PUT => {
            minreq::Method::Put
        }
        Method::DELETE => {
            minreq::Method::Delete
        }
        Method::HEAD => {
            minreq::Method::Head
        }
        Method::OPTIONS => {
            minreq::Method::Options
        }
        Method::CONNECT => {
            minreq::Method::Connect
        }
        Method::PATCH => {
            minreq::Method::Patch
        }
        Method::TRACE => {
            minreq::Method::Trace
        }
        _ => {
            // No idea what these do.
            unimplemented!("Cannot convert http method ({method:#?}) to minreq method!")
        }
    }
}

fn http_response_from_minreq_response(response: minreq::Response, url: impl Into<Box<Url>>) -> Response {
    let status: StatusCode = StatusCode::from_u16(response.status_code as u16).expect("Unable to convert status code!");
    let headers = http_headermap_from_minreq_headers(&response.headers);
    let raw_body = response.into_bytes();
    let bytes = Bytes::from(raw_body);
    // "Full body never errors on collect"
    let body: ResponseBody = Full::new(bytes).map_err(|never| match never {}).boxed();
    Response {
        res: HyperResponse {
            status,
            headers,
            body
        },
        url: url.into(),
    }
}

fn http_headermap_from_minreq_headers(headers: &HashMap<String, String>) -> http::HeaderMap {
    let mut header_map: HeaderMap = HeaderMap::new();
    for (name, value) in headers {
        // get the header name
        if let Ok(header_name) = HeaderName::from_str(name) {
            // now get the value
            if let Ok(header_value) = HeaderValue::from_str(value) {
                header_map.insert(header_name, header_value);
            }
        }
    }
    header_map
}