use crate::{ContentType, DeserializeBuffered, RestResponse, Serialize};

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
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(&data.0)?)
    }
}

impl<T: serde::de::DeserializeOwned + serde::Serialize + RestResponse> ContentType<T> for Json<T> {
    const CONTENT_TYPE: &'static str = "application/json";
}
