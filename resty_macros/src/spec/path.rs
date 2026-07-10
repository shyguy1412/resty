use std::{collections::BTreeMap, convert::identity};

use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};

use crate::{
    Reparse, ResultIterator,
    endpoint::{HandlerArgument, parse_route},
    routing,
    spec::{RequestBody, SPEC, Spec, lit_value},
};

argue! {
    MetaArgument {
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
    use crate::endpoint::HandlerArgument::*;
    use MetaArgument::*;
    let args: ArgumentList<HandlerArgument> = syn::parse(args)?;

    let route = argue!(args may have Route)?
        .parse(parse_route)?
        .map_or_else(routing::default_route, Ok)?
        .join("/");

    let methods = argue!(args may repeat Method)
        .map(|(.., m)| m.to_string().to_ascii_lowercase())
        .collect::<Vec<_>>();

    let Some(meta) = argue!(args may have Meta)?
        .map(|m| m.1.reparse::<ArgumentList<MetaArgument>>())
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?
    else {
        return Ok(());
    };

    let tags = argue!(meta may repeat Tag).parse_iter(lit_value).ok()?;
    let summary = argue!(meta may have Summary)?.parse(lit_value)?;
    let description = argue!(meta may have Description)?.parse(lit_value)?;
    let responses = argue!(meta may repeat Response)
        .parse_iter(parse_response)
        .ok()?;

    let security = argue!(meta may repeat Security)
        .parse_iter(parse_security)
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();
    let request_body = argue!(meta may have Request)?.parse(parse_request_body)?;

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

fn parse_response(arg: &ResponseArgument) -> Result<super::ResponseType, syn::Error> {
    let ResponseArgument(ty, _, desc) = arg;

    let r = match ty {
        ResponseType::Path(path) => super::ResponseType::Ref(
            path.segments
                .last()
                .ok_or(syn::Error::new_spanned(path, "Can not parse path"))?
                .ident
                .to_string(),
            desc.value(),
        ),
        ResponseType::Code(lit_int) => {
            super::ResponseType::Raw(lit_int.base10_digits().to_string(), desc.value())
        }
    };

    Ok(r)
}

fn parse_request_body(arg: &ArgumentList<RequestArgument>) -> Result<RequestBody, syn::Error> {
    use RequestArgument::*;

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
    use SecurityArgument::*;

    let name = argue!(arg must have Name)?.1.value();
    let scope = argue!(arg may repeat Scope).parse_iter(lit_value).ok()?;

    let mut security = BTreeMap::new();

    security.insert(name, scope);

    Ok(security)
}
