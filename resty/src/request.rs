use smol::io::AsyncReadExt;
use url::Url;

use crate::{
    Deserialize,
    Error::{self, ParseError},
    HttpMethod, Parse,
};

pub type Readable<'a> = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send + 'a>>;

/// An partially parsed Request with headers, cookies and path parameters.  
///
/// The request body is read lazily on demand. Reading the request body consumes it from the stream.
/// Trying to read a request body twice will return a StateError
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
            .map(|h| h.value)
        else {
            return Err(Error::MissingContentLength);
        };

        let Some(bytes) = str::from_utf8(content_length)
            .ok()
            .and_then(|content_length| usize::from_str_radix(content_length, 10).ok())
        else {
            return Err(Error::InvalidContentLength);
        };

        fn body_reader<'a, 'b: 'a>(readable: &'a mut Readable<'b>, bytes: u64) -> Readable<'a> {
            readable.take(bytes).boxed_reader()
        }

        let readable = &mut body_reader(self.readable, bytes as u64);

        let res = B::deserialize(readable, bytes)
            .await
            .map_err(|e| Error::ParseError(e.to_string()));

        res
    }

    /// Parse some value out of the request body.
    /// Useful for requests that stream multiple elements like a newline separated list of JSON objects
    pub async fn parse<P: Parse>(&mut self) -> Result<P, Error> {
        P::parse(self.readable).map_err(|e| ParseError(e.to_string()))
    }

    /// Provides access to the raw byte stream of a request.
    /// Calling this function will cause future calls to `body()` to fail
    pub fn raw_stream<'s>(&'s mut self) -> &'s mut Readable<'data> {
        self.body = true;
        &mut self.readable
    }
}
