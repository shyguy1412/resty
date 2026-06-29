use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};

use crate::{parse, spec::register_endpoint};

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

    let args = parse::args(args, u16::MAX)?;

    let (methods, (router, (static_headers, (path, (responds, accepts))))) = combined_errors!(
        parse::methods(&args),
        parse::router(&args),
        parse::static_headers(&args),
        parse::path(&args),
        parse::responds(&args),
        parse::accepts(&args)
    )?;

    let method_byte = method_byte(&methods)?;

    for method in methods {
        register_endpoint(
            path.clone(),
            method.to_token_stream().to_string(),
            &endpoint_fn,
        )
    }
    let handler = quote::quote! {
        pub fn #fn_ident <#lifetime, '__fn_borrow> (
            req: &'__fn_borrow mut ::resty::Request<#lifetime>,
            res: &'__fn_borrow mut ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            #endpoint_fn;

            #(#responds)*
            #(#accepts)*

            const STATIC_HEADERS :&[(&str, &str)] = &[#(#static_headers),*];

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#path),*], ::resty::Handler(&#fn_ident, #method_byte));
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

    use parse::MacroArgumentType::*;
    let args = parse::args(args, Router | Path)?;

    let (router, path) = combined_errors!(parse::router(&args), parse::path(&args))?;

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
        static #slice_ident: ::resty::RouteSlice =(&[#(#path),*], ::resty::Middleware(&#fn_ident));
    };

    Ok(handler.into())
}
