mod schema;
use schema::*;

mod meta;
use meta::*;

mod path;
use path::*;

mod response;

pub use meta::apply_meta;
pub use path::add_path;
pub use response::response_macro_impl;
pub use schema::schema_macro_impl;

use std::{
    collections::HashMap,
    convert::identity,
    ops::{Deref, DerefMut},
    sync::{LazyLock, Mutex, MutexGuard},
};

use serde::Serialize;

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

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    paths: HashMap<String, Path>,
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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    schemas: HashMap<String, Schema>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    request_bodies: HashMap<String, ()>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    security_schemes: HashMap<String, SecurityScheme>,
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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    scopes: HashMap<String, String>,
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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    properties: HashMap<String, Property>,
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

#[derive(Serialize, Debug)]
// #[serde(untagged)]
enum PropertyType {
    #[serde(rename = "type")]
    Type(String),
    #[serde(rename = "$ref", serialize_with = "prefix_ref")]
    Ref(String),
}

pub fn prefix_ref<S: serde::Serializer>(str: &String, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("#/components/schemas/{str}"))
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

static SPEC: LazyLock<Mutex<Specification>> = LazyLock::new(Default::default);

struct SpecGuard<T: DerefMut<Target = Specification>>(T);

trait Spec<'a, T: DerefMut<Target = Specification>> {
    fn get(&'a self) -> SpecGuard<T>;
}

impl<'a> Spec<'a, MutexGuard<'a, Specification>> for LazyLock<Mutex<Specification>> {
    fn get(&'a self) -> SpecGuard<MutexGuard<'a, Specification>> {
        let guard = self.lock().map_or_else(|e| e.into_inner(), identity);
        SpecGuard(guard)
    }
}

impl<T: DerefMut<Target = Specification>> Deref for SpecGuard<T> {
    type Target = Specification;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DerefMut<Target = Specification>> DerefMut for SpecGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: DerefMut<Target = Specification>> Drop for SpecGuard<T> {
    fn drop(&mut self) {
        write_decl(&self.0);
    }
}

fn write_decl(spec: &Specification) {
    if is_io_allowed() {
        let file = decl_file();
        let _ = serde_json::to_writer_pretty(file, spec).expect("foo");
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

trait ParseArgument<'a> {
    type Arg;

    fn parse_iter<R: 'a>(
        self,
        map: fn(arg: &Self::Arg) -> Result<R, syn::Error>,
    ) -> impl Iterator<Item = Result<R, syn::Error>>;

    fn parse<R: 'a>(
        self,
        map: fn(arg: &Self::Arg) -> Result<R, syn::Error>,
    ) -> Result<Option<R>, syn::Error>;
}

impl<'a, I: 'a, T: 'a> ParseArgument<'a> for I
where
    I: IntoIterator<Item = (&'a syn::Ident, &'a T)>,
{
    type Arg = T;

    fn parse<R: 'a>(
        self,
        map: fn(arg: &T) -> Result<R, syn::Error>,
    ) -> Result<Option<R>, syn::Error> {
        self.parse_iter(map)
            .nth(0)
            .map(|r| r.map(Some))
            .map_or(Ok(None), identity)
    }

    fn parse_iter<R: 'a>(
        self,
        map: fn(arg: &Self::Arg) -> Result<R, syn::Error>,
    ) -> impl Iterator<Item = Result<R, syn::Error>> {
        self.into_iter().map(|(.., v)| v).map(map)
    }
}
