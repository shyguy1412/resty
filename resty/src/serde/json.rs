use crate::{ContentType, DeserializeBuffered, RestResponse, Serialize};

/// An extractor and content type for sending and receiving JSON
pub struct Json<T>(pub T);
impl<T> DeserializeBuffered for Json<T>
where
    T: serde::de::DeserializeOwned,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Json(serde_json::from_slice(data)?))
    }
}

impl<T> Serialize for Json<T>
where
    T: serde::Serialize,
{
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(serde_json::to_vec(&data.0)?)
    }
}

impl<T: serde::Serialize + RestResponse> ContentType<&'_ T> for Json<&'_ T> {
    const CONTENT_TYPE: &'static str = "application/json";
}
