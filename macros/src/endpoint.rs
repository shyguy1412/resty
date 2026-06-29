use proc_macro::{Span, TokenStream};
use quote::{ToTokens, format_ident};

use crate::{compile_error, parse, routing::get_endpoint_path, spec::register_endpoint};

fn endpoint_segments(path: Option<String>) -> Result<Vec<String>, syn::Error> {
    let path = path.or_else(get_endpoint_path);
    let segments = path
        .as_ref()
        .map(|v| v.as_str())
        // .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .or_else(|| match Span::call_site().local_file() {
            None => Some("<rust-analyzer has not yet implemented Span::local_file>"),
            Some(..) => None,
        })
        .map(|p| p.split("/"));

    let Some(segments) = segments else {
        return Err(compile_error(
            "Can not determine endpoint route. Maybe you are missing a Path directive?",
        ));
    };

    Ok(segments.map(|s| s.to_string()).skip(1).collect())
}

fn method_byte(methods: &Vec<&syn::Expr>) -> Result<syn::Expr, syn::Error> {
    syn::parse(
        quote::quote! {
            {
                use ::resty::HttpMethod::*;
                #(#methods as u16 )|*
            }
        }
        .into(),
    )
}

macro_rules! combined_errors {
    ($errs: ident => $first: expr) => {
        match $first {
            Ok(ok) => Some(ok),
            Err(err) => {
                $errs.push(err);
                None
            }
        }
    };
    ($errs:ident => $first:expr, $($rest:expr),+) => {{
        combined_errors!($errs => $first).zip(combined_errors!($errs => $($rest),*))
    }};
    ($($rest:expr),+) => {{
        let mut errors = Vec::new();
        let values = combined_errors!(errors => $($rest),* );
        values.ok_or_else(||crate::parse::combine_errors(errors).expect_err("If values isnt Some there must be an error"))
    }};
}

pub fn endpoint_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let endpoint_fn: syn::ItemFn = syn::parse(body)?;
    let fn_ident = &endpoint_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("Handler function is missing a lifetime parameter");

    //TODO Make a system to collect parsing errors so they can all be shown at once
    let args = parse::args(args)?;

    let (methods, (router, (static_headers, (path, responds)))) = combined_errors!(
        parse::methods(&args),
        parse::router(&args),
        parse::static_headers(&args),
        parse::path_override(&args),
        parse::responds(&args)
    )?;

    let segments = endpoint_segments(path)?;
    let method_byte = method_byte(&methods)?;

    for method in methods {
        register_endpoint(
            segments.clone(),
            method.to_token_stream().to_string(),
            &endpoint_fn,
        )
    }
    let handler = quote::quote! {
        pub fn #fn_ident #generics (
            mut req: ::resty::Request<#lifetime>,
            mut res: ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<#lifetime> {
            #endpoint_fn;

            #(#responds)*

            const STATIC_HEADERS :&[(&str, &str)] = &[#(#static_headers),*];

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#segments),*], ::resty::Handler(&#fn_ident, #method_byte));
    };

    Ok(handler.into())
}

pub fn middleware_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let endpoint_fn: syn::ItemFn = syn::parse(body)?;
    let fn_ident = &endpoint_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("Handler function is missing a lifetime parameter");

    let args = parse::args(args)?;

    let (router, path) = combined_errors!(parse::router(&args), parse::path_override(&args))?;

    let segments = endpoint_segments(path)?;

    let handler = quote::quote! {
        pub fn #fn_ident <#lifetime, '__fn_borrow> (
            req: &'__fn_borrow mut ::resty::Request<#lifetime>,
            res: &'__fn_borrow mut ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            #endpoint_fn;

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })

        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#segments),*], ::resty::Middleware(&#fn_ident));
    };

    Ok(handler.into())
}
