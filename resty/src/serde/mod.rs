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

#[doc = include_str!("../../docs/traits/Deserialize.md")]
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

#[doc = include_str!("../../docs/traits/Serialize.md")]
pub trait Serialize {
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(data.clone()))
    }
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

pub struct ServerSentEvent<B: Serialize> {
    pub body: B,
    pub event: String,
}

impl<B: Serialize> Serialize for ServerSentEvent<B> {
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let body = Serialize::serialize(&data.body)?;
        let event = &data.event;

        //100 bytes should be a good extra buffer for regular usage
        let mut result = Vec::with_capacity(body.len() * 100);

        write!(result, "event: {event}\ndata: ")?;
        std::io::Write::write_all(&mut result, &body)?;
        write!(result, "\n\n")?;
        Ok(result)
    }
}

pub trait ContentType<T>: Serialize + Deserialize {
    const CONTENT_TYPE: &'static str;
}

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

pub trait Parse
where
    Self: Sized,
{
    type Error: std::error::Error;
    fn parse(stream: &mut DeserializeStream) -> Result<Self, Self::Error>;
}
