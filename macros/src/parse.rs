use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::ToTokens;

pub fn parse_args(args: TokenStream) -> Vec<(String, Vec<syn::Expr>)> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated;

    let Ok(args) = syn::parse::Parser::parse(parser, args) else {
        return vec![];
    };

    let args: Vec<_> = args
        .into_iter()
        .filter_map(|meta| match meta {
            syn::Meta::Path(..) => None,
            syn::Meta::List(syn::MetaList { tokens, path, .. }) => Some((
                path.to_token_stream().to_string(),
                parse_meta_list(tokens.into()),
            )),
            syn::Meta::NameValue(meta_name_value) => Some((
                meta_name_value.path.to_token_stream().to_string(),
                vec![meta_name_value.value.clone()],
            )),
        })
        .collect();
    args
}

pub fn parse_methods(args: &Vec<(String, Vec<syn::Expr>)>) -> &Vec<syn::Expr> {
    &args
        .iter()
        .find(|meta| meta.0 == "Method")
        .expect("Missing required argument: Method")
        .1
}

pub fn parse_path_override(args: &Vec<(String, Vec<syn::Expr>)>) -> Option<String> {
    args.iter()
        .find_map(|meta| match meta.0 == "Path" {
            true => Some(meta.1.get(0)?),
            false => None,
        })
        .and_then(|expr| match expr {
            syn::Expr::Lit(expr_lit) => Some(&expr_lit.lit),
            _ => None,
        })
        .and_then(|lit| match lit {
            syn::Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        })
}

pub fn parse_static_headers(args: &Vec<(String, Vec<syn::Expr>)>) -> Vec<syn::Expr> {
    args.iter()
        .filter_map(|(key, value)| match key == "Header" {
            true => Some(syn::parse(quote::quote! {(#(#value),*)}.into()).ok()?),
            false => None,
        })
        .collect()
}

pub fn parse_meta_list(tokens: TokenStream) -> Vec<syn::Expr> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;

    syn::parse::Parser::parse(parser, tokens)
        .map(|l| l.into_iter().collect())
        .unwrap_or(vec![])
}
