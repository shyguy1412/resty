use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Server {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub variables: BTreeMap<String, ServerVariable>,
}

#[derive(Serialize)]
pub struct ServerVariable {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    options: Vec<String>,
    default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}
