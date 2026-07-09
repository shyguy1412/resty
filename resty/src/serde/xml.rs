use crate::{ContentType, DeserializeBuffered, RestResponse, Serialize};

pub struct XML<T: serde::de::DeserializeOwned + serde::Serialize>(pub T);
impl<T> DeserializeBuffered for XML<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(XML(serde_xml_rs::from_reader(data)?))
    }
}

impl<T> Serialize for XML<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    fn serialize(data: &Self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut vec = Vec::new();
        serde_xml_rs::to_writer(&mut vec, &data.0)?;
        Ok(vec)
    }
}

impl<T: serde::de::DeserializeOwned + serde::Serialize + RestResponse<XML<T>>> ContentType
    for XML<T>
{
    const CONTENT_TYPE: &'static str = "application/xml";
    type Response = T;

    fn new(val: T) -> Self {
        Self(val)
    }
}
