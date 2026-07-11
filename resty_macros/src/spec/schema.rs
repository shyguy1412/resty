use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};
use quote::ToTokens;

use super::*;
use crate::ResultIterator;

pub fn schema_macro_impl(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input: syn::DeriveInput = syn::parse(input)?;
    let ident = &input.ident;

    declare_schema(&input)?;

    Ok(quote::quote! {
        impl ::resty::__private::Schema for #ident{}
    }
    .into())
}

argue!(
    SchemaArgument {
        Description: syn::LitStr,
        Name: syn::Ident,
        Type: syn::LitStr,
    };
    PropertyArgument {
        Example: syn::Lit,
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

fn declare_schema(input: &syn::DeriveInput) -> Result<(), syn::Error> {
    use SchemaArgument::*;
    let ident = &input.ident;
    let mut name_ident = &input.ident;

    let schema = match &input.data {
        syn::Data::Struct(data_struct) => {
            EnumOrStruct::Struct(declare_struct_schema(&data_struct)?)
        }
        syn::Data::Enum(data_enum) => EnumOrStruct::Enum(declare_enum_schema(&data_enum)?),
        syn::Data::Union(data_union) => {
            return Err(syn::Error::new_spanned(
                data_union.union_token,
                "unions are not supported for schemas",
            ));
        }
    };

    let args: ArgumentList<SchemaArgument> = get_helper_attr(&input.attrs)?
        .map(|attr| &attr.meta)
        .map(|meta| meta.require_list().cloned())
        .map_or(Ok(None), |meta| meta.map(Some))?
        .map(|meta| meta.tokens)
        .map(syn::parse2)
        .map_or(Ok(None), |meta| meta.map(Some))?
        .unwrap_or_default();

    let name = argue!(args may have Name)?.map_or(ident.to_string(), |(.., name)| {
        name_ident = name;
        name.to_string()
    });

    let description = argue!(args may have Description)?.parse(lit_value)?;

    let schema: Schema = Schema {
        schema,
        description,
    };

    let mut spec = SPEC.get();
    if spec.components.schemas.insert(name, schema).is_some() && is_io_allowed() {
        return Err(syn::Error::new_spanned(name_ident, "duplicate schema name"));
    };

    Ok(())
}

fn declare_enum_schema(data_enum: &syn::DataEnum) -> Result<EnumSpec, syn::Error> {
    use VariantArgument::*;

    let mut example = None;

    let variants = data_enum
        .variants
        .iter()
        .map(|variant| {
            //todo: this can def be better
            let default = lowercase_first_letter(&variant.ident.to_string());

            let Some(attr) = get_helper_attr(&variant.attrs)? else {
                return Ok(default);
            };

            let args: ArgumentList<VariantArgument> =
                syn::parse2(attr.meta.require_list()?.tokens.clone())?;

            //todo: parse lit better
            let variant = argue!(args may have Repr)?
                .map_or(default, |(.., name)| name.to_token_stream().to_string());

            if argue!(args may have Example)?.is_some() {
                example.replace(variant.clone());
            }

            Ok(variant)
        })
        .ok()?;

    Ok(EnumSpec {
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

fn declare_struct_schema(data_struct: &syn::DataStruct) -> Result<StructSpec, syn::Error> {
    use PropertyArgument::*;

    let mut required = Vec::new();

    let properties = data_struct
        .fields
        .iter()
        .enumerate()
        .map(
            |(i, field)| -> Result<(String, OrRef<Property>), syn::Error> {
                let ty = match &field.ty {
                    syn::Type::Path(type_path) => type_path,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &field.ty,
                            "Only Path types are supported for Schemas",
                        ));
                    }
                };

                let key = field
                    .ident
                    .as_ref()
                    .map(|ident| ident.to_string())
                    .unwrap_or_else(|| i.to_string());

                let args: ArgumentList<PropertyArgument> = get_helper_attr(&field.attrs)?
                    .map(|attr| &attr.meta)
                    .map(|meta| meta.require_list().cloned())
                    .map_or(Ok(None), |meta| meta.map(Some))?
                    .map(|meta| meta.tokens)
                    .map(syn::parse2)
                    .map_or(Ok(None), |meta| meta.map(Some))?
                    .unwrap_or_default();

                if let Some(reference) = argue!(args may have Ref)?.map(|i| {
                    OrRef::Ref(ReferenceObject {
                        component: ComponentType::Schema,
                        name: i.1.to_string(),
                    })
                }) {
                    return Ok((key, reference));
                }

                let description = argue!(args may have Description)?.parse(lit_value)?;

                let example = argue!(args may have Example)?.parse(|example| match example {
                    syn::Lit::Str(lit_str) => Ok(lit_str.value()),
                    rest => Ok(rest.to_token_stream().to_string()),
                })?;
                let format = argue!(args may have Format)?.parse(lit_value)?;

                let property_required = argue!(args may have Required)?.is_some()
                    || ty
                        .path
                        .segments
                        .last()
                        .expect("Must have at least one segment")
                        .ident
                        .to_string()
                        == "Option";

                let property_type = match path_to_property_type(ty, format)? {
                    OrRef::Val(v) => v,
                    OrRef::Ref(reference) => return Ok((key, OrRef::Ref(reference))),
                };

                if property_required {
                    required.push(key.clone());
                };

                Ok((
                    key,
                    OrRef::Val(Property {
                        example,
                        description,
                        ty: property_type,
                    }),
                ))
            },
        )
        .ok()?
        .into_iter()
        .fold(BTreeMap::new(), |mut map, (k, v)| {
            map.insert(k, v);
            map
        });

    Ok(StructSpec {
        properties,
        required,
    })
}

fn path_to_property_type(
    path: &syn::TypePath,
    format: Option<String>,
) -> Result<OrRef<PropertyType>, syn::Error> {
    let segment = path
        .path
        .segments
        .last()
        .map_or(Err(syn::Error::new_spanned(path, "Empty Path?")), Ok)?;

    let inner = match &segment.arguments {
        syn::PathArguments::AngleBracketed(a) => a.args.iter().find_map(|a| match a {
            syn::GenericArgument::Type(ty) => match ty {
                syn::Type::Path(type_path) => Some(type_path),
                _ => None,
            },
            _ => None,
        }),
        _ => None,
    }
    .ok_or_else(|| syn::Error::new_spanned(path, "expected an inner type"));

    use OrRef::*;
    use PropertyType::*;

    match segment.ident.to_string().as_str() {
        stringify!(Option) => path_to_property_type(inner?, format),
        stringify!(Vec) => Ok(Val(Array {
            items: Box::new(path_to_property_type(inner?, format)?),
        })),
        stringify!(String) => Ok(Val(Primitive {
            ty: "string",
            format,
        })),
        stringify!(i32) => Ok(Val(Primitive {
            ty: "number",
            format: Some("int23".to_string()).and(format),
        })),
        stringify!(i64) => Ok(Val(Primitive {
            ty: "number",
            format: Some("int64".to_string()).and(format),
        })),
        stringify!(f32) => Ok(Val(Primitive {
            ty: "number",
            format: Some("float".to_string()).and(format),
        })),
        stringify!(f64) => Ok(Val(Primitive {
            ty: "number",
            format: Some("double".to_string()).and(format),
        })),
        stringify!(bool) => Ok(Val(Primitive {
            ty: "boolean",
            format: format,
        })),
        ty_ident => Ok(Ref(ReferenceObject {
            component: ComponentType::Schema,
            name: ty_ident.to_string(),
        })),
    }
}

fn get_helper_attr(attrs: &Vec<syn::Attribute>) -> Result<Option<&syn::Attribute>, syn::Error> {
    get_attr_once("schema", attrs)
}
