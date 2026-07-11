use std::collections::BTreeMap;

use serde::Serialize;

use crate::spec::definition::{ContentReference, OrRef};

#[derive(Serialize)]
pub struct PathItem {
    #[serde(flatten)]
    pub operations: BTreeMap<String, OperationObject>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationObject {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub operation_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    pub responses: BTreeMap<String, OrRef<ContentlessResponse>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub is_in: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub explode: bool,
    pub schema: String,
}

#[derive(Serialize, Clone)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub content: BTreeMap<String, ContentReference>,
    pub required: bool,
}

#[derive(Serialize, Clone)]
pub struct ContentlessResponse {
    pub description: String,
}
