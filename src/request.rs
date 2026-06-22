use std::ops::{Deref, DerefMut};

use smol::io::AsyncReadExt;
use url::Url;

use crate::{Error, HttpMethod};

pub type Readable = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

/// An partially parsed Request with headers, cookies and path parameters.  
///
/// The request body is read lazily
pub struct Request<'a, B = ()> {
    pub headers: &'a [httparse::Header<'a>],
    pub url: &'a Url,
    pub cookies: &'a Vec<(&'a str, &'a str)>,
    pub path_params: &'a Vec<&'a str>,
    pub method: HttpMethod,
    pub version: u8,
    body: Option<Box<Result<B, Error>>>,
    readable: &'a mut Readable,
}

impl<'a, B: Deserialize> Request<'a, B> {
    pub async fn new(
        request: &'a httparse::Request<'a, 'a>,
        url: &'a Url,
        cookies: &'a Vec<(&'a str, &'a str)>,
        path_params: &'a Vec<&'a str>,
        readable: &'a mut Readable,
    ) -> Result<Self, Error> {
        Ok(Self {
            method: request.method.ok_or(Error::ParseError)?.into(),
            url,
            cookies,
            version: request.version.ok_or(Error::ParseError)?,
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

impl<'a, 'b: 'a> Request<'b> {
    pub fn as_typed<B>(&'a mut self) -> Request<'a, B> {
        Request {
            headers: self.headers,
            url: self.url,
            cookies: self.cookies,
            path_params: self.path_params,
            method: self.method,
            version: self.version,
            readable: self.readable,
            body: None,
        }
    }
}

#[doc = include_str!("../docs/traits/Deserialize.md")]
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
