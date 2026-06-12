mod parse;
use parse::*;

use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    sync::Mutex,
};

use proc_macro::{TokenStream, TokenTree};
use quote::{ToTokens, format_ident};
use syn::{parse_macro_input, parse_str};

static BASE_PATHS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

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

fn error_to_compile_error<E: std::error::Error>(span: proc_macro2::Span, err: E) -> TokenStream {
    syn::Error::new(span.into(), err.to_string())
        .to_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn use_manual_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    quote::quote! {}.into()
}

#[proc_macro_attribute]
pub fn use_path_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    let path_litstr = parse_macro_input!(args as syn::LitStr);

    let mut body: Vec<TokenTree> = body.into_iter().collect();
    let Some(static_decl_ident) = body.get(1).and_then(|tt| match tt {
        TokenTree::Ident(ident) => Some(proc_macro2::Ident::new(
            &ident.to_string(),
            ident.span().into(),
        )),
        _ => None,
    }) else {
        return syn::Error::new(
            body.get(1)
                .or(body.get(0))
                .expect("Must have input")
                .span()
                .into(),
            "Expected Ident",
        )
        .into_compile_error()
        .into();
    };
    let router_ident = format_ident!("{static_decl_ident}__RESTY__ROUTER");

    if let Some(last) = body.last()
        && last.to_string() != ";"
    {
        return syn::Error::new(last.span().into(), "Expected a ;")
            .into_compile_error()
            .into();
    }

    body.pop();

    let mut stream = TokenStream::new();

    stream.extend(body);
    stream.extend(Into::<TokenStream>::into(
        quote::quote! {= ::std::sync::LazyLock::new(||::resty::Router::new(&#router_ident));},
    ));
    let static_decl = parse_macro_input!(stream as syn::ItemStatic);

    let path = path_litstr.value();
    let path = path.strip_prefix("./").unwrap_or(&path);
    let path = path.strip_prefix("/").unwrap_or(&path);

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let Some(api_path) = source_file
        .as_ref()
        .and_then(|file| file.parent())
        .map(|p| p.into())
        .or_else(|| {
            let hint = std::env::var("RESTY_PATH_ROUTING_HINT").ok()?;
            Some(PathBuf::from(hint))
        })
        .map(|p| p.join(path))
    else {
        let error = syn::Error::new(
            span.into(),
            "Could not resolve base path for path routing.\nThis is likely due to rust-analyzer not supporting Span::local_file() in proc-macros.\nIf this error only appears in rust-analyzer messages set the RESTY_PATH_ROUTING_HINT environment variable to the base path from which to resolve your resty::use_path_routing macro invocation.",
        );
        return error.to_compile_error().into();
    };

    let _ = BASE_PATHS.lock().map(|mut vec| vec.push(api_path.clone()));

    let files =
        match visit_dirs(&api_path).map_err(|e| error_to_compile_error(path_litstr.span(), e)) {
            Ok(v) => v,
            Err(e) => return e,
        };

    let (paths, idents): (Vec<_>, Vec<_>) = files
        .into_iter()
        .map(|d| {
            d.path()
                .strip_prefix(&api_path)
                .expect("Guranteed by earlier path reads")
                // .strip_prefix(base_module_str)
                // .expect("Guranteed by earlier path reads")
                .to_string_lossy()
                .to_string()
        })
        .map(|p| {
            (
                p.clone(),
                format_ident!(
                    "__endpoint_{}",
                    &p.replace("/", "_")
                        .replace("[", "")
                        .replace("]", "_")
                        .strip_suffix(".rs")
                        .unwrap()
                ),
            )
        })
        .collect();

    quote::quote! {
        #[path = #path_litstr]
        #[allow(non_snake_case)]
        mod #static_decl_ident {
            use ::resty::__private::*;
            #[doc(hidden)]
            #[linkme::distributed_slice]
            #[linkme(crate = linkme)]
            pub static #router_ident: [::resty::RouteSlice];
            use #router_ident as __RESTY__ROUTER;


            #(
                #[path = #paths]
                mod #idents;
            )*

        }
        use #static_decl_ident::#router_ident;
        #static_decl
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

    let router = args.iter().find_map(|a| match a.0 == "Router" {
        true => Some(&a.1[0]),
        false => None,
    });

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
                &router,
            )
        })
        .collect()
}

fn generate_endpoint(
    endpoint_fn: &syn::ItemFn,
    method: &syn::Ident,
    static_headers: &Vec<syn::Expr>,
    path: &Option<&str>,
    router: &Option<&syn::Expr>,
) -> TokenStream {
    let fn_ident = &endpoint_fn.sig.ident;

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let endpoint_from_file = || {
        source_file
            .as_ref()
            .and_then(|path| {
                BASE_PATHS
                    .lock()
                    .ok()
                    .and_then(|paths| paths.iter().find_map(|p| path.strip_prefix(p).ok()))
            })
            .and_then(|path| path.to_str())
            .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
            .and_then(|path| path.strip_suffix("mod").or(Some(path)))
    };

    let endpoint = path
        .or_else(endpoint_from_file)
        .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .unwrap_or("<rust-analyzer has not yet implemented Span::local_file>") //this should only be reachable to rust-analyzer since it does not implement local_file
        .split("/");

    let default_router = parse_str("super::__RESTY__ROUTER").ok();
    let router = router.clone().or(default_router.as_ref());
    // .expect("Could not determine router");

    let slice_ident = format_ident!("__{fn_ident}_{method}_route");

    let handler_ident = format_ident!("__{fn_ident}_{method}");
    let handler = generate_handler(endpoint_fn, &handler_ident, static_headers);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    match router {
        Some(_) => quote::quote! {
            use ::resty::__private::*;
            #[linkme::distributed_slice(#router)]
            #[linkme(crate = linkme)]
            static #slice_ident: ::resty::RouteSlice =(&[#(#endpoint),*], &#handler_ident, ::resty::HttpMethod::#method);
            #handler
        }.into(),
        None => quote::quote! {
            #handler
        }
        .into(),
    }
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
