mod endpoint;
mod routing;

mod spec;

use proc_macro::TokenStream;

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

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    tri!(spec::schema_macro_impl(input) => TokenStream::new())
}

#[proc_macro_derive(Response, attributes(response))]
pub fn derive_response(input: TokenStream) -> TokenStream {
    tri!(spec::response_macro_impl(input) => TokenStream::new())
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

// trait Reparse: quote::ToTokens {
//     fn reparse<T: syn::parse::Parse>(&self) -> Result<T, syn::Error> {
//         syn::parse(self.to_token_stream().into())
//     }

//     #[allow(unused)]
//     fn reparse_with<P: syn::parse::Parser>(&self, parser: P) -> Result<P::Output, syn::Error> {
//         parser.parse(self.to_token_stream().into())
//     }
// }

// impl<T: quote::ToTokens> Reparse for T {}

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
