mod parse;
use parse::*;

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
pub fn fallback(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse_args(args);
    let static_headers: Vec<syn::Expr> = parse_static_headers(&args);
    let handler = generate_handler(
        &endpoint_fn,
        &format_ident!("__FALLBACK_HANDLER"),
        &static_headers,
    );
    let handler = parse_macro_input!(handler as syn::ItemFn);

    quote::quote! {
        use ::resty::__private::*;
        #[linkme::distributed_slice(::resty::FALLBACK)]
        #[linkme(crate = linkme)]
        static __FALLBACK_HANDLER_SLICE: &'static ::resty::Handler = &__FALLBACK_HANDLER;

        #handler

    }
    .into()
}

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse_args(args);
    let methods = parse_methods(&args);
    let static_headers: Vec<syn::Expr> = parse_static_headers(&args);
    let path = parse_path_override(&args);

    methods
        .iter()
        .map(|method| method.to_token_stream().to_string())
        .map(|method| format_ident!("{method}"))
        .map(|method| {
            generate_endpoint(
                &endpoint_fn,
                &method,
                &static_headers,
                &path.as_ref().map(|s| s.as_str()),
            )
        })
        .collect()
}

fn generate_endpoint(
    endpoint_fn: &syn::ItemFn,
    method: &syn::Ident,
    static_headers: &Vec<syn::Expr>,
    path: &Option<&str>,
) -> TokenStream {
    let fn_ident = &endpoint_fn.sig.ident;

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let endpoint_from_file = || {
        source_file
            .as_ref()
            .and_then(|file| file.strip_prefix(BASE_PATH.get()?).ok())
            .and_then(|path| path.to_str())
            .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
            .and_then(|path| path.strip_suffix("mod").or(Some(path)))
    };

    let endpoint = path
        .or_else(endpoint_from_file)
        .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .unwrap_or_else(|| {
            if let Ok(var) = std::env::var("RUST_ANALYZER")
                && var == "true"
            {
                return "<rust-analyzer has not yet implemented Span::local_file>";
            }
            panic!("Could not determine endpoint path")
        }) //this should only be reachable to rust-analyzer since it does not implement local_file
        .split("/");

    let slice_ident = format_ident!("__{fn_ident}_{method}_route");

    let handler_ident = format_ident!("__{fn_ident}_{method}");
    let handler = generate_handler(endpoint_fn, &handler_ident, static_headers);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    quote::quote! {
        use ::resty::__private::*;
        #[linkme::distributed_slice(::resty::ROUTES)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#endpoint),*], &#handler_ident, ::resty::HttpMethod::#method);
        #handler
    }
    .into()
}

fn generate_handler(
    endpoint_fn: &syn::ItemFn,
    handler_ident: &syn::Ident,
    static_headers: &Vec<syn::Expr>,
) -> TokenStream {
    let generics = &endpoint_fn.sig.generics;
    let fn_ident = &endpoint_fn.sig.ident;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("Handler function is missing a lifetime parameter");

    quote::quote! {
        pub fn #handler_ident #generics (data: &#lifetime mut ::resty::HandlerData<#lifetime>)
        -> ::resty::EndpointTask<#lifetime> {
            use ::resty::__private::*;
            #endpoint_fn;
            Box::pin(async move {
                let Some(mut request) = ::resty::Request::new(&data.request, &data.path_params, data.stream.clone()).await else {
                    todo!("Handle parsing errors")
                };

                const static_headers :&[(&str, &str)] = &[#(#static_headers),*];

                let mut response = ::resty::Response::new(data.stream.clone(), static_headers);

                #fn_ident(&mut request, &mut response).await;

                use smol::io::AsyncWriteExt;
                let _ = data.stream.close().await;
            })
        }
    }.into()
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
    let generics = ast.generics;

    quote::quote! {
    impl #generics ::resty::Serialize for #ident #generics {
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
    let generics = ast.generics;

    quote::quote! {
    impl #generics ::resty::Deserialize for #ident #generics {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            #deserializer(data)
        }
    }
    }
    .into()
}
