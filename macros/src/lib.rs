use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::parse_macro_input;

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
        .enumerate()
        .map(|(i, p)| (p, format_ident!("__endpoint{i}")))
        .collect();

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
    let args = match args
        .iter()
        .map(|meta| meta.require_name_value())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(err) => panic!("{err}"),
    };

    let method = args
        .iter()
        .find(|meta| meta.path.to_token_stream().to_string() == "Method")
        .expect("Missing required argument: Method")
        .value
        .to_token_stream()
        .to_string();
    let method = format_ident!("{method}");

    let fn_ident = &endpoint_fn.sig.ident;
    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics.lifetimes().take(1).collect::<Vec<_>>()[0];

    let slice_ident = format_ident!("{fn_ident}_route");

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

    quote::quote! {
        use ::resty::__private::*;
        #[linkme::distributed_slice(::resty::ROUTES)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#endpoint),*], &#fn_ident, ::resty::HttpMethod::#method);
        pub fn #fn_ident #generics (data: ::resty::HandlerData<#lifetime>)
        -> ::resty::EndpointTask<#lifetime> {
            #endpoint_fn;
            Box::pin(async move {
                let Some(request) = Request::new(data.request, data.path_params, data.stream.clone()).await else {
                    todo!("Handle parsing errors")
                };

                let response = Response::new(data.stream.clone());
                #fn_ident(request, response).await;
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
        fn serialize(self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
