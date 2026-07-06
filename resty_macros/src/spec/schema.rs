use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};
use quote::ToTokens;

use super::*;
use crate::{Reparse, ResultIterator, combine_errors};

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
        Required,
    };
    VariantArgument {
        Repr: syn::Expr,
        Example,
    };
);

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

    let mut example = None;

    let variants = data_enum
        .variants
        .iter_mut()
        .map(|variant| {
            //todo: this can def be better
            let default = lowercase_first_letter(&variant.ident.to_string());

            let Some(attr) = get_attr_once(&mut variant.attrs)? else {
                return Ok(default);
            };

            let meta: syn::MetaList = attr.meta.reparse()?;
            let args: ArgumentList<VariantArgument> = syn::parse2(meta.tokens)?;

            //todo: parse lit better
            let variant = argue!(args may have Repr)?
                .map_or(default, |(.., name)| name.to_token_stream().to_string());

            if argue!(args may have Example)?.is_some() {
                example.replace(variant.clone());
            }

            Ok(variant)
        })
        .ok()?;

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

fn declare_struct_schema(mut data_struct: syn::DataStruct) -> Result<SpecStruct, syn::Error> {
    use PropertyArgument::*;

    let mut required = Vec::new();

    let properties = data_struct
        .fields
        .iter_mut()
        .enumerate()
        .map(|(i, field)| -> Result<(String, Property), syn::Error> {
            let key = field
                .ident
                .as_ref()
                .map(|ident| ident.to_string())
                .unwrap_or_else(|| i.to_string());

            let args: Option<ArgumentList<PropertyArgument>> = get_attr_once(&mut field.attrs)?
                .map(|attr| attr.meta)
                .map(|meta| meta.require_list().cloned())
                .map_or(Ok(None), |meta| meta.map(Some))?
                .map(|meta| meta.tokens)
                .map(syn::parse2)
                .map_or(Ok(None), |meta| meta.map(Some))?;

            let description = args
                .as_ref()
                .map(|args| argue!(args may have Description))
                .map_or(Ok(None), |meta| meta.map(Some))?
                .and_then(identity)
                .map(|(.., desc)| desc.value());

            let example = args
                .as_ref()
                .map(|args| argue!(args may have Example))
                .map_or(Ok(None), |meta| meta.map(Some))?
                .and_then(identity)
                .map(|(.., example)| example.to_token_stream().to_string());

            let format = args
                .as_ref()
                .map(|args| argue!(args may have Format))
                .map_or(Ok(None), |meta| meta.map(Some))?
                .and_then(identity)
                .map(|(.., format)| format.value());

            let reference = args
                .as_ref()
                .map(|args| argue!(args may have Ref))
                .map_or(Ok(None), |meta| meta.map(Some))?
                .and_then(identity)
                .map(|(.., r)| PropertyType::Ref(r.to_string()));

            let required_arg = args
                .as_ref()
                .map(|args| argue!(args may have Required))
                .map_or(Ok(None), |meta| meta.map(Some))?
                .and_then(identity)
                .is_some();

            let (ty, infered_meta) = get_ty_ident(&field.ty)?;

            let format = format.and(infered_meta.format);
            let ty = reference.unwrap_or(ty);

            if infered_meta.required | required_arg {
                required.push(key.clone());
            }

            Ok((
                key,
                Property {
                    ty,
                    meta: PropertyMeta {
                        format,
                        example,
                        description,
                        items: infered_meta.items,
                    },
                },
            ))
        })
        .ok()?
        .into_iter()
        .fold(HashMap::new(), |mut map, (k, v)| {
            map.insert(k, v);
            map
        });

    Ok(SpecStruct {
        properties,
        required,
    })
}

struct InferedPropertyMeta {
    format: Option<String>,
    items: Option<PropertyType>,
    required: bool,
}

const NO_INFERED_META: InferedPropertyMeta = InferedPropertyMeta {
    format: None,
    required: true,
    items: None,
};

fn get_ty_ident(ty: &syn::Type) -> Result<(PropertyType, InferedPropertyMeta), syn::Error> {
    let path = match ty {
        syn::Type::Path(type_path) => &type_path.path,
        _ => {
            return Err(syn::Error::new_spanned(
                ty,
                "Only Path types are supported for Schemas",
            ));
        }
    };

    let segment = path
        .segments
        .last()
        .map_or(Err(syn::Error::new_spanned(ty, "Empty Path?")), Ok)?;

    let inner = match &segment.arguments {
        syn::PathArguments::AngleBracketed(a) => a.args.iter().find_map(|a| match a {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }),
        _ => None,
    }
    .ok_or_else(|| syn::Error::new_spanned(ty, "expected an inner type"));

    match segment.ident.to_string().as_str() {
        stringify!(Option) => {
            let ty = get_ty_ident(inner?)?;
            Ok((
                ty.0,
                InferedPropertyMeta {
                    format: ty.1.format,
                    items: ty.1.items,
                    required: false,
                },
            ))
        }
        stringify!(Vec) => Ok((
            PropertyType::Type("array".to_string()),
            InferedPropertyMeta {
                format: None,
                items: Some(get_ty_ident(inner?)?.0),
                required: true,
            },
        )),
        stringify!(String) => Ok((PropertyType::Type("string".to_string()), NO_INFERED_META)),
        stringify!(i32) => Ok((
            PropertyType::Type("integer".to_string()),
            InferedPropertyMeta {
                format: Some(String::from("integer32")),
                required: true,
                items: None,
            },
        )),
        stringify!(i64) => Ok((
            PropertyType::Type("integer".to_string()),
            InferedPropertyMeta {
                format: Some(String::from("integer64")),
                required: true,
                items: None,
            },
        )),
        stringify!(f32) => Ok((
            PropertyType::Type("number".to_string()),
            InferedPropertyMeta {
                format: Some(String::from("float32")),
                required: true,
                items: None,
            },
        )),
        stringify!(f64) => Ok((
            PropertyType::Type("number".to_string()),
            InferedPropertyMeta {
                format: Some(String::from("float64")),
                required: true,
                items: None,
            },
        )),
        stringify!(bool) => Ok((PropertyType::Type("boolean".to_string()), NO_INFERED_META)),
        ty_ident => Ok((PropertyType::Ref(ty_ident.to_string()), NO_INFERED_META)),
    }
}
