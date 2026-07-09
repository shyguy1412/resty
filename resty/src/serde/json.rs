use crate::{DeserializeBuffered, Serialize};

pub struct Json<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
impl<T> DeserializeBuffered for Json<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Json(serde_json::from_slice(data)?))
    }
}

impl<T> Serialize for Json<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(&self.0)?)
    }
}
