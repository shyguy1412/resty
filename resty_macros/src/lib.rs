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

trait ResultIterator<V, E>
where
    Self: Iterator<Item = Result<V, E>> + Sized,
    E: std::iter::Extend<E> + IntoIterator<Item = E>,
{
    fn ok(self) -> Result<Vec<V>, E> {
        let mut error: Option<E> = None;
        let extend_error = &mut |e: E| {
            match &mut error {
                Some(error) => error.extend(e),
                None => error = Some(e),
            }
            None
        };

        let ok = self
            .filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => extend_error(e),
            })
            .collect::<Vec<_>>();

        error.map_or(Ok(ok), Err)
    }
}

impl<V, T: Iterator<Item = Result<V, syn::Error>>> ResultIterator<V, syn::Error> for T where
    Self: Iterator<Item = Result<V, syn::Error>> + Sized
{
}
