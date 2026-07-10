use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};
use quote::{ToTokens, format_ident};
use syn::spanned::Spanned;

use crate::{endpoint::HandlerArgument::Method, spec::add_path, *};

argue! {
    pub HandlerArgument {
        Method: ArgumentList<syn::Ident>,
        Router: syn::Path,
        Route: syn::LitStr,
        Header: HeaderArgument,

        Meta: ArgumentList<syn::MetaList>,
    };
    pub HeaderArgument(syn::LitStr, syn::token::Comma, syn::LitStr);
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
    add_path(args.clone())?;
    handler_impl(args, body, endpoint_variant)
}
pub fn middleware_macro_impl(
    args: TokenStream,
    body: TokenStream,
) -> Result<TokenStream, syn::Error> {
    handler_impl(args, body, middleware_variant)
}

pub fn parse_router(router: &syn::Path) -> Result<syn::Path, syn::Error> {
    let ident = router
        .segments
        .last()
        .ok_or(syn::Error::new_spanned(router, "Can not get path segment"))?;

    Ok(syn::parse_quote_spanned! {router.span() => #router::#ident})
}

pub fn parse_route(route: &syn::LitStr) -> Result<Vec<String>, syn::Error> {
    Ok(route
        .value()
        .trim_matches('/')
        .split("/")
        .map(ToString::to_string)
        .collect::<Vec<_>>())
}

fn handler_impl(
    args: TokenStream,
    body: TokenStream,
    variant: fn(
        &ArgumentList<HandlerArgument>,
        &syn::Ident,
    ) -> Result<proc_macro2::TokenStream, syn::Error>,
) -> Result<TokenStream, syn::Error> {
    use HandlerArgument::*;
    let handler_fn = syn::parse::<syn::ItemFn>(body)?;
    // let handler_fn = validate_handler(syn::parse(body)?)?;
    let fn_ident = &handler_fn.sig.ident;
    let slice_ident = format_ident!("__{fn_ident}_route");

    let args: ArgumentList<HandlerArgument> = syn::parse(args)?;
    let router = argue!(args may have Router)?
        .parse(parse_router)?
        .map_or_else(routing::default_router, Ok)?;

    let headers: Vec<_> = argue!(args may repeat Header)
        .map(|(.., header)| header)
        .collect();

    let route = argue!(args may have Route)?
        .parse(parse_route)?
        .map_or_else(routing::default_route, Ok)?;

    let variant = variant(&args, fn_ident)?;

    Ok(quote::quote! {
        pub fn #fn_ident <'a, 'data, '__fn_borrow> (
            req: &'__fn_borrow mut ::resty::Request<'a, 'data>,
            res: &'__fn_borrow mut ::resty::Response<'a>
        ) -> ::resty::EndpointTask<'__fn_borrow> {
            #handler_fn;

            const STATIC_HEADERS :&[(&str, &str)] = &[#((#headers)),*];

            Box::pin(async move {
                #fn_ident(req, res).await?;
                Ok(())
            })
        }
        use ::resty::__private::*;
        #[linkme::distributed_slice(#router)]
        #[linkme(crate = linkme)]
        static #slice_ident: ::resty::RouteSlice =(&[#(#route),*], #variant);
    }
    .into())
}

fn endpoint_variant(
    args: &ArgumentList<HandlerArgument>,
    fn_ident: &syn::Ident,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    use HandlerArgument::*;
    let method_arg = argue!(args must have Method)?;
    let methods = method_arg.1.iter();

    Ok(quote::quote! {::resty::Handler(&#fn_ident, {
        use ::resty::HttpMethod::*;
        #(#methods as u16 )|*
    })})
}

fn middleware_variant(
    args: &ArgumentList<HandlerArgument>,
    fn_ident: &syn::Ident,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    if let Some((ident, ..)) = argue!(args may have Method)? {
        return Err(syn::Error::new_spanned(
            ident,
            "Method may not be declared for middlewares",
        ));
    }

    Ok(quote::quote! {::resty::Middleware(&#fn_ident)})
}

#[allow(unused)]
fn validate_handler(mut handler: syn::ItemFn) -> Result<syn::ItemFn, syn::Error> {
    let handler_ident = &handler.sig.ident;
    let args: Vec<_> = handler
        .sig
        .inputs
        .iter()
        .map(|arg| arg.reparse::<syn::PatType>())
        .ok()?;

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
                let __typecheck_request: &mut ::resty::Request = #req_ident;
                let __typecheck_response: &mut ::resty::Response = #res_ident;
            }
            #(#statements)*
        }}
        .into(),
    )?;

    handler.block = Box::new(block);

    Ok(handler)
}
