use proc_macro::TokenStream;
use quote::ToTokens;

pub fn args(args: TokenStream) -> Vec<(String, Vec<syn::Expr>)> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated;

    let Ok(args) = syn::parse::Parser::parse(parser, args) else {
        return vec![];
    };

    let args: Vec<_> = args
        .into_iter()
        .filter_map(|meta| match meta {
            syn::Meta::Path(..) => None,
            syn::Meta::List(syn::MetaList { tokens, path, .. }) => {
                Some((path.to_token_stream().to_string(), meta_list(tokens.into())))
            }
            syn::Meta::NameValue(meta_name_value) => Some((
                meta_name_value.path.to_token_stream().to_string(),
                vec![meta_name_value.value.clone()],
            )),
        })
        .collect();
    args
}

pub fn methods(args: &Vec<(String, Vec<syn::Expr>)>) -> &Vec<syn::Expr> {
    &args
        .iter()
        .find(|meta| meta.0 == "Method")
        .expect("Missing required argument: Method")
        .1
}

pub fn path_override(args: &Vec<(String, Vec<syn::Expr>)>) -> Option<String> {
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

pub fn static_headers(args: &Vec<(String, Vec<syn::Expr>)>) -> Vec<syn::Expr> {
    args.iter()
        .filter_map(|(key, value)| match key == "Header" {
            true => Some(syn::parse(quote::quote! {(#(#value),*)}.into()).ok()?),
            false => None,
        })
        .collect()
}

pub fn meta_list(tokens: TokenStream) -> Vec<syn::Expr> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;

    syn::parse::Parser::parse(parser, tokens)
        .map(|l| l.into_iter().collect())
        .unwrap_or(vec![])
}

macro_rules! parse_derive_attr {
    ($attr: literal in $input:ident else $msg: literal) => {{
        let ast = parse_macro_input!($input as syn::DeriveInput);
        let tokens = ast
            .attrs
            .iter()
            .find(|attr| attr.path().to_token_stream().to_string() == $attr)
            .map_or(Ok(None), |attr| attr.meta.require_list().map(Some))
            .map(|r| r.map(|list| list.tokens.to_token_stream().into()));

        let tokens = match tokens {
            Ok(v) => v,
            Err(err) => return compile_error(proc_macro2::Span::call_site(), err),
        };

        let Some(tokens) = tokens else {
            return compile_error(proc_macro2::Span::call_site(), $msg);
        };

        (
            parse_macro_input!(tokens as syn::Path),
            ast.ident,
            ast.generics,
        )
    }};
}
