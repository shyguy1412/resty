mod meta;
mod path;
mod response;
mod schema;

pub use meta::apply_meta;
pub use path::add_path;
pub use response::response_macro_impl;
pub use schema::schema_macro_impl;

pub mod definition;
use definition::*;

use std::{
    collections::BTreeMap,
    convert::identity,
    ops::{Deref, DerefMut},
    sync::{Arc, LazyLock, PoisonError, RwLock, RwLockWriteGuard},
};

//TODO: Allow paths to md files for descriptions
//IDEA: Allow for extensions

static SPEC: LazyLock<Arc<RwLock<Specification>>> = LazyLock::new(Default::default);

struct SpecGuard<T: DerefMut<Target = Specification>>(Option<T>);

trait Spec<'a, T: DerefMut<Target = Specification>> {
    fn get(&'a self) -> SpecGuard<T>;
}

impl<'a> Spec<'a, RwLockWriteGuard<'a, Specification>> for LazyLock<Arc<RwLock<Specification>>> {
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
