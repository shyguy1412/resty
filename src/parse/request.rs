use std::{
    ops::{Deref, DerefMut},
    str::from_utf8,
};

use smol::io::AsyncReadExt;
use url::Url;

use crate::parse::parse_cookies;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    MissingContentLength,
    InvalidContentLength,
    ReadError,
    ParseError,
    UnTypedRequest,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingContentLength => write!(f, "MissingContentLength"),
            Error::InvalidContentLength => write!(f, "InvalidContentLength"),
            Error::ReadError => write!(f, "ReadError"),
            Error::ParseError => write!(f, "ParseError"),
            Error::UnTypedRequest => write!(f, "UnTypedRequest"),
        }
    }
}

pub type Readable = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

/// An partially parsed Request with headers, cookies and path parameters.  
///
/// The request body is read lazily
pub struct Request<'a, B = ()> {
    pub headers: &'a [httparse::Header<'a>],
    pub url: Url,
    pub cookies: Vec<(&'a str, &'a str)>,
    pub path_params: &'a Vec<&'a str>,
    pub method: &'a str,
    pub version: u8,
    body: Option<Box<Result<B, Error>>>,
    readable: &'a mut Readable,
}

impl<'a, B: Deserialize> Request<'a, B> {
    pub async fn new(
        request: &'a httparse::Request<'a, 'a>,
        path_params: &'a Vec<&'a str>,
        readable: &'a mut Readable,
    ) -> Option<Self> {
        let host = request
            .headers
            .iter()
            .find_map(|httparse::Header { name, value }| {
                match name.to_ascii_lowercase() == "host" {
                    true => from_utf8(value).ok(),
                    false => None,
                }
            })
            .unwrap_or("localhost");

        let url = Url::parse(&format!("http://{host}{}", request.path?)).ok()?;

        let cookies = parse_cookies(request.headers);

        Some(Self {
            method: request.method?,
            url,
            cookies,
            version: request.version?,
            headers: request.headers,
            readable,
            path_params,
            body: None,
        })
    }

    pub async fn body(&mut self) -> &Result<B, Error> {
        if let Some(ref body) = self.body {
            return body.as_ref();
        }

        let Some(content_length) = self
            .headers
            .iter()
            .find(|h| h.name.to_ascii_lowercase() == "content-length")
            .map(|h| h.value)
        else {
            return &Err(Error::MissingContentLength);
        };

        let Some(bytes) = str::from_utf8(content_length)
            .ok()
            .and_then(|content_length| usize::from_str_radix(content_length, 10).ok())
        else {
            return &Err(Error::InvalidContentLength);
        };

        let mut vec = vec![0; bytes];

        let _ = self.read_exact(&mut vec).await;

        let body = Box::new(B::deserialize(&vec).map_err(|_| Error::ParseError));

        self.body.replace(body);

        match self.body {
            Some(ref b) => b,
            None => unreachable!(),
        }
    }
}

impl<B> Deref for Request<'_, B> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.readable
    }
}

impl<B> DerefMut for Request<'_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.readable
    }
}

#[doc = include_str!("../../macros/docs/traits/Deserialize.md")]
pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
}

impl Deserialize for () {
    fn deserialize(_: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Err(Error::UnTypedRequest)?
    }
}
