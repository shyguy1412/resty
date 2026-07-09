pub use crate::request::Readable as DeserializeStream;
pub use smol::io::AsyncReadExt;

mod json;
pub use json::*;
mod xml;
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
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl<T: Into<Vec<u8>> + Clone> Serialize for T {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(T::into(self.clone()))
    }
}
