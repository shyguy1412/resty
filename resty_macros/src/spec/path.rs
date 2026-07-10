use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};

use crate::{
    ResultIterator,
    endpoint::*,
    routing,
    spec::{RequestBody, SPEC, Spec, lit_value},
};

pub fn add_path(
    args: TokenStream, // (_, args): (&syn::Ident, &ArgumentList<syn::MetaList>),
                       // route: &Vec<String>,
                       // method: &ArgumentList<syn::Expr>,
) -> Result<(), syn::Error> {
    use crate::endpoint::HandlerArgument::*;
    let args: ArgumentList<HandlerArgument> = syn::parse(args)?;

    let route = argue!(args may have Route)?
        .parse(parse_route)?
        .map_or_else(routing::default_route, Ok)?
        .join("/");

    let methods = argue!(args may repeat Method)
        .map(|(.., m)| m.to_string().to_ascii_lowercase())
        .collect::<Vec<_>>();

    let tags = argue!(args may repeat Tag).parse_iter(lit_value).ok()?;
    let summary = argue!(args may have Summary)?.parse(lit_value)?;
    let description = argue!(args may have Description)?.parse(lit_value)?;
    let responses = argue!(args may repeat Response)
        .parse_iter(parse_response)
        .ok()?;

    let security = argue!(args may repeat Security)
        .parse_iter(parse_security)
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();
    let request_body = argue!(args may have Request)?.parse(parse_request_body)?;

    let mut spec = SPEC.get();
    let path = spec
        .paths
        .entry(format!("/{route}"))
        .or_insert_with(|| super::Path {
            methods: BTreeMap::new(),
        });

    for method in methods {
        path.methods.insert(
            method.clone(),
            super::Method {
                tags: tags.clone(),
                summary: summary.clone(),
                description: description.clone(),
                request_body: request_body.clone(),
                operation_id: format!("{method}{}", route.replace("/", "_")),
                parameters: vec![],
                responses: responses.clone(),
                security: security.clone(),
            },
        );
    }

    Ok(())
}

fn parse_response(arg: &ResponseType) -> Result<super::ResponseType, syn::Error> {
    let r = match arg {
        ResponseType::Path(path) => super::ResponseType::Ref(
            path.segments
                .last()
                .ok_or(syn::Error::new_spanned(path, "Can not parse path"))?
                .ident
                .to_string(),
        ),
        ResponseType::Code(ContentlessResponseArgument(lit_int, _, lit_str)) => {
            super::ResponseType::Raw(lit_int.base10_digits().to_string(), lit_str.value())
        }
    };

    Ok(r)
}

fn parse_request_body(arg: &ArgumentList<RequestArgument>) -> Result<RequestBody, syn::Error> {
    use crate::endpoint::RequestArgument::*;

    let description = argue!(arg may have Description)?.parse(lit_value)?;
    let required = argue!(arg may have Required)?.is_some();
    let content = argue!(arg may repeat Schema)
        .parse_iter(parse_request_schema)
        .ok()?;

    Ok(super::RequestBody {
        content: content.into_iter().collect(),
        description,
        required,
    })
}

fn parse_request_schema(arg: &SchemaArgument) -> Result<(String, super::SchemaRef), syn::Error> {
    let SchemaArgument(content_type, _, schema) = arg;

    Ok((
        content_type.value(),
        super::SchemaRef {
            schema: super::PropertyType::Ref(schema.to_string()),
        },
    ))
}

fn parse_security(
    arg: &ArgumentList<SecurityArgument>,
) -> Result<BTreeMap<String, Vec<String>>, syn::Error> {
    use crate::endpoint::SecurityArgument::*;

    let name = argue!(arg must have Name)?.1.value();
    let scope = argue!(arg may repeat Scope).parse_iter(lit_value).ok()?;

    let mut security = BTreeMap::new();

    security.insert(name, scope);

    Ok(security)
}
