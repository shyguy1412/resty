use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};
use quote::ToTokens;

use crate::spec::{
    SPEC, Spec,
    definition::{ComponentType, Content},
    get_attr_once,
};

argue! {
    ResponseArgument {
        Description: syn::LitStr,
        Schema: syn::LitStr,
        Status: StatusArgument,
        Header: HeaderArgument,
        ContentType: syn::LitStr,
    };
    StatusArgument(syn::LitInt, syn::token::Comma, syn::LitStr);
    ContentTypeArgument(syn::LitStr, syn::token::Comma, syn::Path);
    HeaderArgument(syn::LitStr, syn::token::Comma, syn::LitStr);
}

impl quote::ToTokens for HeaderArgument {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let HeaderArgument(a, _, b) = self;
        tokens.extend(quote::quote! {(#a, #b)});
    }
}

pub fn response_macro_impl(input: TokenStream) -> Result<TokenStream, syn::Error> {
    use ResponseArgument::*;

    let input: syn::DeriveInput = syn::parse(input)?;
    let ident = &input.ident;

    let Some(attr) = get_helper_attr(&input.attrs)? else {
        return Err(syn::Error::new_spanned(
            &input,
            "Expected a derive helper for Response",
        ));
    };

    declare_response(ident, attr.meta.to_token_stream())?;

    let args: ArgumentList<ResponseArgument> =
        syn::parse2(attr.meta.require_list()?.tokens.clone())?;
    // let content_types = argue!(args may repeat ContentType).map(|(.., p)| &p.2);
    let StatusArgument(code, _, reason) = argue!(args must have Status)?.1;
    let headers = argue!(args may repeat Header).map(|(.., h)| h);

    Ok(quote::quote! {
    impl ::resty::RestResponse for #ident {
        const CODE: u16 = #code;
        const REASON: &'static str = #reason;
        const HEADERS: &'static [(&'static str, &'static str)] = &[#(#headers),*];
    }
    }
    .into())
}
fn declare_response(ident: &syn::Ident, input: proc_macro2::TokenStream) -> Result<(), syn::Error> {
    use ResponseArgument::*;
    let meta_list: syn::MetaList = syn::parse2(input)?;
    let args: ArgumentList<ResponseArgument> = syn::parse2(meta_list.tokens)?;
    let content_types = argue!(args may repeat ContentType)
        .map(|(.., p)| p.value())
        .map(|content_type| {
            (
                content_type,
                Content {
                    schema: super::OrRef::Ref(super::ReferenceObject {
                        component: ComponentType::Schema,
                        name: ident.to_string(),
                    }),
                },
            )
        });

    let content_types_array = argue!(args may repeat ContentType)
        .map(|(.., p)| p.value())
        .map(|content_type| {
            (
                content_type,
                Content {
                    schema: super::OrRef::Val(super::Schema {
                        description: None,
                        example: None,
                        ty: super::SchemaType::Array(super::ArraySchema {
                            items: Box::new(super::OrRef::Ref(super::ReferenceObject {
                                component: ComponentType::Schema,
                                name: ident.to_string(),
                            })),
                        }),
                    }),
                },
            )
        });

    let name = argue!(args may have Schema)?
        .map(|(.., n)| n.value())
        .unwrap_or_else(|| ident.to_string());

    let description = argue!(args must have Description)?.1.value();

    let response = super::Response {
        content: content_types.collect(),
        description: description.clone(),
    };
    let response_array = super::Response {
        content: content_types_array.collect(),
        description: description.clone(),
    };
    let mut spec = SPEC.get();
    spec.components
        .responses
        .entry(format!("autogen__{name}Array"))
        .or_insert(response_array);
    spec.components.responses.entry(name).or_insert(response);
    Ok(())
}

fn get_helper_attr(attrs: &Vec<syn::Attribute>) -> Result<Option<&syn::Attribute>, syn::Error> {
    get_attr_once("response", attrs)
}
