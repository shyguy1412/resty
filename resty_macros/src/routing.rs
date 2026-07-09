use std::{convert::identity, path::PathBuf, sync::PoisonError};

use proc_macro::{TokenStream, TokenTree};
use quote::format_ident;

use crate::spec;
use proc_macro_argue::{ArgumentList, argue};

argue! {
    RouterArgument {
        FileBased: syn::LitStr,
        Meta: ArgumentList<syn::MetaList>
    }
}

#[derive(Debug)]
struct Router {
    basepath: Option<PathBuf>,
    ident: String,
}

pub fn router(args: TokenStream, body: TokenStream) -> Result<TokenStream, syn::Error> {
    use RouterArgument::*;

    let args: ArgumentList<RouterArgument> = syn::parse(args)?;

    if let Some(meta) = argue!(args may have Meta)? {
        spec::apply_meta(meta)?;
    };

    let static_decl = parse_static_decl(body)?;
    let static_decl_ident = &static_decl.ident;

    let basepath_lit = argue!(args may have FileBased)?.map(|(.., val)| val);
    let basepath_str = basepath_lit.map(syn::LitStr::value);
    let basepath = basepath_str
        .as_ref()
        .map(api_path)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?;

    if basepath_lit.is_some() && !basepath.is_some() {
        //Error message for rusty-analyzer issues
        return Err(syn::Error::new_spanned(
            basepath_lit,
            "Could not resolve base path for path routing. \
            This is likely due to rust-analyzer not supporting Span::local_file() in proc-macros. \
            If this error only appears in rust-analyzer messages set the RESTY_PATH_ROUTING_HINT \
            environment variable to the base path from which to resolve this macro invocation.",
        ));
    }

    let (paths, idents) = basepath
        .as_ref()
        .map_or_else(|| Ok((Vec::new(), Vec::new())), modules)
        .map_err(|e| syn::Error::new_spanned(basepath_lit, e))?;

    ROUTERS
        .lock()
        .map_or_else(std::sync::PoisonError::into_inner, identity)
        .push(Router {
            basepath,
            ident: static_decl_ident.to_string(),
        });

    Ok(quote::quote! {
        #[allow(non_snake_case)]
        #[path = #basepath_str]
        mod #static_decl_ident {

            use ::resty::__private::*;
            #[doc(hidden)]
            #[linkme::distributed_slice]
            #[linkme(crate = linkme)]
            pub static #static_decl_ident: [::resty::RouteSlice];

            #(
                #[path = #paths]
                mod #idents;
            )*

        }
        #static_decl
    }
    .into())
}

static ROUTERS: std::sync::Mutex<Vec<Router>> = std::sync::Mutex::new(Vec::new());

fn parse_static_decl(body: TokenStream) -> Result<syn::ItemStatic, syn::Error> {
    let mut body: Vec<TokenTree> = body.into_iter().collect();
    let static_decl_ident = body
        .get(1)
        .and_then(|tt| match tt {
            TokenTree::Ident(ident) => {
                Some(syn::Ident::new(&ident.to_string(), ident.span().into()))
            }
            _ => None,
        })
        .ok_or(syn::Error::new(
            body.get(1)
                .or(body.get(0))
                .expect("Must have macro input")
                .span()
                .into(),
            "Expected Ident",
        ))?;

    if let Some(last) = body.last()
        && last.to_string() != ";"
    {
        return Err(syn::Error::new(last.span().into(), "Expected a ;"));
    };

    body.pop();

    let mut stream = TokenStream::new();

    stream.extend(body);
    stream.extend(Into::<TokenStream>::into(
        quote::quote! {= ::std::sync::LazyLock::new(||::resty::Router::new(&#static_decl_ident::#static_decl_ident));},
    ));

    syn::parse(stream)
}

fn api_path(path: &String) -> Result<PathBuf, syn::Error> {
    let path = path.strip_prefix("./").unwrap_or(&path);
    let path = path.strip_prefix("/").unwrap_or(&path);

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();

    source_file
        .or_else(|| {
            let hint = std::env::var("RESTY_PATH_ROUTING_HINT").ok()?;
            Some(PathBuf::from(hint))
        })
        .ok_or(syn::Error::new(
            span.into(),
            "Could not resolve base path for path routing. \
        This is likely due to rust-analyzer not supporting Span::local_file() in proc-macros. \
        If this error only appears in rust-analyzer messages set the RESTY_PATH_ROUTING_HINT \
        environment variable to the file path where this macro is invoked",
        ))
        .map(|p| {
            p.parent()
                .expect("Can not get parent of call_site")
                .join(path)
        })
}

fn modules(base_path: &std::path::PathBuf) -> std::io::Result<(Vec<String>, Vec<syn::Ident>)> {
    let files = visit_dirs(base_path)?;

    Ok(files
        .into_iter()
        .map(|d| {
            d.path()
                .strip_prefix(base_path)
                .expect("Guranteed by earlier path reads")
                .to_string_lossy()
                .to_string()
        })
        .map(|p| {
            (
                p.clone(),
                format_ident!(
                    "__endpoint_{}",
                    &p.replace("/", "_")
                        .replace("[", "")
                        .replace("]", "_")
                        .replace("%", "")
                        .strip_suffix(".rs")
                        .unwrap()
                ),
            )
        })
        .collect())
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &std::path::Path) -> std::io::Result<Vec<std::fs::DirEntry>> {
    let mut files = vec![];
    fn visit_dirs(
        dir: &std::path::Path,
        files: &mut Vec<std::fs::DirEntry>,
    ) -> std::io::Result<()> {
        // if std::fs::read_dir(dir).is_err() {
        //     panic!("{:?}", &std::env::current_dir())
        // }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else {
                files.push(entry);
            }
        }
        Ok(())
    }
    visit_dirs(dir, &mut files)?;
    Ok(files)
}

fn get_endpoint_path() -> Option<String> {
    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();
    source_file
        .as_ref()
        .and_then(|path| {
            ROUTERS.lock().ok().and_then(|paths| {
                paths.iter().find_map(|p| match &p.basepath {
                    Some(p) => path.strip_prefix(p).ok(),
                    None => None,
                })
            })
        })
        .and_then(|path| path.to_str())
        .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
        .and_then(|path| path.strip_suffix("mod").or(Some(path)))
        .and_then(|path| path.strip_suffix("/").or(Some(path)))
        .map(|s| s.to_string())
}

pub fn default_router() -> Result<syn::Path, syn::Error> {
    let source_file = proc_macro::Span::call_site()
        .local_file()
        .unwrap_or("".into());

    if source_file == *"" {
        return syn::parse_str("super::RUST_ANALYZER_PLACEHOLDER");
    }

    ROUTERS
        .lock()
        .map_or_else(PoisonError::into_inner, identity)
        .iter()
        .find(|router| match &router.basepath {
            Some(p) => source_file.strip_prefix(p).is_ok(),
            None => false,
        })
        .and_then(|router| syn::parse_str(&format!("super::{}", router.ident)).ok())
        .ok_or(syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Can not infer Router. Maybe you are missing a Router directive?",
        ))
}

pub fn default_route() -> Result<Vec<String>, syn::Error> {
    let path = get_endpoint_path();

    let segments = path
        .as_ref()
        .map(|v| v.as_str())
        .or_else(|| match proc_macro::Span::call_site().local_file() {
            None => Some("<rust-analyzer has not yet implemented Span::local_file>"),
            Some(..) => None,
        })
        .map(|p| p.split("/"));

    let Some(segments) = segments else {
        return Err(syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Can not infer route. Maybe you are missing a Route directive?",
        ));
    };

    Ok(segments.map(|s| s.to_string()).collect())
}
