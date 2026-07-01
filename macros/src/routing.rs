use std::path::PathBuf;

use proc_macro::{TokenStream, TokenTree};
use quote::format_ident;

use proc_macro_argue::{ArgumentList, argue};

argue! {
    RouterArgument {
        FileBased: syn::LitStr,
        Meta: ArgumentList<MetaArgument>
    }
    MetaArgument {
        Description: syn::LitStr,
        Title: syn::LitStr,
        Version: syn::LitStr
    }
}
pub fn router(args: TokenStream, body: TokenStream) -> Result<TokenStream, syn::Error> {
    use RouterArgument::*;

    let args: ArgumentList<RouterArgument> = syn::parse(args)?;

    let basepath = argue!(args may have FileBased).map(|(.., val)| val);
    let (paths, idents) = basepath.map_or_else(|| Ok((Vec::new(), Vec::new())), paths)?;
    let basepath = basepath.map(|p| p.value()).unwrap_or("./".to_owned());

    let static_decl = parse_static_decl(body)?;
    let static_decl_ident = &static_decl.ident;

    Ok(quote::quote! {
        #[allow(non_snake_case)]
        #[path = #basepath]
        mod #static_decl_ident {
            use ::resty::__private::*;
            #[doc(hidden)]
            #[linkme::distributed_slice]
            #[linkme(crate = linkme)]
            pub static __RESTY__ROUTER: [::resty::RouteSlice];

            #(
                #[path = #paths]
                mod #idents;
            )*

        }
        #static_decl
    }
    .into())
}

static BASE_PATHS: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

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
        quote::quote! {= ::std::sync::LazyLock::new(||::resty::Router::new(&#static_decl_ident::__RESTY__ROUTER));},
    ));

    syn::parse(stream)
}

fn paths(path_litstr: &syn::LitStr) -> Result<(Vec<String>, Vec<syn::Ident>), syn::Error> {
    let path = path_litstr.value();
    let path = path.strip_prefix("./").unwrap_or(&path);
    let path = path.strip_prefix("/").unwrap_or(&path);

    let span = proc_macro::Span::call_site();
    let source_file = span.local_file();

    let api_path = source_file
        .as_ref()
        .and_then(|file| file.parent())
        .map(|p| p.into())
        .or_else(|| {
            let hint = std::env::var("RESTY_PATH_ROUTING_HINT").ok()?;
            Some(PathBuf::from(hint))
        })
        .map(|p| p.join(path))
        .ok_or_else(|| syn::Error::new(
        span.into(),
        "Could not resolve base path for path routing.
            This is likely due to rust-analyzer not supporting Span::local_file() in proc-macros.
            If this error only appears in rust-analyzer messages set the RESTY_PATH_ROUTING_HINT
            environment variable to the base path from which to resolve your resty::use_path_routing macro invocation.",
))?;

    BASE_PATHS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(api_path.clone());

    let files = visit_dirs(&api_path).map_err(|e| syn::Error::new(path_litstr.span(), e))?;

    Ok(files
        .into_iter()
        .map(|d| {
            d.path()
                .strip_prefix(&api_path)
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
            crate::routing::BASE_PATHS
                .lock()
                .ok()
                .and_then(|paths| paths.iter().find_map(|p| path.strip_prefix(p).ok()))
        })
        .and_then(|path| path.to_str())
        .and_then(|path| path.strip_suffix(".rs").or(Some(path)))
        .and_then(|path| path.strip_suffix("mod").or(Some(path)))
        .map(|s| s.to_string())
}

pub fn default_router() -> Result<syn::Path, syn::Error> {
    match get_endpoint_path().is_some()
        //workaround for rust-analyzer
            || proc_macro::Span::call_site().local_file().is_none()
    {
        true => syn::parse_str("super"),
        false => Err(syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Can not infer Router. Maybe you are missing a Router directive?",
        )),
    }
}

pub fn default_route() -> Result<Vec<String>, syn::Error> {
    let path = get_endpoint_path();
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
        return Err(syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Can not infer route. Maybe you are missing a Route directive?",
        ));
    };

    Ok(segments.map(|s| s.to_string()).skip(1).collect())
}
