use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};
use quote::ToTokens;

use super::*;
use crate::{Reparse, combine_errors};

argue!(
    SchemaArgument {
        Description: syn::LitStr,
        Name: syn::Ident,
        Type: syn::LitStr,
    };
    PropertyArgument {
        Example: syn::Expr,
        Format: syn::LitStr,
        Ref: syn::Ident,
        Description: syn::LitStr,
    };
    VariantArgument {
        Repr: syn::Expr,
        Example,
    };
);

#[allow(unused)]
enum VariantArgumentOld {
    Repr(::syn::Ident, syn::Expr),
    Example(::syn::Ident, ::syn::Path),
}

impl ::syn::parse::Parse for VariantArgumentOld {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        let meta: ::syn::Meta = input.parse()?;
        let path = match &meta {
            ::syn::Meta::Path(path) => path,
            ::syn::Meta::List(meta_list) => &meta_list.path,
            ::syn::Meta::NameValue(meta_name_value) => &meta_name_value.path,
        };
        use VariantArgumentOld::*;
        let ident: ::syn::Ident = path.require_ident()?.clone();

        match ident.to_string().as_str() {
            stringify!(Repr) => ::proc_macro_argue::Expect::<::syn::MetaList>::expect(meta)
                .map(|list| list.tokens)
                .and_then(::syn::parse2)
                .map(|r| Repr(ident, r)),
            stringify!(Example) => {
                ::proc_macro_argue::Expect::<::syn::Path>::expect(meta).map(|r| Example(ident, r))
            }
            _ => Err(syn::Error::new_spanned(ident, "Invalid Argument")),
        }
    }
}

fn get_attr_once(attrs: &mut Vec<syn::Attribute>) -> Result<Option<syn::Attribute>, syn::Error> {
    combine_errors(
        attrs
            .iter()
            .skip(1)
            .map(|attr| {
                syn::Error::new_spanned(
                    attr.path(),
                    format!("{} attribute may not repeat", attr.path().to_token_stream()),
                )
            })
            .collect(),
    )
    .and(Ok((0 < attrs.len()).then(|| attrs.remove(0))))
}

pub fn schema_macro_impl(input: TokenStream) -> Result<TokenStream, syn::Error> {
    use SchemaArgument::*;

    let mut input: syn::DeriveInput = syn::parse(input)?;
    let ident = &input.ident;
    let mut name_ident = &input.ident;

    let schema = match input.data {
        syn::Data::Struct(data_struct) => EnumOrStruct::Struct(declare_struct_schema(data_struct)?),
        syn::Data::Enum(data_enum) => EnumOrStruct::Enum(declare_enum_schema(data_enum)?),
        syn::Data::Union(data_union) => {
            return Err(syn::Error::new_spanned(
                data_union.union_token,
                "unions are not supported for schemas",
            ));
        }
    };

    let args: Option<ArgumentList<SchemaArgument>> = get_attr_once(&mut input.attrs)?
        .map(|attr| attr.meta)
        .map(|meta| meta.require_list().cloned())
        .map_or(Ok(None), |meta| meta.map(Some))?
        .map(|meta| meta.tokens)
        .map(syn::parse2)
        .map_or(Ok(None), |meta| meta.map(Some))?;

    let name = args
        .as_ref()
        .map(|args| argue!(args may have Name))
        .map_or(Ok(None), |meta| meta.map(Some))?
        .and_then(identity)
        .map_or(ident.to_string(), |(.., name)| {
            name_ident = name;
            name.to_string()
        });

    let description = args
        .as_ref()
        .map(|args| argue!(args may have Description))
        .map_or(Ok(None), |meta| meta.map(Some))?
        .and_then(identity)
        .map(|(.., desc)| desc.value());

    let schema: Schema = Schema {
        schema,
        description,
    };

    let mut spec = SPEC.get();
    if spec.components.schemas.insert(name, schema).is_some() {
        return Err(syn::Error::new_spanned(name_ident, "duplicate schema name"));
    }

    Ok(quote::quote! {
        impl ::resty::__private::Schema for #ident{}
    }
    .into())
}

fn declare_enum_schema(mut data_enum: syn::DataEnum) -> Result<SpecEnum, syn::Error> {
    use VariantArgument::*;

    let mut variants = Vec::new();

    let mut example = None;

    for variant in data_enum.variants.iter_mut() {
        //todo: this can def be better
        let default = lowercase_first_letter(&variant.ident.to_string());

        let Some(attr) = get_attr_once(&mut variant.attrs)? else {
            variants.push(default);
            continue;
        };
        let meta: syn::MetaList = attr.meta.reparse()?;
        let args: ArgumentList<VariantArgument> = syn::parse2(meta.tokens)?;

        //todo: parse lit better
        let variant = argue!(args may have Repr)?
            .map_or(default, |(.., name)| name.to_token_stream().to_string());

        if argue!(args may have Example)?.is_some() {
            example.replace(variant.clone());
        }

        variants.push(variant);
    }

    Ok(SpecEnum {
        //Todo: infer other types
        ty: "string".to_string(),
        variants,
        example,
    })
}

fn lowercase_first_letter(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        Some(char) => char.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn declare_struct_schema(data_struct: syn::DataStruct) -> Result<SpecStruct, syn::Error> {
    Ok(SpecStruct { example: None })
}
