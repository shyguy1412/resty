mod generate;
#[macro_use]
mod parse;
mod routing;

use proc_macro::TokenStream;
use quote::ToTokens;

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
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    routing::endpoint_macro_impl(args, body)
}

#[doc = include_str!("../../docs/traits/Serialize.md")]
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

#[doc = include_str!("../../docs/traits/Deserialize.md")]
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
