use crate::{DeserializeBuffered, Serialize};

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
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut vec = Vec::new();
        serde_xml_rs::to_writer(&mut vec, &self.0)?;
        Ok(vec)
    }
}
