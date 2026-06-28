use proc_macro::TokenStream;
use quote::ToTokens;

use crate::compile_error;

pub type MacroArguments = Vec<MacroArgument>;
pub enum MacroArgument {
    Single(syn::Ident, syn::Expr),
    List(syn::Ident, Vec<syn::Expr>),
}

impl MacroArgument {
    pub fn single(&self) -> Result<&syn::Expr, syn::Error> {
        match self {
            MacroArgument::Single(.., expr) => Ok(expr),
            MacroArgument::List(ident, expr) => match expr.len() <= 1 {
                true => expr.get(0).ok_or(syn::Error::new(
                    ident.span(),
                    "Expected a single parameter, got an empty list",
                )),
                false => Err(syn::Error::new(
                    ident.span(),
                    "Expected a single parameter, got a list",
                )),
            },
        }
    }

    pub fn list(&self) -> Result<&Vec<syn::Expr>, syn::Error> {
        match self {
            MacroArgument::Single(ident, ..) => Err(syn::Error::new(
                ident.span(),
                "Expected a list parameter, got a single",
            )),
            MacroArgument::List(.., expr) => Ok(expr),
        }
    }

    pub fn ident(&self) -> &syn::Ident {
        match self {
            MacroArgument::Single(ident, ..) => ident,
            MacroArgument::List(ident, ..) => ident,
        }
    }
}

pub fn args(args: TokenStream) -> Result<MacroArguments, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated;

    let args = syn::parse::Parser::parse(parser, args)?;

    let result: Vec<_> = args
        .into_iter()
        .filter_map(|meta| match meta {
            syn::Meta::Path(..) => None,
            syn::Meta::List(syn::MetaList { tokens, path, .. }) => {
                Some(path.require_ident().and_then(|ident| {
                    Ok(meta_list(tokens.into())
                        .map(|list| MacroArgument::List(ident.clone(), list))?)
                }))
            }
            syn::Meta::NameValue(meta_name_value) => Some(
                meta_name_value
                    .path
                    .require_ident()
                    .map(|ident| MacroArgument::Single(ident.clone(), meta_name_value.value)),
            ),
        })
        .collect();

    let (args, errors) = collect_errors(result.into_iter());

    combine_errors(errors)?;

    Ok(args)
}

pub fn methods<'a>(args: &'a MacroArguments) -> Result<&'a Vec<syn::Expr>, syn::Error> {
    required_argument(args, "Method")?.list()
}

pub fn path_override(args: &MacroArguments) -> Result<Option<String>, syn::Error> {
    let Some(path) = optional_argument(args, "Path")? else {
        return Ok(None);
    };

    let path = path.single()?;

    let lit: syn::LitStr = syn::parse(path.to_token_stream().into())?;

    Ok(Some(lit.value()))
}

pub fn static_headers(args: &MacroArguments) -> Result<Vec<syn::Expr>, syn::Error> {
    let headers = repeatable_argument(args, "Header")
        .into_iter()
        .map(|header| header.list().map(|list| (header.ident(), list)))
        .collect::<Vec<_>>()
        .ok()?
        .into_iter()
        .map(|(ident, header)| match header.len() {
            2 => syn::parse(quote::quote! {(#(#header),*)}.into()),
            n => Err(syn::Error::new(
                ident.span(),
                format!("Expected 2 arguments, got {n}"),
            )),
        })
        .collect::<Vec<_>>()
        .ok()?;

    Ok(headers)
}

pub fn router(args: &MacroArguments) -> Result<syn::Path, syn::Error> {
    let Some(router) = optional_argument(args, "Router")? else {
        return syn::parse_str("super::__RESTY__ROUTER");
    };

    let router = router.single()?;

    syn::parse(router.into_token_stream().into())
}

pub fn meta_list(tokens: TokenStream) -> Result<Vec<syn::Expr>, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;

    let args = syn::parse::Parser::parse(parser, tokens)?;

    Ok(args.into_iter().collect())
}

macro_rules! parse_derive_attr {
    ($attr: literal in $input:ident else $msg: literal) => {{
        let ast = ::syn::parse_macro_input!($input as ::syn::DeriveInput);
        let tokens = ast
            .attrs
            .iter()
            .find(|attr| attr.path().to_token_stream().to_string() == $attr)
            .map_or(Ok(None), |attr| attr.meta.require_list().map(Some))
            .map(|r| r.map(|list| list.tokens.to_token_stream().into()));

        let tokens = match tokens {
            Ok(v) => v,
            Err(err) => return compile_error(err).to_compile_error().into(),
        };

        let Some(tokens) = tokens else {
            return compile_error($msg).to_compile_error().into();
        };

        (
            ::syn::parse_macro_input!(tokens as syn::Path),
            ast.ident,
            ast.generics,
        )
    }};
}

fn repeatable_argument<'a>(args: &'a MacroArguments, arg: &str) -> Vec<&'a MacroArgument> {
    args.iter()
        .filter_map(|macro_arg| match macro_arg.ident().to_string() == arg {
            true => Some(macro_arg),
            false => None,
        })
        .collect()
}

fn optional_argument<'a>(
    args: &'a MacroArguments,
    arg: &str,
) -> Result<Option<&'a MacroArgument>, syn::Error> {
    let args = repeatable_argument(args, arg);

    args.iter()
        .skip(1)
        .map(|arg| arg.ident())
        .map(|ident| {
            syn::Error::new(
                ident.span(),
                format!("Expected at most one `{}` parameter", ident.to_string()),
            )
        })
        .collect::<Vec<_>>()
        .ok()?;

    Ok(args.get(0).map(|arg| *arg))
}

fn required_argument<'a>(
    args: &'a MacroArguments,
    arg: &str,
) -> Result<&'a MacroArgument, syn::Error> {
    optional_argument(args, arg)?
        .ok_or_else(|| compile_error(format!("Missing required argument: {arg}")))
}

trait ResultVector<T> {
    fn ok(self) -> Result<Vec<T>, syn::Error>;
}

impl<T> ResultVector<T> for Vec<Result<T, syn::Error>> {
    fn ok(self) -> Result<Vec<T>, syn::Error> {
        let (values, errors) = collect_errors(self.into_iter());
        combine_errors(errors)?;
        Ok(values)
    }
}

impl ResultVector<()> for Vec<syn::Error> {
    fn ok(self) -> Result<Vec<()>, syn::Error> {
        combine_errors(self).map(|_| Vec::new())
    }
}

fn collect_errors<V, E>(it: impl Iterator<Item = Result<V, E>>) -> (Vec<V>, Vec<E>) {
    it.fold((Vec::new(), Vec::new()), |mut collector, next| {
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
