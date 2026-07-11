use std::collections::BTreeMap;

use super::OrRef;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,

    #[serde(flatten)]
    pub ty: SchemaType,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum SchemaType {
    Enum(EnumSchema),
    Struct(StructSchema),
    Array(ArraySchema),
    Primitive(PrimitiveSchema),
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename = "array")]
pub struct ArraySchema {
    pub items: Box<OrRef<SchemaType>>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename = "object")]
pub struct StructSchema {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, OrRef<Schema>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct EnumSchema {
    #[serde(rename = "type")]
    pub ty: String,

    #[serde(rename = "enum")]
    pub variants: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct PrimitiveSchema {
    #[serde(rename = "type")]
    pub ty: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}
