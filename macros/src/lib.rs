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
    match endpoint::endpoint_macro_impl(args, body.clone()) {
        Ok(ok) => ok,
        Err(err) => {
            let mut out: TokenStream = err.into_compile_error().into();
            out.extend(body);
            return out;
        }
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
    ) = match parse::parse_derive_helper::<syn::Path>("serializer", input.clone()) {
        Ok(v) => v,
        Err(e) => {
            let mut out: TokenStream = e.into_compile_error().into();
            out.extend(input);
            return out;
        }
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

trait ResultIterator<T> {
    fn ok(self) -> Result<Vec<T>, syn::Error>;
}

impl<V, T: IntoIterator<Item = Result<V, syn::Error>>> ResultIterator<V> for T {
    fn ok(self) -> Result<Vec<V>, syn::Error> {
        let (values, errors) = collect_errors(self.into_iter());
        combine_errors(errors)?;
        Ok(values)
    }
}

trait Reparse: quote::ToTokens {
    fn reparse<T: syn::parse::Parse>(&self) -> Result<T, syn::Error> {
        syn::parse(self.to_token_stream().into())
    }

    #[allow(unused)]
    fn reparse_with<P: syn::parse::Parser>(&self, parser: P) -> Result<P::Output, syn::Error> {
        parser.parse(self.to_token_stream().into())
    }
}

impl<T: quote::ToTokens> Reparse for T {}

fn collect_errors<V, E>(it: impl IntoIterator<Item = Result<V, E>>) -> (Vec<V>, Vec<E>) {
    it.into_iter()
        .fold((Vec::new(), Vec::new()), |mut collector, next| {
            match next {
                Ok(ident) => collector.0.push(ident),
                Err(err) => collector.1.push(err),
            };
            collector
        })
}

fn combine_errors(errors: Vec<syn::Error>) -> Result<(), syn::Error> {
    if let Some(mut error) = errors.get(0).cloned() {
        error.extend(errors.into_iter().skip(1));
        return Err(error);
    }

    return Ok(());
}
