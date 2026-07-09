use crate::RestResponse;
pub use crate::request::Readable as DeserializeStream;
pub use smol::io::AsyncReadExt;

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
