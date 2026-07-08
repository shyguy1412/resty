use std::ops::{Deref, DerefMut};

use smol::io::AsyncReadExt;
use url::Url;

use crate::{Error, HttpMethod};

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
// pub trait Deserialize
// where
//     Self: Sized,
// {
//     fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
// }

// impl Deserialize for () {
//     fn deserialize(_: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
//         Err(Error::UnTypedRequest)?
//     }
// }
// pub trait Deserialize
// where
//     Self: Sized,
// {
//     fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>>;
// }

pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize<'a, 'b>(
        data: &'a mut Readable<'b>,
        bytes: usize,
    ) -> impl Future<Output = Result<Self, Box<dyn std::error::Error>>>;
}

// impl Deserialize for () {
//     async fn deserialize<'a, 'b>(
//         _: &'a mut Readable<'b>,
//     ) -> Result<Self, Box<dyn std::error::Error>> {
//         Err(Error::UnTypedRequest)?
//     }
// }
