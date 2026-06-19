use std::{fmt::write, io::Write, path::Display, sync::Mutex};

use quote::ToTokens;

static STRUCTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static ENDPOINTS: Mutex<Vec<Endpoint>> = Mutex::new(Vec::new());

#[derive(Debug)]
struct Endpoint {
    path: Vec<String>,
    method: String,
    request: String,
    response: String,
    error: String,
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n  ")?;
        write!(
            f,
            "\"path\": [{}],\n  ",
            self.path
                .iter()
                .map(|p| format!("\"/{p}\""))
                .map(|p| p.replace("/%", "%"))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        write!(f, "\"method\": \"{}\",\n  ", self.method)?;
        write!(f, "\"request\": \"{}\",\n  ", self.request)?;
        write!(f, "\"response\": \"{}\",\n  ", self.response)?;
        write!(f, "\"error\": \"{}\"\n", self.error)?;
        write!(f, "}}")
    }
}

pub fn register_struct(item_struct: &syn::ItemStruct) {
    let ident = item_struct.ident.to_string();
    let mut fields = item_struct
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            (
                f.ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or(i.to_string()),
                f.ty.to_token_stream().to_string(),
            )
        })
        .fold("[".to_string(), |prev, (ident, ty)| {
            format!("{prev}[\"{ident}\", \"{ty}\"],")
        });

    fields.pop();
    fields.push(']');
    let struct_string = format!("{{\"{ident}\": {fields}}}");

    let _ = STRUCTS.lock().map(|mut l| l.push(struct_string));
    write_decl();
}

fn get_generic_from_fn_input(item_fn: &syn::ItemFn, arg: usize, generic: usize) -> String {
    item_fn
        .sig
        .inputs
        .iter()
        .nth(arg)
        .and_then(|ty| match ty {
            syn::FnArg::Typed(ty) => Some(ty),
            _ => None,
        })
        .and_then(|ty| match ty.ty.as_ref() {
            syn::Type::Reference(ty) => Some(ty.elem.as_ref()),
            _ => None,
        })
        .and_then(|ty| match ty {
            syn::Type::Path(ty) => Some(ty),
            _ => None,
        })
        .and_then(|ty| match &ty.path.segments.iter().last()?.arguments {
            syn::PathArguments::AngleBracketed(generics) => Some(generics),
            _ => None,
        })
        .and_then(|generics| generics.args.iter().nth(generic))
        .and_then(|generic| match generic {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .and_then(|ty| match ty {
            syn::Type::Path(type_path) => Some(type_path.path.get_ident()?),
            _ => None,
        })
        .map(|ty| ty.to_token_stream().to_string())
        .unwrap_or("void".to_string())
}

#[allow(unreachable_code)]
pub fn register_endpoint<'a>(path: Vec<String>, method: String, item_fn: &syn::ItemFn) {
    let request = get_generic_from_fn_input(item_fn, 0, 1);

    let response = get_generic_from_fn_input(item_fn, 1, 1);

    let error = get_generic_from_fn_input(item_fn, 1, 2);

    let endpoint = Endpoint {
        path,
        method,
        request: request,
        response: response,
        error: error,
    };

    let _ = ENDPOINTS.lock().map(|mut l| l.push(endpoint));

    write_decl();
}

fn write_decl() {
    if is_io_allowed() {
        let mut file = decl_file();
        let dts = decl_content();
        file.write_all(dts.as_bytes()).unwrap();
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

fn decl_content() -> String {
    let structs = STRUCTS
        .lock()
        .map(|structs| format!("[\n  {}\n]", structs.join(",\n  ")))
        .expect("Can not be poisoned");

    let endpoints = ENDPOINTS
        .lock()
        .map(|endpoints| {
            format!(
                "[\n{}\n]",
                endpoints
                    .iter()
                    .map(|e| e.to_string())
                    .map(|e| indent(e))
                    .collect::<Vec<_>>()
                    .join(",\n")
            )
        })
        .expect("Can not be poisoned");

    let structs = indent(structs);
    let endpoints = indent(endpoints);

    let output = format!("{{\n  \"structs\": {structs},\n  \"endpoints\": {endpoints}\n}}");

    output
}

fn indent(str: String) -> String {
    str.lines()
        .map(|l| format!("  {l}"))
        .collect::<Vec<String>>()
        .join("\n")
}
