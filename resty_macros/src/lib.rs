mod endpoint;
mod routing;

mod spec;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, parse_macro_input};

// use crate::spec::register_struct;

macro_rules! tri {
    ($expr:expr => $body:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => {
                let mut out: TokenStream = err.into_compile_error().into();
                out.extend($body);
                return out;
            }
        }
    };
}

#[proc_macro_attribute]
pub fn router(args: TokenStream, body: TokenStream) -> TokenStream {
    tri!(routing::router(args, body.clone()) => body)
}

#[doc = include_str!("../docs/macros/endpoint.md")]
#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    tri!(endpoint::endpoint_macro_impl(args, body.clone()) => body)
}

#[doc = include_str!("../docs/macros/middleware.md")]
#[proc_macro_attribute]
pub fn middleware(args: TokenStream, body: TokenStream) -> TokenStream {
    tri!(endpoint::middleware_macro_impl(args, body.clone()) => body)
}

#[doc = include_str!("../docs/traits/Serialize.md")]
#[proc_macro_derive(Serialize, attributes(serializer))]
pub fn derive_resty_serialize(input: TokenStream) -> TokenStream {
    let (
        serializer,
        DeriveInput {
            ident, generics, ..
        },
    ) = tri!(parse_derive_helper::<syn::Path>(
        "serializer",
        input.clone()
    ) => input);

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
    ) = tri!(parse_derive_helper::<syn::Path>(
        "deserializer",
        input.clone()
    ) => input);

    quote::quote! {
    impl #generics ::resty::Deserialize for #ident #generics {
        fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
            #deserializer(data)
        }
    }
    }
    .into()
}

fn parse_derive_helper<P: syn::parse::Parse>(
    helper: &str,
    input: TokenStream,
) -> Result<(P, syn::DeriveInput), syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input)?;
    let tokens = ast
        .attrs
        .iter()
        .find(|attr| attr.path().to_token_stream().to_string() == helper)
        .map_or(Ok(None), |attr| attr.meta.require_list().map(Some))
        .map(|r| r.map(|list| list.tokens.to_token_stream().into()));

    let tokens = match tokens {
        Ok(v) => v,
        Err(err) => return Err(syn::Error::new(ast.ident.span(), err)),
    };

    let Some(tokens) = tokens else {
        return Err(syn::Error::new(
            ast.ident.span(),
            format!("{} attribute required", helper),
        ));
    };

    Ok((syn::parse(tokens)?, ast))
}

/// Mark a struct to be documented as openapi schema
#[proc_macro_attribute]
pub fn public(_: TokenStream, body: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(body as syn::ItemStruct);
    let ident = &item_struct.ident;

    // register_struct(&item_struct);

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
