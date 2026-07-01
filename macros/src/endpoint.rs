use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};
use quote::{ToTokens, format_ident};
use syn::spanned::Spanned;

use crate::*;

argue! {
    MiddlewareArgument {
        Router: syn::Path,
        Route: syn::LitStr,
        Header: HeaderArgument,
    }
    HandlerArgument {
        Method: ArgumentList<syn::Expr>,
        Router: syn::Path,
        Route: syn::LitStr,
        Header: HeaderArgument,
        Accepts: syn::Path,
        Responds: RespondsArgument,
        Summary: syn::LitStr,
        Description: syn::LitStr,
    }
    HeaderArgument(syn::LitStr, syn::token::Comma, syn::LitStr)
    RespondsArgument(syn::LitInt, syn::token::Comma, syn::Path)
}

impl ToTokens for HeaderArgument {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream());
        tokens.extend(self.1.to_token_stream());
        tokens.extend(self.2.to_token_stream());
    }
}

pub fn endpoint_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    use HandlerArgument::*;

    let args: ArgumentList<HandlerArgument> = syn::parse(args)?;

    let headers: Vec<_> = argue!(args may repeat Header)
        .map(|(.., header)| header)
        .collect();

    let router = argue!(args may have Router)
        .map(|(.., value)| value.clone())
        .map_or_else(routing::default_router, Ok)?;

    let route = argue!(args may have Route)
        .map(|(.., value)| value.value().split("/").map(ToString::to_string).collect())
        .map_or_else(routing::default_route, Ok)?;

    let methods = argue!(args must have Method)?.1.into_iter();
    let method_byte: syn::Expr = syn::parse2(quote::quote! {
        {
            use ::resty::HttpMethod::*;
            #(#methods as u16 )|*
        }
    })?;

    let responds = argue!(args may repeat Responds)
        .map(|(.., RespondsArgument(code, _, ty))| {
            syn::parse2::<syn::ExprBlock>(quote::quote! {{
                fn __doc_validate<T: ::resty::__private::Public>(code: u16) {
                    __doc_validate::<#ty>(#code);
                }
            }})
        })
        .ok()?;

    let accepts = argue!(args may repeat Accepts)
        .map(|(.., ty)| {
            syn::parse2::<syn::ExprBlock>(quote::quote! {{
                fn __doc_validate<T: ::resty::__private::Public>() {
                    __doc_validate::<#ty>();
                }
            }})
        })
        .ok()?;

    let handler_fn = validate_handler(syn::parse(body)?)?;
    let fn_ident = &handler_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &handler_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("guranteed by validate_handler");

    Ok(quote::quote! {
        pub fn #fn_ident <#lifetime, '__fn_borrow> (
            req: &'__fn_borrow mut ::resty::Request<#lifetime>,
            res: &'__fn_borrow mut ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            #handler_fn;

            #(#responds)*
            #(#accepts)*

            const STATIC_HEADERS :&[(&str, &str)] = &[#((#headers)),*];

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router::__RESTY__ROUTER)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#route),*], ::resty::Handler(&#fn_ident, #method_byte));
    }
    .into())
}

pub fn middleware_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    use MiddlewareArgument::*;

    let args: ArgumentList<MiddlewareArgument> = syn::parse(args)?;

    let router = argue!(args may have Router)
        .map(|(.., value)| value.clone())
        .map_or_else(routing::default_router, Ok)?;

    let route = argue!(args may have Route)
        .map(|(.., value)| value.value().split("/").map(ToString::to_string).collect())
        .map_or_else(routing::default_route, Ok)?;

    let headers: Vec<_> = argue!(args may repeat Header)
        .map(|(.., header)| header)
        .collect();

    let handler_fn = validate_handler(syn::parse(body)?)?;
    let fn_ident = &handler_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");
    let generics = &handler_fn.sig.generics;
    let lifetime = generics
        .lifetimes()
        .nth(0)
        .expect("guranteed by validate_handler");

    Ok(quote::quote! {
        pub fn #fn_ident <#lifetime, '__fn_borrow> (
            req: &'__fn_borrow mut ::resty::Request<#lifetime>,
            res: &'__fn_borrow mut ::resty::Response<#lifetime>
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            #handler_fn;

            const STATIC_HEADERS :&[(&str, &str)] = &[#((#headers)),*];

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router::__RESTY__ROUTER)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#route),*], ::resty::Middleware(&#fn_ident));
    }
    .into())
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
                let __typecheck_request: &mut ::resty::Request<#lifetime> = #req_ident;
                let __typecheck_response: &mut ::resty::Response<#lifetime> = #res_ident;
            }
            #(#statements)*
        }}
        .into(),
    )?;

    handler.block = Box::new(block);

    Ok(handler)
}
