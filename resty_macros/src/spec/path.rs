use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro_argue::{ArgumentList, ParseArgument, argue};

use crate::{
    ResultIterator,
    endpoint::*,
    routing,
    spec::{
        RequestBody, SPEC, Spec,
        definition::{
            ComponentType, ContentReference, OrRef, Parameter, ReferenceObject, Response,
        },
        lit_value,
    },
};

pub fn add_path(args: TokenStream) -> Result<(), syn::Error> {
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
    let request_body = argue!(args may have Request)?.parse(request_body)?;

    let parameters = argue!(args may repeat Parameter)
        .parse_iter(parameter)
        .ok()?;

    let responses = argue!(args may repeat Response)
        .parse_iter(response)
        .ok()?
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    let security = argue!(args may repeat Security)
        .parse_iter(security)
        .ok()?
        .into_iter()
        .collect::<Vec<_>>();

    let mut spec = SPEC.get();
    let path = spec
        .paths
        .entry(format!("/{route}"))
        .or_insert_with(|| super::PathItem {
            operations: BTreeMap::new(),
        });

    let operation_object = super::OperationObject {
        tags,
        summary,
        description,
        request_body,
        operation_id: String::new(),
        parameters,
        responses,
        security,
    };

    for method in methods {
        let mut operation_object = operation_object.clone();
        operation_object.operation_id = format!("{method}{}", route.replace("/", "_"));
        path.operations.insert(method, operation_object);
    }

    Ok(())
}

fn parameter(arg: &ArgumentList<ParameterArgument>) -> Result<Parameter, syn::Error> {
    use ParameterArgument::*;

    let name = argue!(arg must have Name).map(|(.., v)| v.value())?;
    let is_in = argue!(arg must have In).map(|(.., v)| v.value())?;
    let description = argue!(arg may have Description)?.parse(lit_value)?;
    let required = argue!(arg may have Required)?.is_some();
    let explode = argue!(arg may have Explode)?.is_some();
    let schema = argue!(arg must have Schema).map(|(.., r)| {
        OrRef::Ref(ReferenceObject {
            component: ComponentType::Schema,
            name: r.to_string(),
        })
    })?;

    Ok(Parameter {
        name,
        is_in,
        description,
        required,
        explode,
        schema,
    })
}

fn response(arg: &ResponseArgument) -> Result<(String, OrRef<Response>), syn::Error> {
    let r = match &arg.2 {
        ResponseType::Ref(ident) => OrRef::Ref(ReferenceObject {
            component: ComponentType::Response,
            name: ident.to_string(),
        }),
        ResponseType::Contentless(lit_str) => OrRef::Val(Response {
            description: lit_str.value(),
            content: Default::default(),
        }),
        ResponseType::Array(ident) => OrRef::Ref(ReferenceObject {
            component: ComponentType::Response,
            name: format!("autogen__{}Array", ident.to_string()),
        }),
    };

    Ok((arg.0.base10_digits().to_string(), r))
}

fn request_body(arg: &ArgumentList<RequestArgument>) -> Result<RequestBody, syn::Error> {
    use crate::endpoint::RequestArgument::*;

    let description = argue!(arg may have Description)?.parse(lit_value)?;
    let required = argue!(arg may have Required)?.is_some();
    let content = argue!(arg may repeat Schema)
        .parse_iter(request_schema)
        .ok()?;

    Ok(super::RequestBody {
        content: content.into_iter().collect(),
        description,
        required,
    })
}

fn request_schema(arg: &SchemaArgument) -> Result<(String, ContentReference), syn::Error> {
    let SchemaArgument(content_type, _, schema) = arg;

    Ok((
        content_type.value(),
        ContentReference {
            schema: ReferenceObject {
                component: ComponentType::Schema,
                name: schema.to_string(),
            },
        },
    ))
}

fn security(
    arg: &ArgumentList<SecurityArgument>,
) -> Result<BTreeMap<String, Vec<String>>, syn::Error> {
    use crate::endpoint::SecurityArgument::*;

    let name = argue!(arg must have Name)?.1.value();
    let scope = argue!(arg may repeat Scope).parse_iter(lit_value).ok()?;

    let mut security = BTreeMap::new();

    security.insert(name, scope);

    Ok(security)
}
