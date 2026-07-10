use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, argue};
use serde::Serialize;
use syn::{MetaList, ext::IdentExt};

use crate::spec::{self, SPEC, Spec};

argue! {
    MetaArgument {
        Method: syn::Ident,
        Tag: syn::LitStr,
        Summary: syn::LitStr,
        Description: syn::LitStr,
        Request: ArgumentList<RequestArgument>,
        Response: ResponseArgument,
        Security: ArgumentList<SecurityArgument>
    };
    RequestArgument {
        Description: syn::LitStr,
        Schema: SchemaArgument,
        Required
    };
    ResponseArgument(ResponseType, syn::token::Comma, syn::LitStr);
    SchemaArgument(syn::LitStr, syn::token::Comma, syn::Ident);
    SecurityArgument {
        Name: syn::LitStr,
        Scope: syn::LitStr
    }
}

enum ResponseType {
    Path(syn::Path),
    Code(syn::LitInt),
}

impl syn::parse::Parse for ResponseType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use ResponseType::*;
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitInt) {
            Ok(Code(input.parse::<syn::LitInt>()?))
        } else {
            Ok(Path(input.parse::<syn::Path>()?))
        }
    }
}

pub fn add_path(
    args: TokenStream, // (_, args): (&syn::Ident, &ArgumentList<syn::MetaList>),
                       // route: &Vec<String>,
                       // method: &ArgumentList<syn::Expr>,
) -> Result<(), syn::Error> {
    let args: ArgumentList<syn::Meta> = syn::parse(args)?;
    let method_arg = args
        .iter()
        .filter_map(|arg| match arg {
            syn::Meta::List(meta_list) => Some(meta_list),
            _ => None,
        })
        .find_map(|list| {
            match list
                .path
                .get_ident()
                .map(ToString::to_string)
                .as_ref()
                .map(String::as_str)
            {
                Some("Method") => Some(list.tokens.clone()),
                _ => None,
            }
        })
        .unwrap_or(proc_macro2::TokenStream::new());
    let methods: ArgumentList<syn::Ident> = syn::parse2(method_arg)?;

    let meta_arg = args
        .iter()
        .filter_map(|arg| match arg {
            syn::Meta::List(meta_list) => Some(meta_list),
            _ => None,
        })
        .find_map(|list| {
            match list
                .path
                .get_ident()
                .map(ToString::to_string)
                .as_ref()
                .map(String::as_str)
            {
                Some("Meta") => Some(list.tokens.clone()),
                _ => None,
            }
        })
        .unwrap_or(proc_macro2::TokenStream::new());

    let meta: ArgumentList<MetaArgument> = syn::parse2(meta_arg)?;

    let mut spec = SPEC.get();
    let path = spec
        .paths
        .entry("/pet".to_string())
        .or_insert_with(|| super::Path {
            methods: BTreeMap::new(),
        });

    path.methods.insert(
        "get".to_string(),
        super::Method {
            tags: vec!["pet".to_string()],
            summary: None,
            description: None,
            operation_id: "".to_string(),
            parameters: vec![],
            responses: vec![super::ResponseType::Ref("Pet".to_string(), "".to_string())],
            security: vec![],
        },
    );

    Ok(())
}
