pub use crate::request::Readable as DeserializeStream;
pub use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::io::Write;

#[cfg(all(feature = "serde", feature = "json"))]
mod json;
#[cfg(all(feature = "serde", feature = "json"))]
pub use json::*;
#[cfg(all(feature = "serde", feature = "xml"))]
mod xml;
#[cfg(all(feature = "serde", feature = "xml"))]
pub use xml::*;

// A trait to Deserialize Self from a stream of bytes. This is not usually implemented manually.
pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize<'a, 'b>(
        data: &'a mut DeserializeStream<'b>,
        bytes: usize,
    ) -> impl Future<Output = Result<Self, Box<dyn std::error::Error>>>;
}

/// The main trait to deserialize Self from a request.
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
        let buf = &mut vec![0; bytes];
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

//The main trait to serialize Self into a `Vec<u8>`
pub trait Serialize {
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(data.clone()))
    }
}

/// A content type for responses that have no body. Mostly used for generic HTTP Errors
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

/// A struct that serializes into a single message for a server sent event
/// Server sent events can be implemented by streaming an iterator over these
pub struct ServerSentEvent<B: Serialize> {
    pub body: B,
    pub event: String,
}

impl<B: Serialize> Serialize for ServerSentEvent<B> {
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let body = Serialize::serialize(&data.body)?;
        let event = &data.event;

        let mut result = Vec::with_capacity(0);
        result.reserve_exact(body.len() + event.as_bytes().len() + b"event: \ndata: \n\n".len());

        write!(result, "event: {event}\ndata: ")?;
        std::io::Write::write_all(&mut result, &body)?;
        write!(result, "\n\n")?;
        Ok(result)
    }
}

/// Implementing ContentType lets the response infer the content type header
/// this is  implemented by an extractor like Json or XML
/// The content type trait makes no assumption about how and if a given T can be serialized
pub trait ContentType<T>: Serialize + Deserialize {
    const CONTENT_TYPE: &'static str;
}

/// A REST API response as defined by the openapi spec.
/// By deriving this trait resty can auto generate the openapi spec for this response
pub trait RestResponse
where
    Self: Sized,
{
    const CODE: u16;
    const REASON: &'static str;
    const HEADERS: &'static [(&'static str, &'static str)];
}

impl<T: RestResponse> ContentType<T> for NoBody<T> {
    const CONTENT_TYPE: &'static str = "none";
}

impl<R: RestResponse> RestResponse for Vec<R> {
    const CODE: u16 = R::CODE;
    const REASON: &'static str = R::REASON;
    const HEADERS: &'static [(&'static str, &'static str)] = R::HEADERS;
}

/// This trait reads a Self from the request stream.
/// This can be used to read something like a new line delimited JSON stream sequentially
pub trait Parse
where
    Self: Sized,
{
    type Error: std::error::Error;
    fn parse(stream: &mut DeserializeStream) -> Result<Self, Self::Error>;
}
