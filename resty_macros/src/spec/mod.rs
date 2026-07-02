mod parse;
pub use parse::*;

use std::{
    collections::HashMap,
    convert::identity,
    ops::{Deref, DerefMut},
    sync::{LazyLock, Mutex, MutexGuard},
};

use serde::Serialize;

#[derive(Serialize, Default)]
struct Specification {
    openapi: OpenAPIVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_docs: Option<()>,
    #[serde(skip_serializing_if = "Option::is_none")]
    servers: Option<()>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<()>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    paths: HashMap<String, Path>,
    components: Components, // schemas: Vec<Schema>,
                            // paths: Vec<Path>,
                            // meta: String,
}

#[derive(Serialize, Default)]
enum OpenAPIVersion {
    #[default]
    #[serde(rename = "3.0.4")]
    V304,
}

#[derive(Serialize, Default)]
struct Components {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    schemas: HashMap<String, Schema>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    request_bodies: HashMap<String, ()>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    security_schemes: HashMap<String, ()>,
}

#[derive(Serialize)]
struct Path;

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
struct SpecStruct {
    #[serde(skip_serializing_if = "Option::is_none")]
    example: Option<String>,
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
        let _ = serde_json::to_writer_pretty(file, spec);
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
