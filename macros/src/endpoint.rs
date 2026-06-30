use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::spanned::Spanned;

use crate::*;

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
        values.ok_or_else(||crate::combine_errors(errors).expect_err("If values isnt Some there must be an error"))
    }};
}

pub fn endpoint_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let endpoint_fn = validate_handler(syn::parse(body)?)?;

    let fn_ident = &endpoint_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("guranteed by validate_handler");

    let args = parse::args(args, u16::MAX)?;

    let (methods, (router, (static_headers, (path, (responds, accepts))))) = combined_errors!(
        parse::methods(&args),
        parse::router(&args),
        parse::static_headers(&args),
        parse::route(&args),
        parse::responds(&args),
        parse::accepts(&args)
    )?;

    let method_byte = method_byte(&methods)?;

    for method in methods {
        spec::register_endpoint(
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
    let endpoint_fn = validate_handler(syn::parse(body)?)?;
    let fn_ident = &endpoint_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &endpoint_fn.sig.generics;
    let lifetime = generics.lifetimes().nth(0).ok_or(syn::Error::new(
        endpoint_fn.sig.span(),
        "Handler function is missing a lifetime parameter",
    ))?;

    use parse::MacroArgumentType::*;
    let args = parse::args(args, Router | Route)?;

    let (router, path) = combined_errors!(parse::router(&args), parse::route(&args))?;

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

fn validate_handler(mut handler: syn::ItemFn) -> Result<syn::ItemFn, syn::Error> {
    let handler_ident = &handler.sig.ident;
    let args: Vec<_> = handler
        .sig
        .inputs
        .iter()
        .map(|arg| arg.reparse::<syn::PatType>())
        .ok()?;

    let lifetime = handler
        .sig
        .generics
        .lifetimes()
        .nth(0)
        .ok_or(syn::Error::new_spanned(
            handler.sig.clone(),
            "Expected a lifetime parameter",
        ))?;

    let Some((req, res)) = args.get(0).zip(args.get(1)) else {
        return Err(syn::Error::new_spanned(
            handler.sig.clone(),
            "Expected Request and Response arguments",
        ));
    };

    let req_ident: syn::Ident = req.pat.reparse()?;
    let req_ty: syn::TypeReference = req.ty.reparse()?;
    let mut req_ty: syn::TypePath = req_ty.elem.reparse()?;
    req_ty
        .path
        .segments
        .last_mut()
        .map(|seg| seg.arguments = syn::PathArguments::None);

    let res_ident: syn::Ident = res.pat.reparse()?;
    let res_ty: syn::TypeReference = res.ty.reparse()?;
    let mut res_ty: syn::TypePath = res_ty.elem.reparse()?;
    res_ty
        .path
        .segments
        .last_mut()
        .map(|seg| seg.arguments = syn::PathArguments::None);

    let statements = &handler.block.stmts;
    let span = handler.block.span();

    let block: syn::Block = syn::parse(
        quote::quote_spanned! {span => {
            {
                let __typecheck_handler = async ||{let _: ::resty::Result = #handler_ident(#req_ident, #res_ident).await;};
                let __typecheck_request = |_: &mut ::resty::Request<#lifetime>|{};
                let __typecheck_response = |_: &mut ::resty::Response<#lifetime>|{};
                __typecheck_request(#req_ident);
                __typecheck_response(#res_ident);
            }
            #(#statements)*
        }}
        .into(),
    )?;

    handler.block = Box::new(block);

    Ok(handler)
}
