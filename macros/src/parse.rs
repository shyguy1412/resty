use std::ops::BitOr;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::compile_error;

pub type MacroArguments = Vec<MacroArgument>;
pub enum MacroArgument {
    Single(syn::Ident, syn::Expr),
    List(syn::Ident, Vec<syn::Expr>),
}

#[repr(u16)]
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum MacroArgumentType {
    Path        = 0b00000001,
    Router      = 0b00000010,
    Method      = 0b00000100,
    Header      = 0b00001000,
    Accepts     = 0b00010000,
    Responds    = 0b00100000,
    Summary     = 0b01000000,
    Description = 0b10000000,
}

impl BitOr for MacroArgumentType {
    type Output = u16;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u16 | rhs as u16
    }
}

impl BitOr<u16> for MacroArgumentType {
    type Output = u16;

    fn bitor(self, rhs: u16) -> Self::Output {
        self as u16 | rhs
    }
}

impl BitOr<MacroArgumentType> for u16 {
    type Output = u16;

    fn bitor(self, rhs: MacroArgumentType) -> Self::Output {
        self | rhs as u16
    }
}

impl TryFrom<&syn::Ident> for MacroArgumentType {
    type Error = syn::Error;

    fn try_from(ident: &syn::Ident) -> Result<Self, Self::Error> {
        match ident.to_string().as_str() {
            "Path" => Ok(MacroArgumentType::Path),
            "Router" => Ok(MacroArgumentType::Router),
            "Method" => Ok(MacroArgumentType::Method),
            "Header" => Ok(MacroArgumentType::Header),
            "Accepts" => Ok(MacroArgumentType::Accepts),
            "Responds" => Ok(MacroArgumentType::Responds),
            "Summary" => Ok(MacroArgumentType::Summary),
            "Description" => Ok(MacroArgumentType::Description),
            _ => Err(syn::Error::new(ident.span(), format!("Unknown Argument"))),
        }
    }
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

    pub fn list(&self) -> Vec<&syn::Expr> {
        match self {
            MacroArgument::Single(.., expr) => vec![expr],
            MacroArgument::List(.., expr) => Vec::from_iter(expr),
        }
    }

    pub fn ident(&self) -> &syn::Ident {
        match self {
            MacroArgument::Single(ident, ..) => ident,
            MacroArgument::List(ident, ..) => ident,
        }
    }

    pub fn validate(self, fields: u16) -> Result<Self, syn::Error> {
        let ident = self.ident();
        let ty: MacroArgumentType = ident.try_into()?;
        if (ty as u16) & fields == 0 {
            return Err(syn::Error::new(ident.span(), "Invalid Argument"));
        }
        Ok(self)
    }
}

pub fn args(args: TokenStream, fields: u16) -> Result<MacroArguments, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated;

    syn::parse::Parser::parse(parser, args)?
        .into_iter()
        .map(|meta| match meta {
            syn::Meta::Path(path) => Err(syn::Error::new(path.span(), "Invalid Argument")),
            syn::Meta::List(syn::MetaList { tokens, path, .. }) => {
                path.require_ident().and_then(|ident| {
                    Ok(meta_list(tokens.into())
                        .map(|list| MacroArgument::List(ident.clone(), list))?)
                })
            }
            syn::Meta::NameValue(meta_name_value) => meta_name_value
                .path
                .require_ident()
                .map(|ident| MacroArgument::Single(ident.clone(), meta_name_value.value)),
        })
        .ok()?
        .into_iter()
        .map(|arg| arg.validate(fields))
        .ok()
}

pub fn methods<'a>(args: &'a MacroArguments) -> Result<Vec<&'a syn::Expr>, syn::Error> {
    required_argument(args, "Method").map(|methods| methods.list())
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
        .map(|header| (header.ident(), header.list()))
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

pub fn accepts(args: &MacroArguments) -> Result<Vec<syn::ExprBlock>, syn::Error> {
    optional_argument(args, "Accepts")?
        .map(|arg| arg.list())
        .unwrap_or(vec![])
        .into_iter()
        .map(|accepts| {
            syn::parse(
                quote::quote! {
                    {
                        fn __doc_validate<T: ::resty::__private::Public>() {
                            __doc_validate::<#accepts>();
                        };
                    }
                }
                .into(),
            )
        })
        .ok()
}

pub fn responds(args: &MacroArguments) -> Result<Vec<syn::ExprBlock>, syn::Error> {
    Ok(repeatable_argument(args, "Responds")
        .into_iter()
        .map(|arg| (arg.list(), arg.ident()))
        .map(|(list, ident)| match list.len() {
            2 => Ok((list[0], list[1])),
            n => Err(syn::Error::new(
                ident.span(),
                format!("Expected 2 arguments, got {n}"),
            )),
        })
        .ok()?
        .into_iter()
        .map(|(code, ty)| {
            Ok((
                syn::parse::<syn::LitInt>(code.to_token_stream().into())?,
                syn::parse::<syn::Path>(ty.to_token_stream().into())?,
            ))
        })
        .ok()?
        .into_iter()
        .map(|(code, ty)| {
            syn::parse(
                quote::quote! {
                    {
                        fn __doc_validate<T: ::resty::__private::Public>(code:u16) {
                            __doc_validate::<#ty>(#code);
                        };
                    }
                }
                .into(),
            )
        })
        .ok()?)
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
        .map(|ident| -> Result<(), syn::Error> {
            Err(syn::Error::new(
                ident.span(),
                format!("Expected at most one `{}` parameter", ident.to_string()),
            ))
        })
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

trait ResultIterator<T> {
    fn ok(self) -> Result<Vec<T>, syn::Error>;
}

impl<V, T: IntoIterator<Item = Result<V, syn::Error>>> ResultIterator<V> for T {
    fn ok(self) -> Result<Vec<V>, syn::Error> {
        let (values, errors) = collect_errors(self.into_iter());
        combine_errors(errors)?;
        Ok(values)
    }
}

fn collect_errors<V, E>(it: impl IntoIterator<Item = Result<V, E>>) -> (Vec<V>, Vec<E>) {
    it.into_iter()
        .fold((Vec::new(), Vec::new()), |mut collector, next| {
            match next {
                Ok(ident) => collector.0.push(ident),
                Err(err) => collector.1.push(err),
            };
            collector
        })
}

pub fn combine_errors(errors: Vec<syn::Error>) -> Result<(), syn::Error> {
    if let Some(mut error) = errors.get(0).cloned() {
        error.extend(errors.into_iter().skip(1));
        return Err(error);
    }

    return Ok(());
}

pub fn parse_derive_helper<P: syn::parse::Parse>(
    helper: &str,
    input: TokenStream,
) -> Result<(P, syn::DeriveInput), syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input)?;
    let tokens = ast
        .attrs
        .iter()
        .find(|attr| attr.path().to_token_stream().to_string() == helper)
        .map_or(Ok(None), |attr| attr.meta.require_list().map(Some))
        .map(|r| r.map(|list| list.tokens.to_token_stream().into()));

    let tokens = match tokens {
        Ok(v) => v,
        Err(err) => return Err(syn::Error::new(ast.ident.span(), err)),
    };

    let Some(tokens) = tokens else {
        return Err(syn::Error::new(
            ast.ident.span(),
            format!("{} attribute required", helper),
        ));
    };

    Ok((syn::parse(tokens)?, ast))
}
