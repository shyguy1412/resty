use std::ops::{Deref, DerefMut};

use smol::io::AsyncReadExt;
use url::Url;

use crate::{Error, HttpMethod};

pub type Readable = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

/// An partially parsed Request with headers, cookies and path parameters.  
///
/// The request body is read lazily
pub struct Request<'a> {
    pub headers: &'a [httparse::Header<'a>],
    pub url: &'a Url,
    pub cookies: &'a Vec<(&'a str, &'a str)>,
    pub path_params: &'a Vec<&'a str>,
    pub method: HttpMethod,
    pub version: u8,
    body: Option<Result<Box<[u8]>, Error>>,
    readable: &'a mut Readable,
}

impl<'a> Request<'a> {
    pub async fn new(
        request: &'a httparse::Request<'a, 'a>,
        url: &'a Url,
        cookies: &'a Vec<(&'a str, &'a str)>,
        path_params: &'a Vec<&'a str>,
        readable: &'a mut Readable,
    ) -> Result<Self, Error> {
        Ok(Self {
            method: request.method.ok_or(Error::RequestError)?.into(),
            url,
            cookies,
            version: request.version.ok_or(Error::RequestError)?,
            headers: request.headers,
            readable,
            path_params,
            body: None,
        })
    }

    pub async fn body<B: Deserialize>(&mut self) -> Result<B, Error> {
        if let Some(ref body) = self.body {
            return body.as_ref().map_err(|e| e.clone()).and_then(|body| {
                B::deserialize(&body).map_err(|e| Error::ParseError(e.to_string()))
            });
        }

        let Some(content_length) = self
            .headers
            .iter()
            .find(|h| h.name.to_ascii_lowercase() == "content-length")
            .map(|h| h.value)
        else {
            self.body.replace(Err(Error::MissingContentLength));
            return Err(Error::MissingContentLength);
        };

        let Some(bytes) = str::from_utf8(content_length)
            .ok()
            .and_then(|content_length| usize::from_str_radix(content_length, 10).ok())
        else {
            self.body.replace(Err(Error::InvalidContentLength));
            return Err(Error::InvalidContentLength);
        };

        let mut vec = vec![0; bytes].into_boxed_slice();

        let _ = self.read_exact(&mut vec).await;

        let res = B::deserialize(&vec).map_err(|e| Error::ParseError(e.to_string()));

        self.body.replace(Ok(vec));

        res
    }
}

impl Deref for Request<'_> {
    type Target = std::pin::Pin<Box<dyn smol::io::AsyncRead + Send>>;

    fn deref(&self) -> &Self::Target {
        &self.readable
    }
}

impl DerefMut for Request<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.readable
    }
}

// impl<'a, 'b: 'a> Request<'b> {
//     pub fn as_typed(&'a mut self) -> Request<'a> {
//         Request {
//             headers: self.headers,
//             url: self.url,
//             cookies: self.cookies,
//             path_params: self.path_params,
//             method: self.method,
//             version: self.version,
//             readable: self.readable,
//             body: None,
//         }
//     }
// }

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
