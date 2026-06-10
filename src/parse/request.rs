use std::ops::{Deref, DerefMut};

use smol::io::AsyncReadExt;

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

pub struct Request<'a, B = ()> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: u8,
    pub headers: &'a mut [httparse::Header<'a>],
    pub body: Result<Box<B>, Error>,
    readable: std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>,
}

impl<'a, B: Deserialize> Request<'a, B> {
    pub async fn new(
        request: httparse::Request<'a, 'a>,
        mut stream: smol::net::TcpStream,
    ) -> Option<Self> {
        let Some(data) = request
            .method
            .zip(request.path)
            .zip(request.version)
            .map(|((a, b), c)| (a, b, c))
        else {
            return None;
        };

        let body = body(&request, &mut stream).await;

        Some(Self {
            method: data.0,
            path: data.1,
            version: data.2,
            headers: request.headers,
            readable: stream.boxed_reader(),
            body,
        })
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

pub async fn body<B: Deserialize>(
    req: &httparse::Request<'_, '_>,
    stream: &mut smol::net::TcpStream,
) -> Result<Box<B>, Error> {
    let Some(content_length) = req
        .headers
        .iter()
        .find(|h| h.name == "Content-Length")
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

    let mut vec = vec![0; bytes];

    let _ = stream.read_exact(&mut vec).await;

    B::deserialize(&vec)
        .map_err(|_| Error::ParseError)
        .map(Box::new)
}

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
