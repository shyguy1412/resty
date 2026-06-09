use std::{path::PathBuf, sync::OnceLock};

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::parse_macro_input;

static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[proc_macro]
pub fn api_module(body: TokenStream) -> TokenStream {
    let decl: syn::Ident = syn::parse_str(&format!("{}", body))
        .inspect_err(|e| panic!("{e}"))
        .unwrap();

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let source_file = source_file
        .as_ref()
        .and_then(|file| file.parent())
        .map(|file| file.join(decl.to_string()));

    if let Some(source_file) = source_file
        && let Err(_) = BASE_PATH.set(source_file)
    {
        panic!("`api_module` macro may only be called once");
    };

    quote::quote! {
        mod #decl;
    }
    .into()
}

#[proc_macro_attribute]
pub fn endpoint(args: TokenStream, body: TokenStream) -> TokenStream {
    let endpoint_fn = parse_macro_input!(body as syn::ItemFn);

    let args = parse_macro_input!(args with syn::punctuated::Punctuated<syn::Meta, syn::token::Comma>::parse_terminated);
    let args = match args
        .iter()
        .map(|meta| meta.require_name_value())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(err) => panic!("{err}"),
    };

    let method = args
        .iter()
        .find(|meta| meta.path.to_token_stream().to_string() == "Method")
        .expect("Missing required argument: Method")
        .value
        .to_token_stream()
        .to_string();
    let method = format_ident!("{method}");

    let fn_ident = &endpoint_fn.sig.ident;
    let generics = &endpoint_fn.sig.generics;
    let lifetimes: Vec<_> = generics.lifetimes().collect();

    let (input, input_type): (Vec<_>, Vec<_>) = endpoint_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some((&pat_type.pat, &pat_type.ty)),
        })
        .filter_map(|(input, ty)| match **input {
            syn::Pat::Ident(ref pat_ident) => Some((&pat_ident.ident, ty)),
            _ => None,
        })
        .map(|(input, ty)| (input, remove_generics(*ty.clone())))
        .collect();

    let slice_ident = format_ident!("{fn_ident}_route");

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    let endpoint = source_file
        .as_ref()
        .and_then(|file| file.strip_prefix(BASE_PATH.get().unwrap()).ok())
        .and_then(|path| path.to_str())
        .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
        .and_then(|path| path.strip_suffix("mod").or(Some(path)))
        .unwrap_or("<error endpoint>")
        .split("/");

    quote::quote! {
        #[::resty::linkme::distributed_slice(::resty::ROUTES)]
        #[linkme(crate = ::resty::linkme)]
        static #slice_ident: (&'static [&'static str],::resty::Handler, ::resty::HttpMethod) = (&[#(#endpoint),*], &#fn_ident, ::resty::HttpMethod::#method);
        pub fn #fn_ident #generics (#(#input: #input_type),*) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + #(#lifetimes)+* + Send>> {
            #endpoint_fn;
            // ::resty::spawn_task()
            Box::pin(async {#fn_ident(#(#input.into()),*).await})
        }
    }
    .into()
}

fn remove_generics(mut ty: syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(ref mut type_path) => {
            type_path.path.segments.iter_mut().for_each(|path_segment| {
                match &mut path_segment.arguments {
                    syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                        angle_bracketed_generic_arguments.args = angle_bracketed_generic_arguments
                            .args
                            .clone()
                            .into_iter()
                            .take(1)
                            .collect();
                    }
                    _ => (),
                }
            });

            ty
        }
        _ => ty,
    }
}
