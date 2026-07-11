use std::collections::BTreeMap;

use super::OrRef;
use serde::Serialize;

#[derive(Serialize)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(flatten)]
    pub schema: EnumOrStruct,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum EnumOrStruct {
    Enum(EnumSpec),
    Struct(StructSpec),
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "object")]
pub struct StructSpec {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, OrRef<Property>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

#[derive(Serialize)]
pub struct Property {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub ty: PropertyType,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PropertyType {
    Primitive {
        #[serde(rename = "type")]
        ty: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
    },
    Array {
        items: Box<OrRef<PropertyType>>,
    },
}

#[derive(Serialize)]
pub struct EnumSpec {
    #[serde(rename = "type")]
    pub ty: String,

    #[serde(rename = "enum")]
    pub variants: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}
