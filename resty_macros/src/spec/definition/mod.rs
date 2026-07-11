mod info;
pub use info::*;

mod server;
pub use server::*;

mod path;
pub use path::*;

mod schema;
pub use schema::*;

mod security;
pub use security::*;

use std::collections::BTreeMap;

use serde::{Serialize, ser::SerializeStruct};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Specification {
    pub openapi: OpenAPIVersion,
    pub info: Info,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub paths: BTreeMap<String, PathItem>,

    pub components: Components,
    // pub security: (),
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Serialize, Default)]
pub enum OpenAPIVersion {
    #[default]
    #[serde(rename = "3.0.4")]
    V304,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Serialize, Default)]
pub struct ExternalDocs {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Components {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub schemas: BTreeMap<String, Schema>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub security_schemes: BTreeMap<String, SecurityScheme>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub responses: BTreeMap<String, Response>,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum OrRef<T> {
    Ref(ReferenceObject),
    Val(T),
}

#[derive(Clone)]
pub struct ReferenceObject {
    pub component: ComponentType,
    pub name: String,
}

#[derive(Clone)]
pub enum ComponentType {
    Response,
    Schema,
}

impl Serialize for ReferenceObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut object = serializer.serialize_struct("ReferenceObject", 1)?;

        let component = match self.component {
            ComponentType::Response => "responses",
            ComponentType::Schema => "schemas",
        };

        object.serialize_field("$ref", &format!("#/components/{}/{}", component, self.name))?;
        object.end()
    }
}
