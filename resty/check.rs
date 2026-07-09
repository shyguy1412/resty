#![feature(prelude_import)]
/*!# resty

resty is a fast and lightweight framework for concurrent and multithreadable REST APIs.

Its capable of manual routing aswell as path based routing similiar to Next.js or Tanstack Router.
It also capable of generating a JSON description of the API that can be used for client snippet and documentation generation.
To do so set `RESTY_DECL_GEN=output.json` as environment variable during compilation.

Known issues with rust-analyzer:

rust-analyzer currently does not support getting the sourcefile path of a Span.
This is vital for path based routing and can be worked around by setting `RESTY_PATH_ROUTING_HINT` to the base directory of your API routes.

Example settings for vscode

```json
{
    "rust-analyzer.server.extraEnv": {
        "RESTY_PATH_ROUTING_HINT": "src/routes/"
    }
}
```

## Example

```rust
#[resty::manual_routing]
static ROUTER: LazyLock<Router>;

#[resty::endpoint(
    Route("/"),
    Router(ROUTER),
    Method(GET),
    Responds(200, String)
    Header("Content-Type", "text/html; charset=utf-8")
)]
async fn get_hello_world<'a>(_req: &mut Request<'a>, res: &mut Response<'a>) -> resty::Result {
    res.ok(&"Hello World!").await?;

    Ok(())
}

fn main() -> ExitCode {
    const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3333);

    if let Err(error) = resty::bind::<TcpSocket>(ADDR, &ROUTER) {
        println!("{error:?}");
        return ExitCode::FAILURE;
    }

    println!("Listening on port 3333");

    let _: Vec<_> = std::thread::available_parallelism()
        .ok()
        .map(|n| 0..n.get())
        .unwrap_or(0..1)
        .map(|_| resty::spawn_thread())
        .collect();

    std::thread::park();

    return ExitCode::SUCCESS;
}
```
*/
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub use resty_macros::*;
mod parse {
    use smol::{io::AsyncReadExt, stream::StreamExt};
    use url::Url;
    use crate::Socket;
    /// Enum for every valid HTTP method with a variant for invalid methods
    #[rustfmt::skip]
    #[repr(u16)]
    pub enum HttpMethod {
        GET = 0b00000001,
        PUT = 0b00000010,
        POST = 0b00000100,
        DELETE = 0b00001000,
        PATCH = 0b00010000,
        HEAD = 0b00100000,
        OPTIONS = 0b01000000,
        TRACE = 0b10000000,
        INVALID = 0b00000000,
        ALL = u16::MAX,
    }
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for HttpMethod {}
    #[automatically_derived]
    impl ::core::clone::Clone for HttpMethod {
        #[inline]
        fn clone(&self) -> HttpMethod {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for HttpMethod {}
    #[automatically_derived]
    impl ::core::fmt::Debug for HttpMethod {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    HttpMethod::GET => "GET",
                    HttpMethod::PUT => "PUT",
                    HttpMethod::POST => "POST",
                    HttpMethod::DELETE => "DELETE",
                    HttpMethod::PATCH => "PATCH",
                    HttpMethod::HEAD => "HEAD",
                    HttpMethod::OPTIONS => "OPTIONS",
                    HttpMethod::TRACE => "TRACE",
                    HttpMethod::INVALID => "INVALID",
                    HttpMethod::ALL => "ALL",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for HttpMethod {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for HttpMethod {
        #[inline]
        fn eq(&self, other: &HttpMethod) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for HttpMethod {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for HttpMethod {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    impl From<&str> for HttpMethod {
        fn from(value: &str) -> HttpMethod {
            use HttpMethod::*;
            match value.to_ascii_uppercase().as_str() {
                "GET" => GET,
                "PUT" => PUT,
                "POST" => POST,
                "DELETE" => DELETE,
                "PATCH" => PATCH,
                "HEAD" => HEAD,
                "OPTIONS" => OPTIONS,
                "TRACE" => TRACE,
                _ => INVALID,
            }
        }
    }
    impl std::fmt::Display for HttpMethod {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{0:?}", self))
        }
    }
    #[inline(always)]
    pub(crate) async fn read_headers<S: Socket>(
        stream: &mut S::Stream,
        buff: &mut Vec<u8>,
    ) -> Option<usize> {
        let mut header_count: usize = 0;
        let bytes = &mut stream.bytes();
        while let Some(Ok(byte)) = bytes.next().await {
            buff.push(byte);
            if buff.len() < 4 {
                continue;
            }
            if buff[buff.len() - 2..] == *b"\r\n" {
                header_count = header_count.wrapping_add(1);
            }
            if buff[buff.len() - 4..] == *b"\r\n\r\n" {
                return Some(header_count);
            }
        }
        None
    }
    #[inline(always)]
    pub(crate) fn parse_cookies<'a>(
        req: &httparse::Request<'a, 'a>,
    ) -> Vec<(&'a str, &'a str)> {
        req.headers
            .iter()
            .filter_map(|h| match h.name == "Cookie" {
                true => str::from_utf8(h.value).ok(),
                false => None,
            })
            .flat_map(|value| value.split("; "))
            .filter_map(|cookie| cookie.split_once("="))
            .collect()
    }
    pub(crate) fn parse_url<'a>(req: &httparse::Request<'a, 'a>) -> Url {
        let host = req
            .headers
            .iter()
            .find_map(|h| match h.name.to_ascii_lowercase() == "host" {
                true => str::from_utf8(h.value).ok(),
                false => None,
            })
            .unwrap_or("localhost");
        req.path
            .map(|path| ::alloc::__export::must_use({
                ::alloc::fmt::format(format_args!("http://{1}{0}", path, host))
            }))
            .and_then(|url| Url::parse(&url).ok())
            .ok_or(crate::Error::RequestError)
            .unwrap()
    }
}
pub use parse::HttpMethod;
mod request {
    use std::ops::{Deref, DerefMut};
    use smol::io::AsyncReadExt;
    use url::Url;
    use crate::{Deserialize, Error, HttpMethod};
    pub type Readable<'a> = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send + 'a>>;
    /// An partially parsed Request with headers, cookies and path parameters.
    ///
    /// The request body is read lazily
    pub struct Request<'a, 'data: 'a> {
        pub headers: &'a [httparse::Header<'data>],
        pub url: &'a Url,
        pub cookies: &'a Vec<(&'data str, &'data str)>,
        pub path_params: &'a Vec<&'data str>,
        pub method: HttpMethod,
        pub version: u8,
        body: bool,
        readable: &'a mut Readable<'data>,
    }
    impl<'a, 'data: 'a> Request<'a, 'data> {
        pub async fn new(
            request: &'a httparse::Request<'data, 'data>,
            url: &'a Url,
            cookies: &'a Vec<(&'data str, &'data str)>,
            path_params: &'a Vec<&'data str>,
            readable: &'a mut Readable<'data>,
        ) -> Result<Self, Error> {
            Ok(Self {
                method: request.method.ok_or(Error::RequestError)?.into(),
                url,
                cookies,
                version: request.version.ok_or(Error::RequestError)?,
                headers: request.headers,
                readable,
                path_params,
                body: false,
            })
        }
        pub async fn body<B: Deserialize>(&mut self) -> Result<B, Error> {
            if self.body {
                return Err(Error::StateError);
            }
            self.body = true;
            let Some(content_length) = self
                .headers
                .iter()
                .find(|h| h.name.to_ascii_lowercase() == "content-length")
                .map(|h| h.value) else {
                return Err(Error::MissingContentLength);
            };
            let Some(bytes) = str::from_utf8(content_length)
                .ok()
                .and_then(|content_length| {
                    usize::from_str_radix(content_length, 10).ok()
                }) else {
                return Err(Error::InvalidContentLength);
            };
            fn body_reader<'a, 'b: 'a>(
                readable: &'a mut Readable<'b>,
                bytes: u64,
            ) -> Readable<'a> {
                readable.take(bytes).boxed_reader()
            }
            let readable = &mut body_reader(self.readable, bytes as u64);
            let res = B::deserialize(readable, bytes)
                .await
                .map_err(|e| Error::ParseError(e.to_string()));
            res
        }
    }
    impl<'data> Deref for Request<'_, 'data> {
        type Target = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send + 'data>>;
        fn deref(&self) -> &Self::Target {
            &self.readable
        }
    }
    impl<'data> DerefMut for Request<'_, 'data> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.readable
        }
    }
}
pub use request::Request;
mod response {
    use std::{marker::PhantomData, ops::{Deref, DerefMut}};
    use smol::io::AsyncWriteExt;
    use crate::{ContentType, Error, Serialize, response};
    pub(crate) enum ResponseState {
        Status,
        StaticHeaders,
        Header,
        Body,
        Done,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ResponseState {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ResponseState::Status => "Status",
                    ResponseState::StaticHeaders => "StaticHeaders",
                    ResponseState::Header => "Header",
                    ResponseState::Body => "Body",
                    ResponseState::Done => "Done",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for ResponseState {}
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for ResponseState {}
    #[automatically_derived]
    impl ::core::clone::Clone for ResponseState {
        #[inline]
        fn clone(&self) -> ResponseState {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for ResponseState {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for ResponseState {
        #[inline]
        fn eq(&self, other: &ResponseState) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for ResponseState {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for ResponseState {
        #[inline]
        fn partial_cmp(
            &self,
            other: &ResponseState,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for ResponseState {
        #[inline]
        fn cmp(&self, other: &ResponseState) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    pub type Writeable = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;
    pub struct Response<'a> {
        pub(crate) state: ResponseState,
        writeable: &'a mut Writeable,
        static_headers: &'static [(&'static str, &'static str)],
        once: bool,
    }
    impl Deref for Response<'_> {
        type Target = std::pin::Pin<Box<dyn smol::io::AsyncWrite + Send>>;
        fn deref(&self) -> &Self::Target {
            &self.writeable
        }
    }
    impl DerefMut for Response<'_> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.writeable
        }
    }
    pub trait RestResponse<T>
    where
        Self: Sized,
        T: ContentType,
    {
        const CODE: u16;
        const REASON: &'static str;
        const HEADERS: &'static [(&'static str, &'static str)];
    }
    impl<'a> Response<'a> {
        pub fn new(
            writeable: &'a mut Writeable,
            static_headers: &'static [(&'static str, &'static str)],
        ) -> Self {
            Self {
                state: ResponseState::Status,
                writeable,
                static_headers,
                once: false,
            }
        }
        pub async fn status(&mut self, code: u16, reason: &str) -> Result<(), Error> {
            if code < 100 || code >= 1000 || self.state != ResponseState::Status {
                return Err(Error::InvalidStatus);
            }
            let _ = self
                .writeable
                .write_all(
                    ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("HTTP/1.1 {0} {1}\r\n", code, reason),
                            )
                        })
                        .as_bytes(),
                )
                .await;
            self.state = ResponseState::StaticHeaders;
            Box::pin(self.headers(&[])).await?;
            Ok(())
        }
        pub async fn headers(&mut self, headers: &[(&str, &str)]) -> Result<(), Error> {
            if self.state >= ResponseState::Body || self.state == ResponseState::Status {
                return Err(Error::StateError);
            }
            if self.state == ResponseState::StaticHeaders {
                self.state = ResponseState::Header;
                Box::pin(self.headers(self.static_headers)).await?;
            }
            for (name, value) in headers {
                if name.to_ascii_lowercase() == "connection" && *value == "close" {
                    self.once = true;
                }
                let header = ::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("{0}: {1}\r\n", name, value))
                    })
                    .into_bytes();
                self.writeable
                    .write_all(&header)
                    .await
                    .map_err(|e| Error::WriteError(e.to_string()))?;
            }
            Ok(())
        }
        ///send arbitrary serializable data
        pub async fn send(&mut self, data: &impl Serialize) -> Result<(), Error> {
            if self.state != ResponseState::Header {
                return Err(Error::StateError);
            }
            let data = Serialize::serialize(data).map_err(|_| Error::SerializeError)?;
            self.writeable
                .write_all(
                    &::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Content-Length: {0}\r\n\r\n", data.len()),
                            )
                        })
                        .into_bytes(),
                )
                .await
                .map_err(|e| e.to_string())
                .map_err(Error::WriteError)?;
            self.writeable
                .write_all(&data)
                .await
                .map_err(|e| e.to_string())
                .map_err(Error::WriteError)?;
            let _ = self.writeable.flush().await;
            if self.once {
                let _ = self.writeable.close().await;
            }
            self.state = ResponseState::Done;
            Ok(())
        }
        ///Send a response as defined by the openapi rest spec
        pub async fn respond<C: ContentType>(
            &mut self,
            response: C,
        ) -> Result<(), Error> {
            self.status(C::Response::CODE, C::Response::REASON).await?;
            self.headers(C::Response::HEADERS).await?;
            self.headers(&[("Content-Type", C::CONTENT_TYPE)]).await?;
            self.send(&response).await?;
            Ok(())
        }
        pub async fn close(&mut self) {
            if self.state == ResponseState::Done {
                return;
            }
            let _ = self.status(500, "Internal Server Error").await;
        }
    }
}
pub use response::{Response, RestResponse};
mod serde {
    use crate::RestResponse;
    pub use crate::request::Readable as DeserializeStream;
    pub use smol::io::AsyncReadExt;
    mod json {
        use crate::{ContentType, DeserializeBuffered, RestResponse, Serialize};
        pub struct Json<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
        impl<T> DeserializeBuffered for Json<T>
        where
            T: serde::de::DeserializeOwned + serde::Serialize,
        {
            fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(Json(serde_json::from_slice(data)?))
            }
        }
        impl<T> Serialize for Json<T>
        where
            T: serde::de::DeserializeOwned + serde::Serialize,
        {
            fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                Ok(serde_json::to_vec(&data.0)?)
            }
        }
        impl<
            T: serde::de::DeserializeOwned + serde::Serialize + RestResponse<Json<T>>,
        > ContentType for Json<T> {
            const CONTENT_TYPE: &'static str = "application/json";
            type Response = T;
            fn new(val: T) -> Self {
                Self(val)
            }
        }
    }
    pub use json::*;
    mod xml {
        use crate::{ContentType, DeserializeBuffered, RestResponse, Serialize};
        pub struct XML<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
        impl<T> DeserializeBuffered for XML<T>
        where
            T: serde::de::DeserializeOwned + serde::Serialize,
        {
            fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
                Ok(XML(serde_xml_rs::from_reader(data)?))
            }
        }
        impl<T> Serialize for XML<T>
        where
            T: serde::de::DeserializeOwned + serde::Serialize,
        {
            fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
                let mut vec = Vec::new();
                serde_xml_rs::to_writer(&mut vec, &data.0)?;
                Ok(vec)
            }
        }
        impl<
            T: serde::de::DeserializeOwned + serde::Serialize + RestResponse<XML<T>>,
        > ContentType for XML<T> {
            const CONTENT_TYPE: &'static str = "application/xml";
            type Response = T;
            fn new(val: T) -> Self {
                Self(val)
            }
        }
    }
    pub use xml::*;
    /**# Deserialize

The resty::Deserialize trait provides a unified interface for deserialization from a Vec\<u8\>.
This allows free choice of serialization format and framework.

## Usage

An example using serde and serde_json

```rust
#[derive(serde::Deserialize, resty::Deserialize)]
#[deserializer(crate::deserialize)]
struct MyResponse {
  foo: String,
  bar: f64
};


fn deserialize<'a, T: serde::Deserialize<'a>>(
    data: &'a [u8],
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(data).inspect_err(|e| println!("{e:?}"))?)
}
```
*/
    pub trait Deserialize
    where
        Self: Sized,
    {
        fn deserialize<'a, 'b>(
            data: &'a mut DeserializeStream<'b>,
            bytes: usize,
        ) -> impl Future<Output = Result<Self, Box<dyn std::error::Error>>>;
    }
    pub trait DeserializeBuffered
    where
        Self: Sized,
    {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
    }
    impl<T: DeserializeBuffered> Deserialize for T {
        async fn deserialize<'a, 'b>(
            data: &'a mut DeserializeStream<'b>,
            bytes: usize,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            let buf = &mut ::alloc::vec::from_elem(0, bytes);
            data.read_exact(buf).await?;
            <Self as DeserializeBuffered>::deserialize(buf)
        }
    }
    #[doc(hidden)]
    pub trait Demueslify
    where
        Self: Sized,
    {
        fn demueslify(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
    }
    impl<T: Demueslify> DeserializeBuffered for T {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            Self::demueslify(data)
        }
    }
    /**# Serialize

The resty::Serialize trait provides a unified interface for serialization to a Vec\<u8\>.
This allows free choice of serialization format and framework.

## Usage

An example using serde and serde_json

```rust
#[derive(serde::Serialize, resty::Serialize)]
#[serializer(crate::serialize)]
struct MyResponse {
  foo: String,
  bar: f64
};


fn serialize<'a, T: serde::Serialize<'a>>(
    data: &'a [u8],
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::to_vec(&data)?)
}
```
*/
    pub trait Serialize {
        fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    }
    impl<T: Into<Vec<u8>> + Clone> Serialize for T {
        fn serialize(data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            Ok(T::into(data.clone()))
        }
    }
    pub trait ContentType: Serialize + Deserialize {
        const CONTENT_TYPE: &'static str;
        type Response: RestResponse<Self>;
        fn new(val: Self::Response) -> Self;
    }
    pub struct NoBody<T>(pub T);
    impl<T> Serialize for NoBody<T> {
        fn serialize(_: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            Ok(Vec::new())
        }
    }
    impl<T> Deserialize for NoBody<T> {
        async fn deserialize<'a, 'b>(
            _: &'a mut DeserializeStream<'b>,
            _: usize,
        ) -> Result<Self, Box<dyn std::error::Error>> {
            Err(crate::Error::UnTypedRequest)?
        }
    }
    impl<T: RestResponse<NoBody<T>>> ContentType for NoBody<T> {
        const CONTENT_TYPE: &'static str = "none";
        type Response = T;
        fn new(val: Self::Response) -> Self {
            Self(val)
        }
    }
}
pub use serde::*;
mod http_error {
    const NO_HEADERS: &'static [(&'static str, &'static str)] = &[];
}
mod routing {
    use std::collections::HashMap;
    use crate::{HttpMethod, Request, Response};
    /// Type alias for the dyn Trait a Handler function must have
    ///
    /// This is not generally used directly since the `#[endpoint]` macro wraps your
    /// async function to comply with this Trait
    type Handler = dyn for<'a, 'data, 'b> Fn(
        &'b mut Request<'a, 'data>,
        &'b mut Response<'a>,
    ) -> crate::EndpointTask<'b> + Sync;
    /// A router that routes a path to an endpoint while resolving path parameters
    ///
    /// Any route segment starting with `[` is considered a path parameter
    /// The route `%404` is used as fallback if no other route could be found
    pub struct Router {
        pub(crate) segments: HashMap<&'static str, Router>,
        pub(crate) endpoints: Vec<(&'static Handler, u16)>,
        pub(crate) middleware: Option<&'static Handler>,
    }
    impl std::fmt::Debug for Router {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{0}", self))
        }
    }
    impl std::fmt::Display for Router {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fn fmt_mthod(value: u16) -> Vec<HttpMethod> {
                let mut vec = Vec::with_capacity(16);
                let mut cur = 1u16;
                while cur != 0 && cur <= HttpMethod::TRACE as u16 {
                    if cur & value > 0 {
                        vec.push(unsafe { std::mem::transmute(cur) });
                    }
                    cur = cur << 1;
                }
                vec
            }
            f.write_fmt(format_args!("Router {{\n"))?;
            for (key, value) in &self.segments {
                let route = ::alloc::__export::must_use({
                    ::alloc::fmt::format(
                        format_args!(
                            "{0}: {1}\n",
                            ::alloc::__export::must_use({
                                    ::alloc::fmt::format(format_args!("/{0}", key))
                                })
                                .replace("/%", "%"),
                            value,
                        ),
                    )
                });
                let route: String = route
                    .lines()
                    .map(|line| ::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("  {0}\n", line))
                    }))
                    .collect();
                f.write_fmt(format_args!("{0}", route))?;
            }
            for method in fmt_mthod(self.endpoints.iter().fold(0, |a, (.., b)| a | b)) {
                f.write_fmt(format_args!("  {0}\n", method))?;
            }
            f.write_fmt(
                format_args!(
                    "  {0}\n",
                    self
                        .middleware
                        .map(|_| "Middleware: Yes")
                        .unwrap_or("Middleware: No"),
                ),
            )?;
            f.write_fmt(format_args!("}}"))
        }
    }
    impl Router {
        pub fn empty() -> Self {
            Self {
                segments: HashMap::new(),
                endpoints: Vec::new(),
                middleware: None,
            }
        }
        pub fn new(route_slices: &[RouteSlice]) -> Self {
            let mut route_table = Router::empty();
            for slice in route_slices {
                route_table.add_route(slice)
            }
            return route_table;
        }
        pub fn add_route(&mut self, (route, handler_or_middleware): &RouteSlice) {
            let mut current_router = self;
            for current_segment in *route {
                let Router { segments, .. } = current_router;
                let key = match current_segment.chars().nth(0).map(|c| c == '[') {
                    Some(true) => "%param",
                    _ => current_segment,
                };
                current_router = segments.entry(key).or_insert_with(Router::empty);
            }
            match handler_or_middleware {
                HandlerOrMiddleware::Handler(method, handler) => {
                    current_router.endpoints.push((*method, *handler))
                }
                HandlerOrMiddleware::Middleware(middleware) => {
                    current_router.middleware.replace(*middleware);
                }
            }
        }
        pub fn route<'a>(
            &'a self,
            path: &'a str,
        ) -> Option<(&'a Router, Vec<&'a str>, Vec<&'static Handler>)> {
            let mut path_parameters = ::alloc::vec::Vec::new();
            let mut middlewares = ::alloc::vec::Vec::new();
            let mut segments = path
                .strip_prefix("/")
                .unwrap_or(path)
                .split("?")
                .take(1)
                .last()
                .unwrap_or("")
                .split("/");
            let mut route = self;
            route.middleware.inspect(|middleware| middlewares.push(*middleware));
            while let Some(current_segment) = segments.next() {
                if current_segment == "" {
                    continue;
                }
                let dynamic = || {
                    route
                        .segments
                        .get("%param")
                        .inspect(|_| path_parameters.push(current_segment))
                };
                let Some(next_route) = route
                    .segments
                    .get(current_segment)
                    .or_else(dynamic)
                    .or_else(|| self.segments.get("%404")) else {
                    return None;
                };
                next_route
                    .middleware
                    .inspect(|middleware| middlewares.push(*middleware));
                route = next_route;
            }
            Some((route, path_parameters, middlewares))
        }
        pub fn handler<'a>(
            &'a self,
            path: &'a str,
            method: HttpMethod,
        ) -> Option<(Vec<&'static Handler>, Vec<&'a str>)> {
            let (route, params, mut middlewares) = self.route(path)?;
            let handler = route.method(method)?;
            middlewares.push(handler);
            Some((middlewares, params))
        }
        pub fn method(&self, method: HttpMethod) -> Option<&'static Handler> {
            self.endpoints
                .iter()
                .find_map(|(handler, mask)| {
                    match method as u16 & mask > 0 || *mask == HttpMethod::ALL as u16 {
                        true => Some(*handler),
                        false => None,
                    }
                })
        }
    }
    pub enum HandlerOrMiddleware {
        Handler(&'static Handler, u16),
        Middleware(&'static Handler),
    }
    /// Type alias for the description of a Route
    pub type RouteSlice = (&'static [&'static str], HandlerOrMiddleware);
}
pub use routing::HandlerOrMiddleware::*;
pub use routing::*;
mod runtime {
    use smol::io::{AsyncReadExt, AsyncWriteExt};
    use crate::{
        HttpMethod, Router, Socket, parse::{parse_cookies, parse_url},
        response::ResponseState,
    };
    /// Type alias for the Future returned by a Handler
    pub type EndpointTask<'a> = std::pin::Pin<
        Box<dyn Future<Output = Result<(), crate::Error>> + 'a + Send>,
    >;
    static EXECUTOR: smol::Executor = smol::Executor::new();
    #[inline(always)]
    pub fn bind<S: Socket + 'static>(
        addr: S::Address,
        router: &'static Router,
    ) -> Result<(), S::Error> {
        let connector = smol::block_on(S::bind(addr))?;
        EXECUTOR.spawn(accept_connections(connector, router)).detach();
        Ok(())
    }
    /// Spawns a worker thread to handle the async task queue.
    ///
    /// At least one worker thread must be spawned
    #[allow(unreachable_code)]
    #[inline(always)]
    pub fn spawn_thread() -> std::thread::JoinHandle<std::convert::Infallible> {
        std::thread::spawn(|| {
            smol::block_on(async {
                loop {
                    EXECUTOR.tick().await;
                }
            })
        })
    }
    async fn accept_connections<S: Socket + 'static>(
        connector: Box<S>,
        router: &'static Router,
    ) -> () {
        loop {
            let Ok(stream) = connector.accept().await else {
                continue;
            };
            EXECUTOR.spawn(handle_stream::<S>(stream, router)).detach();
        }
    }
    async fn handle_stream<S: Socket + 'static>(
        mut stream: S::Stream,
        router: &Router,
    ) -> () {
        loop {
            if let Err(body) = handle_request::<S>(&mut stream, router).await {
                let _ = stream.write_all(body).await;
                return;
            }
        }
    }
    async fn handle_request<S: Socket + 'static>(
        stream: &mut S::Stream,
        router: &Router,
    ) -> Result<(), &'static [u8]> {
        let buffer = &mut Vec::with_capacity(4000);
        let Some(header_count) = crate::parse::read_headers::<S>(stream, buffer).await
        else {
            return Err(
                "HTTP/1.1 400 Bad Request\r\nContent-Length:0\r\nConnection:close\r\n\r\n"
                    .as_bytes(),
            );
        };
        buffer.shrink_to_fit();
        let headers = &mut ::alloc::vec::from_elem(httparse::EMPTY_HEADER, header_count)
            .into_boxed_slice();
        let mut request = httparse::Request::new(headers);
        if request.parse(buffer).is_err() {
            return Err(
                "HTTP/1.1 400 Bad Request\r\nContent-Length:0\r\nConnection:close\r\n\r\n"
                    .as_bytes(),
            );
        }
        let Some((handlers, path_params)) = request
            .path
            .zip(request.method.map(Into::<HttpMethod>::into))
            .and_then(|(route, method)| router.handler(route, method)) else {
            return Err(
                "HTTP/1.1 404 Not Found\r\nContent-Length:0\r\nConnection:close\r\n\r\n"
                    .as_bytes(),
            );
        };
        let content_length = request
            .headers
            .iter()
            .find_map(|h| {
                match h.name.to_ascii_lowercase() == "content-length" {
                    true => u64::from_str_radix(str::from_utf8(h.value).ok()?, 10).ok(),
                    false => None,
                }
            });
        let readable = stream.clone();
        let readable = &mut match content_length {
            Some(value) => readable.take(value).boxed_reader(),
            None => readable.boxed_reader(),
        };
        let url = parse_url(&request);
        let cookies = parse_cookies(&request);
        let mut request = crate::Request::new(
                &request,
                &url,
                &cookies,
                &path_params,
                readable,
            )
            .await
            .map_err(|_| {
                "HTTP/1.1 500 Internal Server Error\r\nContent-Length:0\r\nConnection:close\r\n\r\n"
                    .as_bytes()
            })?;
        let writeable = &mut stream.clone().boxed_writer();
        let mut response = crate::Response::new(writeable, &[]);
        for handler in handlers {
            if let Err(..) = handler(&mut request, &mut response).await {
                return Err(
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length:0\r\nConnection:close\r\n\r\n"
                        .as_bytes(),
                );
            }
            if response.state == ResponseState::Done {
                break;
            }
        }
        response.close().await;
        if let Some(length) = content_length && let Ok(1) = readable.read(&mut [0]).await
        {
            let _ = readable
                .read_to_end(&mut ::alloc::vec::from_elem(0, length as usize - 1))
                .await;
        }
        Ok(())
    }
}
pub use runtime::*;
mod socket {
    use smol::io::{AsyncRead, AsyncWrite};
    mod tcp {
        use std::{marker::PhantomData, net::SocketAddrV4};
        use smol::net::{AsyncToSocketAddrs, TcpListener, TcpStream};
        pub struct TcpScocket<A = SocketAddrV4>(TcpListener, PhantomData<A>);
        impl<A: AsyncToSocketAddrs + Send + Sync> super::Socket for TcpScocket<A> {
            type Error = smol::io::Error;
            type Address = A;
            type Stream = TcpStream;
            async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
                TcpListener::bind(addr)
                    .await
                    .map(|listener| Box::new(TcpScocket(listener, PhantomData)))
            }
            async fn accept(&self) -> Result<Self::Stream, Self::Error> {
                let (stream, ..) = self.0.accept().await?;
                stream.set_nodelay(true)?;
                Ok(stream)
            }
        }
    }
    pub use tcp::TcpScocket;
    mod unix {
        use std::{marker::PhantomData, path::Path};
        use smol::net::unix::{UnixListener, UnixStream};
        pub struct UnixSocket<P = &'static str>(UnixListener, PhantomData<P>);
        impl<P: AsRef<Path> + Send + Sync> super::Socket for UnixSocket<P> {
            type Error = smol::io::Error;
            type Address = P;
            type Stream = UnixStream;
            async fn bind(addr: Self::Address) -> Result<Box<Self>, Self::Error> {
                UnixListener::bind(addr)
                    .map(|listener| Box::new(UnixSocket(listener, PhantomData)))
            }
            async fn accept(&self) -> Result<Self::Stream, Self::Error> {
                let (stream, ..) = self.0.accept().await?;
                Ok(stream)
            }
        }
    }
    pub use unix::UnixSocket;
    /// The Socket trait allows for implementation of custom transport layers
    pub trait Socket: Send {
        type Error;
        type Address;
        type Stream: AsyncRead + AsyncWrite + Unpin + Clone + Send;
        fn bind(
            addr: Self::Address,
        ) -> impl std::future::Future<Output = Result<Box<Self>, Self::Error>>;
        fn accept(
            &self,
        ) -> impl std::future::Future<Output = Result<Self::Stream, Self::Error>> + Send;
    }
}
pub use socket::*;
pub enum Error {
    SerializeError,
    WriteError(String),
    StateError,
    InvalidStatus,
    MissingContentLength,
    InvalidContentLength,
    ReadError,
    ParseError(String),
    RequestError,
    UnTypedRequest,
}
#[automatically_derived]
impl ::core::fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Error::SerializeError => {
                ::core::fmt::Formatter::write_str(f, "SerializeError")
            }
            Error::WriteError(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "WriteError",
                    &__self_0,
                )
            }
            Error::StateError => ::core::fmt::Formatter::write_str(f, "StateError"),
            Error::InvalidStatus => ::core::fmt::Formatter::write_str(f, "InvalidStatus"),
            Error::MissingContentLength => {
                ::core::fmt::Formatter::write_str(f, "MissingContentLength")
            }
            Error::InvalidContentLength => {
                ::core::fmt::Formatter::write_str(f, "InvalidContentLength")
            }
            Error::ReadError => ::core::fmt::Formatter::write_str(f, "ReadError"),
            Error::ParseError(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "ParseError",
                    &__self_0,
                )
            }
            Error::RequestError => ::core::fmt::Formatter::write_str(f, "RequestError"),
            Error::UnTypedRequest => {
                ::core::fmt::Formatter::write_str(f, "UnTypedRequest")
            }
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Error {
    #[inline]
    fn clone(&self) -> Error {
        match self {
            Error::SerializeError => Error::SerializeError,
            Error::WriteError(__self_0) => {
                Error::WriteError(::core::clone::Clone::clone(__self_0))
            }
            Error::StateError => Error::StateError,
            Error::InvalidStatus => Error::InvalidStatus,
            Error::MissingContentLength => Error::MissingContentLength,
            Error::InvalidContentLength => Error::InvalidContentLength,
            Error::ReadError => Error::ReadError,
            Error::ParseError(__self_0) => {
                Error::ParseError(::core::clone::Clone::clone(__self_0))
            }
            Error::RequestError => Error::RequestError,
            Error::UnTypedRequest => Error::UnTypedRequest,
        }
    }
}
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingContentLength => {
                f.write_fmt(format_args!("MissingContentLength"))
            }
            Error::InvalidContentLength => {
                f.write_fmt(format_args!("InvalidContentLength"))
            }
            Error::ReadError => f.write_fmt(format_args!("ReadError")),
            Error::ParseError(e) => f.write_fmt(format_args!("ParseError({0})", e)),
            Error::UnTypedRequest => f.write_fmt(format_args!("UnTypedRequest")),
            Error::SerializeError => f.write_fmt(format_args!("SerializeError")),
            Error::WriteError(e) => f.write_fmt(format_args!("WriteError({0})", e)),
            Error::StateError => f.write_fmt(format_args!("StateError")),
            Error::InvalidStatus => f.write_fmt(format_args!("InvalidStatus")),
            Error::RequestError => f.write_fmt(format_args!("RequestError")),
        }
    }
}
pub type Result = std::result::Result<(), Error>;
#[doc(hidden)]
pub mod __private {
    pub use linkme;
    /// This trait is a marker for Schemas. Do not implement manually, use the Schema derive macro
    pub trait Schema {}
}
