mod generate;
#[macro_use]
mod parse;
mod routing;

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::{parse_macro_input, spanned::Spanned};

fn compile_error<E: std::fmt::Display>(span: proc_macro2::Span, err: E) -> TokenStream {
    syn::Error::new(span, err.to_string())
        .to_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn use_manual_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    routing::manual_routing(args, body)
}

#[proc_macro_attribute]
pub fn use_path_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    routing::path_routing(args, body)
}

#[proc_macro_attribute]
pub fn fallback(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse::args(args);
    let static_headers: Vec<syn::Expr> = parse::static_headers(&args);

    let handler = generate::handler(
        &endpoint_fn,
        &format_ident!("__FALLBACK_HANDLER"),
        &static_headers,
    );

    let mut output: TokenStream = quote::quote! {
        use ::resty::__private::*;
        #[linkme::distributed_slice(::resty::FALLBACK)]
        #[linkme(crate = linkme)]
        static __FALLBACK_HANDLER_SLICE: &'static ::resty::Handler = &__FALLBACK_HANDLER;
    }
    .into();

    output.extend(handler);

    output
}

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
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

    methods
        .iter()
        .map(|method| method.to_token_stream().into())
        .filter_map(|method| syn::parse(method).ok())
        .map(|method| {
            generate::endpoint(
                &endpoint_fn,
                &method,
                &static_headers,
                &path.as_ref(),
                &router.as_ref(),
            )
        })
        .collect()
}

#[proc_macro_derive(Serialize, attributes(serializer))]
pub fn derive_resty_serialize(input: TokenStream) -> TokenStream {
    let (serializer, ident, generics) = parse_derive_attr!(
        "serializer" in input
        else "serializer attribute required for deriving Serialize"
    );

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
    let (deserializer, ident, generics) = parse_derive_attr!(
        "deserializer" in input
        else "deserializer attribute required for deriving Deserialize"
    );

    quote::quote! {
    impl #generics ::resty::Deserialize for #ident #generics {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            #deserializer(data)
        }
    }
    }
    .into()
}
