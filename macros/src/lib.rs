mod parse;

mod endpoint;
mod routing;

mod spec;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::spec::register_struct;

#[doc = include_str!("../docs/macros/manual_routing.md")]
#[proc_macro_attribute]
pub fn manual_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    match routing::manual_routing(args, body) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc = include_str!("../docs/macros/file_routing.md")]
#[proc_macro_attribute]
pub fn file_routing(args: TokenStream, body: TokenStream) -> TokenStream {
    match routing::file_routing(args, body) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc = include_str!("../docs/macros/endpoint.md")]
#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    match endpoint::endpoint_macro_impl(args, body) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc = include_str!("../docs/macros/middleware.md")]
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, body: TokenStream) -> TokenStream {
    match endpoint::middleware_macro_impl(args, body) {
        Ok(ok) => ok,
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc = include_str!("../docs/traits/Serialize.md")]
#[proc_macro_derive(Serialize, attributes(serializer))]
pub fn derive_resty_serialize(input: TokenStream) -> TokenStream {
    let (
        serializer,
        DeriveInput {
            ident, generics, ..
        },
    ) = match parse::parse_derive_helper::<syn::Path>("serializer", input) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

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
    let (
        deserializer,
        DeriveInput {
            ident, generics, ..
        },
    ) = match parse::parse_derive_helper::<syn::Path>("deserializer", input) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };

    quote::quote! {
    impl #generics ::resty::Deserialize for #ident #generics {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            #deserializer(data)
        }
    }
    }
    .into()
}

/// Mark a struct to be documented as openapi schema
#[proc_macro_attribute]
pub fn public(_: TokenStream, body: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(body as syn::ItemStruct);
    let ident = &item_struct.ident;

    register_struct(&item_struct);

    quote::quote! {
        #item_struct
        impl ::resty::__private::Public for #ident {}
    }
    .into()
}
