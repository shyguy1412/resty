use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::{Expr, parse_macro_input};

static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path) -> io::Result<Vec<DirEntry>> {
    let mut files = vec![];
    fn visit_dirs(dir: &Path, files: &mut Vec<DirEntry>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, files)?;
                } else {
                    files.push(entry);
                }
            }
        }
        Ok(())
    }
    visit_dirs(dir, &mut files)?;
    Ok(files)
}

#[proc_macro]
pub fn api_module(body: TokenStream) -> TokenStream {
    let decl: syn::Ident = syn::parse_str(&format!("{}", body))
        .inspect_err(|e| panic!("{e}"))
        .unwrap();

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let source_file = source_file
        .as_ref()
        .and_then(|file| file.parent())
        .map(|file| file.join(decl.to_string()));

    if let Some(source_file) = source_file
        && let Err(_) = BASE_PATH.set(source_file)
    {
        panic!("`api_module` macro may only be called once");
    };

    let (paths, idents): (Vec<_>, Vec<_>) = BASE_PATH
        .get()
        .and_then(|p| visit_dirs(p).ok())
        .unwrap_or(vec![])
        .into_iter()
        .map(|d| {
            d.path()
                .strip_prefix(BASE_PATH.get().unwrap())
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .filter(|p| p != "mod.rs")
        .enumerate()
        .map(|(i, p)| (p, format_ident!("__endpoint{i}")))
        .collect();

    //rust-analyzer is missing support for Span::local_file
    //this enabled manual mod declaration purely for rust_analyzer so intellisense can still work
    match paths.iter().count() == 0 {
        true => quote::quote! {mod #decl;},
        false => quote::quote! {
            mod #decl {
                #(
                    #[path = #paths]
                    mod #idents;
                )*
            }
        },
    }
    .into()
}

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse_macro_input!(args with syn::punctuated::Punctuated<syn::Meta, syn::token::Comma>::parse_terminated);
    let args: Vec<_> = args
        .into_iter()
        .filter_map(|meta| match meta {
            syn::Meta::Path(..) => None,
            syn::Meta::List(syn::MetaList { tokens, path, .. }) => Some((
                path.to_token_stream().to_string(),
                parse_meta_list(tokens.into()),
            )),
            syn::Meta::NameValue(meta_name_value) => Some((
                meta_name_value.path.to_token_stream().to_string(),
                vec![meta_name_value.value.clone()],
            )),
        })
        .collect();

    let methods = &args
        .iter()
        .find(|meta| meta.0 == "Method")
        .expect("Missing required argument: Method")
        .1;

    let static_headers: &Vec<syn::Expr> = &args
        .iter()
        .filter_map(|(key, value)| match key == "Header" {
            true => Some(syn::parse(quote::quote! {(#(#value),*)}.into()).ok()?),
            false => None,
        })
        .collect();

    methods
        .iter()
        .map(|method| method.to_token_stream().to_string())
        .map(|method| format_ident!("{method}"))
        .map(|method| generate_endpoint(&endpoint_fn, &method, static_headers))
        .collect()
}

fn parse_meta_list(tokens: TokenStream) -> Vec<Expr> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;
    let list = syn::parse::Parser::parse(parser, tokens)
        .map(|l| l.into_iter().collect())
        .unwrap_or(vec![]);

    list
}

fn generate_endpoint(
    endpoint_fn: &syn::ItemFn,
    method: &syn::Ident,
    static_headers: &Vec<Expr>,
) -> TokenStream {
    let generics = &endpoint_fn.sig.generics;
    let fn_ident = &endpoint_fn.sig.ident;
    let lifetime = generics.lifetimes().take(1).collect::<Vec<_>>()[0];

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let endpoint = source_file
        .as_ref()
        .and_then(|file| file.strip_prefix(BASE_PATH.get().unwrap()).ok())
        .and_then(|path| path.to_str())
        .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
        .and_then(|path| path.strip_suffix("mod").or(Some(path)))
        .unwrap_or("<error endpoint>")
        .split("/");

    let slice_ident = format_ident!("__{fn_ident}_{method}_route");
    let endpoint_ident = format_ident!("__{fn_ident}_{method}");

    quote::quote! {
        use ::resty::__private::*;
        #[linkme::distributed_slice(::resty::ROUTES)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#endpoint),*], &#endpoint_ident, ::resty::HttpMethod::#method);
        pub fn #endpoint_ident #generics (data: &#lifetime mut ::resty::HandlerData<#lifetime>)
        -> ::resty::EndpointTask<#lifetime> {
            #endpoint_fn;
            Box::pin(async move {
                let Some(mut request) = Request::new(&data.request, &data.path_params, data.stream.clone()).await else {
                    todo!("Handle parsing errors")
                };

                const static_headers :&[(&str, &str)] = &[#(#static_headers),*];

                let mut response = Response::new(data.stream.clone(), static_headers);

                #fn_ident(&mut request, &mut response).await;
                let _ = data.stream.close().await;
            })
        }
    }
    .into()
}

#[proc_macro_derive(Serialize, attributes(serializer))]
pub fn derive_resty_serialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let serializer_tokens = ast
        .attrs
        .iter()
        .find(|attr| attr.path().to_token_stream().to_string() == "serializer")
        .expect("serializer attribute required for deriving Serialize")
        .meta
        .require_list()
        .inspect_err(|err| panic!("{err}"))
        .unwrap()
        .tokens
        .to_token_stream()
        .into();

    let serializer = parse_macro_input!(serializer_tokens as syn::Path);

    let ident = ast.ident;

    quote::quote! {
    impl ::resty::Serialize for #ident {
        fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            #serializer(self)
        }
    }
    }
    .into()
}

#[proc_macro_derive(Deserialize, attributes(deserializer))]
pub fn derive_resty_deserialize(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let deserializer_tokens = ast
        .attrs
        .iter()
        .find(|attr| attr.path().to_token_stream().to_string() == "deserializer")
        .expect("deserializer attribute required for deriving Serialize")
        .meta
        .require_list()
        .inspect_err(|err| panic!("{err}"))
        .unwrap()
        .tokens
        .to_token_stream()
        .into();

    let deserializer = parse_macro_input!(deserializer_tokens as syn::Path);
    let ident = ast.ident;

    quote::quote! {
    impl ::resty::Deserialize for #ident {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            #deserializer(data)
        }
    }
    }
    .into()
}
