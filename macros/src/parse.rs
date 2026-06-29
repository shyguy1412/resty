use std::ops::BitOr;

use proc_macro::TokenStream;
use quote::{ToTokens, format_ident};
use syn::{LitStr, spanned::Spanned};

use crate::{
    compile_error,
    routing::{self, get_endpoint_path},
};

pub type MacroArguments = Vec<MacroArgument>;
pub enum MacroArgument {
    Single(syn::Ident, syn::Expr),
    List(syn::Ident, Vec<syn::Expr>),
}

#[repr(u16)]
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum MacroArgumentType {
    Route       = 0b00000001,
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
            "Route" => Ok(MacroArgumentType::Route),
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

impl From<MacroArgumentType> for &'static str {
    fn from(value: MacroArgumentType) -> Self {
        match value {
            MacroArgumentType::Route => "Route",
            MacroArgumentType::Router => "Router",
            MacroArgumentType::Method => "Method",
            MacroArgumentType::Header => "Header",
            MacroArgumentType::Accepts => "Accepts",
            MacroArgumentType::Responds => "Responds",
            MacroArgumentType::Summary => "Summary",
            MacroArgumentType::Description => "Description",
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
    required_argument(args, MacroArgumentType::Method).map(|methods| methods.list())
}

pub fn route(args: &MacroArguments) -> Result<Vec<String>, syn::Error> {
    let path = optional_argument(args, MacroArgumentType::Route)?
        .map_or(Ok(None), |path| path.single().map(Some))?;
    let path: Option<LitStr> = path.map_or(Ok(None), |p| p.reparse())?;
    let path = path.map(|p| p.value());
    let path = path.or_else(get_endpoint_path);
    let segments = path
        .as_ref()
        .map(|v| v.as_str())
        // .and_then(|p| p.strip_prefix("/").or(Some(p)))
        .or_else(|| match proc_macro::Span::call_site().local_file() {
            None => Some("<rust-analyzer has not yet implemented Span::local_file>"),
            Some(..) => None,
        })
        .map(|p| p.split("/"));

    let Some(segments) = segments else {
        return Err(compile_error(
            "Can not infer route. Maybe you are missing a Route directive?",
        ));
    };

    Ok(segments.map(|s| s.to_string()).skip(1).collect())
}

pub fn static_headers(args: &MacroArguments) -> Result<Vec<syn::Expr>, syn::Error> {
    let headers = repeatable_argument(args, MacroArgumentType::Header)
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
    optional_argument(args, MacroArgumentType::Accepts)?
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
    Ok(repeatable_argument(args, MacroArgumentType::Responds)
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
    let Some(router) = optional_argument(args, MacroArgumentType::Router)? else {
        return match routing::get_endpoint_path().is_some()
        //workaround for rust-analyzer
            || proc_macro::Span::call_site().local_file().is_none()
        {
            true => syn::parse_str("super::__RESTY__ROUTER"),
            false => Err(syn::Error::new(
                proc_macro::Span::call_site().into(),
                "Can not infer Router. Maybe you are missing a Router directive?",
            )),
        };
    };

    let ident = router.ident();
    let router = router.single()?;
    let mut router: syn::Path = router.reparse()?;

    router
        .segments
        .last_mut()
        .ok_or(syn::Error::new(ident.span(), "Expected a Path to a Router"))
        .map(|segment| segment.ident = format_ident!("__RESTY__ROUTER_{}", &segment.ident))?;

    Ok(router)
}

pub fn meta_list(tokens: TokenStream) -> Result<Vec<syn::Expr>, syn::Error> {
    let parser = syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated;

    let args = syn::parse::Parser::parse(parser, tokens)?;

    Ok(args.into_iter().collect())
}

fn repeatable_argument<'a>(
    args: &'a MacroArguments,
    arg: MacroArgumentType,
) -> Vec<&'a MacroArgument> {
    args.iter()
        .filter_map(|macro_arg| {
            match macro_arg.ident().to_string().as_str() == Into::<&str>::into(arg) {
                true => Some(macro_arg),
                false => None,
            }
        })
        .collect()
}

fn optional_argument<'a>(
    args: &'a MacroArguments,
    arg: MacroArgumentType,
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
    arg: MacroArgumentType,
) -> Result<&'a MacroArgument, syn::Error> {
    optional_argument(args, arg)?.ok_or_else(|| {
        compile_error(format!(
            "Missing required argument: {}",
            Into::<&str>::into(arg)
        ))
    })
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
pub trait Reparse: quote::ToTokens {
    fn reparse<T: syn::parse::Parse>(&self) -> Result<T, syn::Error> {
        syn::parse(self.to_token_stream().into())
    }
}

impl<T: quote::ToTokens> Reparse for T {}

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
