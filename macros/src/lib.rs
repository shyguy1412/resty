#[macro_use]
mod parse;

mod endpoint;
mod middleware;
mod routing;

mod spec;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use crate::spec::register_struct;

fn compile_error<E: std::fmt::Display>(span: proc_macro2::Span, err: E) -> TokenStream {
    syn::Error::new(span, err.to_string())
        .to_compile_error()
        .into()
}

#[doc = include_str!("../docs/macros/manual_routing.md")]
#[proc_macro_attribute]
pub fn use_manual_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    routing::manual_routing(args, body)
}

#[doc = include_str!("../docs/macros/path_routing.md")]
#[proc_macro_attribute]
pub fn use_path_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    routing::path_routing(args, body)
}

#[doc = include_str!("../docs/macros/endpoint.md")]
#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    endpoint::endpoint_macro_impl(args, body)
}

#[doc = include_str!("../docs/macros/middleware.md")]
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, body: TokenStream) -> TokenStream {
    middleware::middleware_macro_impl(args, body)
}

#[doc = include_str!("../docs/traits/Serialize.md")]
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

#[doc = include_str!("../docs/traits/Deserialize.md")]
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

#[proc_macro_attribute]
pub fn public(_: TokenStream, body: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(body as syn::ItemStruct);
    register_struct(&item_struct);
    item_struct.into_token_stream().into()
}
