use proc_macro::{Span, TokenStream};
use quote::{ToTokens, format_ident};
use syn::{parse_macro_input, parse_str};

use crate::{compile_error, parse, routing::get_endpoint_path, spec::register_endpoint};

pub fn endpoint_macro_impl(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse::args(args);
    let methods = parse::methods(&args);
    let static_headers: Vec<syn::Expr> = parse::static_headers(&args);
    let path = parse::path_override(&args);

    let router = args.iter().find_map(|a| match a.0 == "Router" {
        true => Some(a.1.get(0)?),
        false => None,
    });
    let mut router = match router.map(|e| e.into_token_stream().into()) {
        Some(expr) => Some(parse_macro_input!(expr as syn::Path)),
        None => None,
    };
    router.as_mut().and_then(|path| {
        path.segments
            .iter_mut()
            .last()
            .map(|seg| seg.ident = format_ident!("__RESTY__ROUTER_{}", &seg.ident))
    });

    let endpoint = path.or_else(get_endpoint_path);
    let endpoint = endpoint
        .as_ref()
        .map(|v| v.as_str())
        // .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .or_else(|| match Span::call_site().local_file() {
            None => Some("<rust-analyzer has not yet implemented Span::local_file>"),
            Some(..) => None,
        })
        .map(|p| p.split("/"));

    let Some(endpoint) = endpoint else {
        return compile_error(
            Span::call_site().into(),
            "Can not determine endpoint route. Maybe you are missing a Path directive?",
        );
    };
    let endpoint: Vec<_> = endpoint.map(|s| s.to_string()).skip(1).collect();

    let methods: Vec<syn::Ident> = methods
        .into_iter()
        .map(|method| method.to_token_stream())
        .inspect(|method| register_endpoint(endpoint.clone(), method.to_string(), &endpoint_fn))
        .filter_map(|method| syn::parse(method.into()).ok())
        .collect();

    let method_byte = quote::quote! {
        {#(::resty::HttpMethod::#methods as u16 )|*}
    }
    .into();

    let method_byte = parse_macro_input!(method_byte as syn::ExprBlock);

    let fn_ident = &endpoint_fn.sig.ident;
    let router = router.or(parse_str("super::__RESTY__ROUTER").ok());

    let slice_ident = format_ident!("__{fn_ident}_route");

    let mut slice = match router {
        Some(_) => quote::quote! {
            use ::resty::__private::*;
            #[linkme::distributed_slice(#router)]
            #[linkme(crate = linkme)]
            static #slice_ident: ::resty::RouteSlice =(&[#(#endpoint),*], ::resty::Handler(&#fn_ident, #method_byte));
        },
        None => quote::quote! {},
    };

    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("Handler function is missing a lifetime parameter");
    let handler = quote::quote! {
        pub fn #fn_ident #generics (
            mut req: ::resty::Request<#lifetime>,
            mut res: ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<#lifetime> {
            #endpoint_fn

            const STATIC_HEADERS :&[(&str, &str)] = &[#(#static_headers),*];

            Box::pin(async move {
                let req = &mut req.as_typed();
                let res = &mut res.as_typed(STATIC_HEADERS);

                let result = #fn_ident(req, res).await;
                res.close().await;
                result?;

                Ok(())
            })

        }
    };

    slice.extend(handler);

    slice.into()
}
