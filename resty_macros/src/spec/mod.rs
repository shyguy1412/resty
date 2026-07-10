mod meta;
mod path;
mod response;
mod schema;

pub use meta::apply_meta;
pub use path::add_path;
pub use response::response_macro_impl;
pub use schema::schema_macro_impl;

use std::{
    collections::BTreeMap,
    convert::identity,
    ops::{Deref, DerefMut},
    sync::{LazyLock, PoisonError, RwLock, RwLockWriteGuard},
};

use serde::{Serialize, ser::SerializeMap};

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Specification {
    openapi: OpenAPIVersion,
    #[serde(skip_serializing_if = "Info::is_empty")]
    info: Info,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<ExternalDocs>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    servers: Vec<Server>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    paths: BTreeMap<String, Path>,
    components: Components,
}

#[derive(Serialize, Default)]
struct Server {
    url: String,
}

#[derive(Serialize, Default)]
enum OpenAPIVersion {
    #[default]
    #[serde(rename = "3.0.4")]
    V304,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Info {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    terms_of_service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<License>,
}

impl Info {
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.version.is_none()
            && self.terms_of_service.is_none()
            && self.contact.is_none()
            && self.license.is_none()
    }
}

#[derive(Serialize, Default)]
struct Contact {
    email: String,
}

#[derive(Serialize, Default)]
struct License {
    name: String,
    url: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Tag {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<ExternalDocs>,
}

#[derive(Serialize, Default)]
struct ExternalDocs {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Components {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    schemas: BTreeMap<String, Schema>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    request_bodies: BTreeMap<String, ()>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    security_schemes: BTreeMap<String, SecurityScheme>,

    //this is skipped and inlined into the paths
    #[serde(skip)]
    responses: BTreeMap<String, Response>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey(ApiKeyScheme),
    #[serde(rename = "oauth2")]
    OAuth2(OAuth2Scheme),
}

#[derive(Serialize, Default)]
struct ApiKeyScheme {
    name: String,
    #[serde(rename = "in")]
    is_in: String,
}
#[derive(Serialize)]
struct OAuth2Scheme {
    flows: OAuth2SchemeFlows,
}
#[derive(Serialize)]
struct OAuth2SchemeFlows {
    implicit: Option<ImplicitOAuth2Flow>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]

struct ImplicitOAuth2Flow {
    authorization_url: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    scopes: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct SpecEnum {
    #[serde(rename = "type")]
    ty: String,

    #[serde(rename = "enum")]
    variants: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
}

#[derive(Serialize)]
struct Schema {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(flatten)]
    schema: EnumOrStruct,
}

#[derive(Serialize)]
#[serde(untagged)]
enum EnumOrStruct {
    Enum(SpecEnum),
    Struct(SpecStruct),
}

#[derive(Serialize)]
#[serde(tag = "type", rename = "object")]
struct SpecStruct {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, Property>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,
}

#[derive(Serialize)]
struct Property {
    #[serde(flatten)]
    ty: PropertyType,
    #[serde(skip_serializing_if = "PropertyMeta::is_none", flatten)]
    meta: PropertyMeta,
}

#[derive(Serialize, Debug, Clone)]
// #[serde(untagged)]
enum PropertyType {
    #[serde(rename = "type")]
    Type(String),
    #[serde(rename = "$ref", serialize_with = "prefix_schema_ref")]
    Ref(String),
}

#[derive(Serialize)]
struct PropertyMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<PropertyType>,
}

impl PropertyMeta {
    fn is_none(&self) -> bool {
        self.format.is_none()
            && self.example.is_none()
            && self.description.is_none()
            && self.items.is_none()
    }
}

#[derive(Serialize)]
pub struct Path {
    #[serde(flatten)]
    methods: BTreeMap<String, Method>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    tags: Vec<String>,
    summary: Option<String>,
    description: Option<String>,
    operation_id: String,
    parameters: Vec<Parameter>,
    request_body: Option<RequestBody>,
    #[serde(serialize_with = "response_type")]
    responses: Vec<ResponseType>,
    security: Vec<BTreeMap<String, Vec<String>>>,
}

#[derive(Serialize, Clone)]
pub struct RequestBody {
    description: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    content: BTreeMap<String, SchemaRef>,
    required: bool,
}

#[derive(Serialize, Clone)]
enum ResponseType {
    //code, description
    Raw(String, String),
    //ref, description
    Ref(String, String),
}

#[derive(Serialize)]
pub struct Parameter {
    name: String,
    #[serde(rename = "in")]
    is_in: String,
    description: Option<String>,
    required: bool,
    explode: bool,
    schema: String,
}

#[derive(Serialize)]
pub struct Response {
    #[serde(skip)]
    code: String,
    description: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    content: BTreeMap<String, SchemaRef>,
}

#[derive(Serialize, Clone)]
struct SchemaRef {
    schema: PropertyType,
}

fn response_type<S: serde::Serializer>(
    responses: &Vec<ResponseType>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut responses_struct = serializer.serialize_map(Some(responses.len()))?;

    let spec = SPEC.read().map_or_else(PoisonError::into_inner, identity);
    let schemas = &spec.components.responses;

    for response in responses {
        match response {
            ResponseType::Raw(code, desc) => responses_struct.serialize_entry(
                code,
                &Response {
                    code: code.clone(),
                    description: desc.clone(),
                    content: BTreeMap::new(),
                },
            )?,
            ResponseType::Ref(reference, desc) => match schemas.get(reference) {
                Some(r) => responses_struct.serialize_entry(
                    &r.code,
                    &Response {
                        code: r.code.clone(),
                        description: desc.clone(),
                        content: r.content.clone(),
                    },
                )?,
                None => responses_struct.serialize_entry(
                    reference,
                    &Response {
                        code: reference.clone(),
                        description: desc.clone(),
                        content: BTreeMap::new(),
                    },
                )?,
            },
        }
    }
    responses_struct.end()
}

fn prefix_schema_ref<S: serde::Serializer>(str: &String, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("#/components/schemas/{str}"))
}

static SPEC: LazyLock<RwLock<Specification>> = LazyLock::new(Default::default);

struct SpecGuard<T: DerefMut<Target = Specification>>(Option<T>);

trait Spec<'a, T: DerefMut<Target = Specification>> {
    fn get(&'a self) -> SpecGuard<T>;
}

impl<'a> Spec<'a, RwLockWriteGuard<'a, Specification>> for LazyLock<RwLock<Specification>> {
    fn get(&'a self) -> SpecGuard<RwLockWriteGuard<'a, Specification>> {
        let guard = self.write().map_or_else(|e| e.into_inner(), identity);
        SpecGuard(Some(guard))
    }
}

impl<T: DerefMut<Target = Specification>> Deref for SpecGuard<T> {
    type Target = Specification;

    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .expect("This is only an option to drop it earlier in the destructor")
    }
}

impl<T: DerefMut<Target = Specification>> DerefMut for SpecGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .expect("This is only an option to drop it earlier in the destructor")
    }
}

impl<T: DerefMut<Target = Specification>> Drop for SpecGuard<T> {
    fn drop(&mut self) {
        //Janky af
        drop(self.0.take());
        write_decl();
    }
}

fn write_decl() {
    if is_io_allowed() {
        let spec = SPEC.read().map_or_else(PoisonError::into_inner, identity);
        let file = decl_file();
        let _ = serde_json::to_writer_pretty(file, &*spec).expect("foo");
    }
}
fn is_io_allowed() -> bool {
    // return true;
    match std::env::var("RESTY_DECL_GEN") {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn decl_file() -> std::fs::File {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(std::env::var("RESTY_DECL_GEN").expect("Must have a path"))
        .expect("Can not open declaration file")
}

fn get_attr_once<'a>(
    name: &str,
    attrs: &'a Vec<syn::Attribute>,
) -> Result<Option<&'a syn::Attribute>, syn::Error> {
    Ok(attrs
        .iter()
        .filter(|attr| {
            match attr
                .path()
                .require_ident()
                .ok()
                .map(|i| i.to_string())
                .as_ref()
                .map(|s| s.as_str())
            {
                Some(cur) => cur == name,
                _ => false,
            }
        })
        .nth(0))
}

fn lit_value(lit: &syn::LitStr) -> Result<String, syn::Error> {
    Ok(lit.value())
}
