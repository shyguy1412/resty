use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};
use quote::{ToTokens, format_ident};
use syn::{parse::discouraged::AnyDelimiter, spanned::Spanned};

use crate::{endpoint::HandlerArgument::Method, spec::add_path, *};

argue! {
    pub MiddlewareArgument {
        Method: syn::Ident,
        Router: syn::Path,
        Route: syn::LitStr,
    };
    pub HandlerArgument {
        Method: syn::Ident,
        Router: syn::Path,
        Route: syn::LitStr,
        Header: HeaderArgument,
        Tag: syn::LitStr,
        Summary: syn::LitStr,
        Description: syn::LitStr,
        Request: ArgumentList<RequestArgument>,
        Response: ResponseArgument,
        Security: ArgumentList<SecurityArgument>
    };
    pub RequestArgument {
        Description: syn::LitStr,
        Schema: SchemaArgument,
        Required
    };
    pub SecurityArgument {
        Name: syn::LitStr,
        Scope: syn::LitStr
    };
    pub ResponseArgument(syn::LitInt, syn::token::Comma, ResponseType);
    pub SchemaArgument(syn::LitStr, syn::token::Comma, syn::Ident);
    pub HeaderArgument(syn::LitStr, syn::token::Comma, syn::LitStr);
}

pub enum ResponseType {
    Ref(syn::Ident),
    Array(syn::Ident),
    Contentless(syn::LitStr),
}

impl syn::parse::Parse for ResponseType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use ResponseType::*;
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitStr) {
            Ok(Contentless(input.parse()?))
        } else if lookahead.peek(syn::token::Bracket) {
            let (.., buf) = input.parse_any_delimiter()?;
            Ok(Array(buf.parse()?))
        } else {
            Ok(Ref(input.parse()?))
        }
    }
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
    //Restrict the args available for middlewares
    let _: ArgumentList<MiddlewareArgument> = syn::parse(args.clone())?;
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
    let methods = argue!(args may repeat Method).map(|(.., method)| method);

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
