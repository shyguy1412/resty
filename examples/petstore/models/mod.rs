mod api_response;
pub use api_response::*;
mod category;
pub use category::*;
mod order;
pub use order::*;
mod pet;
pub use pet::*;
mod tag;
pub use tag::*;
mod user;
pub use user::*;

pub struct Json<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
impl<T> resty::Deserialize for Json<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Json(serde_json::from_slice(data)?))
    }
}

impl<T> resty::Serialize for Json<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(&self.0)?)
    }
}

pub struct XML<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
impl<T> resty::Deserialize for XML<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(XML(serde_xml_rs::from_reader(data)?))
    }
}

impl<T> resty::Serialize for XML<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut vec = Vec::new();
        serde_xml_rs::to_writer(&mut vec, &self.0)?;
        Ok(vec)
    }
}
