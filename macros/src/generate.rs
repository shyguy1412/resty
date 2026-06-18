use proc_macro::{Span, TokenStream};
use quote::format_ident;
use syn::{parse_macro_input, parse_str};

use crate::{compile_error, routing::get_endpoint_path};

pub fn handler(
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
                let Some(mut request) = ::resty::Request::new(&data.request, &data.path_params, data.readable).await else {
                    todo!("Handle parsing errors")
                };

                const static_headers :&[(&str, &str)] = &[#(#static_headers),*];

                let mut response = ::resty::Response::new(data.writeable, static_headers);

                #fn_ident(&mut request, &mut response).await;
            })
        }
    }.into()
}

pub fn endpoint(
    endpoint_fn: &syn::ItemFn,
    method: &syn::Ident,
    static_headers: &Vec<syn::Expr>,
    path: &Option<&String>,
    router: &Option<&syn::Path>,
) -> TokenStream {
    let fn_ident = &endpoint_fn.sig.ident;
    let endpoint = get_endpoint_path();
    let endpoint = path
        .or(endpoint.as_ref())
        .map(|v| v.as_str())
        .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .or_else(|| match Span::call_site().local_file() {
            None => Some("<rust-analyzer has not yet implemented Span::local_file>"),
            Some(..) => panic!("IDK why this would be reachable"),
        })
        .map(|p| p.split("/"));

    let Some(endpoint) = endpoint else {
        return compile_error(
            Span::call_site().into(),
            "Can not determine endpoint route. Maybe you are missing a Path directive?",
        );
    };

    let default_router = parse_str("super::__RESTY__ROUTER").ok();
    let router = router.or(default_router.as_ref());

    let handler_ident = format_ident!("__{fn_ident}_{method}");
    let slice_ident = format_ident!("{handler_ident}_route");
    let handler = handler(endpoint_fn, &handler_ident, static_headers);
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
