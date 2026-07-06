use std::{collections::HashMap, convert::identity};

use proc_macro_argue::{ArgumentList, argue};
use syn::Item::Impl;

use crate::{
    ResultIterator,
    spec::{SPEC, SecurityScheme, Spec},
};

argue! {
    MetaArgument {
        Title: syn::LitStr,
        Description: syn::LitStr,
        TermsOfService: syn::LitStr,
        Contact: ArgumentList<ContactArgument>,
        License: ArgumentList<LicenseArgument>,
        Version: syn::LitStr,
        Server: ArgumentList<ServerArgument>,
        Tag: ArgumentList<TagArgument>,
        SecuritySchemes: ArgumentList<SecuritySchemeArgument>,
        ExternalDocs: ArgumentList<ExternalDocsArgument>,
    };

    ServerArgument {
        Url: syn::LitStr,
        Description: syn::LitStr,
    };

    ContactArgument {
        Email: syn::LitStr,
    };
    LicenseArgument {
        Name: syn::LitStr,
        Url: syn::LitStr,
    };
    TagArgument {
        Name: syn::LitStr,
        Description: syn::LitStr,
        ExternalDocs: ArgumentList<ExternalDocsArgument>,
    };
    ExternalDocsArgument {
        Description: syn::LitStr,
        Url: syn::LitStr,
    };
    SecuritySchemeArgument {
        ApiKey: ArgumentList<ApiKeySchemeArgument>,
        OAuth2: ArgumentList<OAuth2SchemeArgument>,
    };
    ApiKeySchemeArgument {
        Name: syn::LitStr,
        In: syn::LitStr,
    };
    OAuth2SchemeArgument {
        Name: syn::LitStr,
        Flows: ArgumentList<FlowArgument>,
    };
    FlowArgument {
        Implicit: ArgumentList<ImplicitFlowArgument>,
    };
    ImplicitFlowArgument {
        AuthorizationUrl: syn::LitStr,
        Scope: ImplicitScope,
    };
    ImplicitScope(syn::LitStr, syn::token::Comma, syn::LitStr)
}

pub fn apply_meta(
    (_, args): (&syn::Ident, &ArgumentList<syn::MetaList>),
) -> Result<(), syn::Error> {
    use MetaArgument::*;
    let args = &**args;
    let args: ArgumentList<MetaArgument> = syn::parse2(quote::quote! {#args})?;

    let mut spec = SPEC.get();

    let title = argue!(args may have Title)?.map(|(.., lit)| lit.value());
    spec.info.title = title;

    let desc = argue!(args may have Description)?.map(|(.., lit)| lit.value());
    spec.info.description = desc;

    let tos = argue!(args may have TermsOfService)?.map(|(.., lit)| lit.value());
    spec.info.terms_of_service = tos;

    let version = argue!(args may have Version)?.map(|(.., lit)| lit.value());
    spec.info.version = version;

    let external_docs = argue!(args may have ExternalDocs)?
        .map(|(.., docs)| docs)
        .map(parse_external_docs_arg)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?;
    spec.external_docs = external_docs;

    let servers = argue!(args may repeat Server)
        .map(|(.., server)| server)
        .map(parse_server_arg)
        .ok()?;
    spec.servers = servers;

    let contact = argue!(args may have Contact)?
        .map(|(.., contact)| contact)
        .map(parse_contact_arg)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?;
    spec.info.contact = contact;

    let tags = argue!(args may repeat Tag)
        .map(|(.., tag)| tag)
        .map(parse_tag_arg)
        .ok()?;
    spec.tags = tags;

    let security_schemes = argue!(args may have SecuritySchemes)?
        .map(|(.., scheme)| scheme)
        .map(parse_security_schemes)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?
        .unwrap_or_default();
    spec.components.security_schemes.extend(security_schemes);

    Ok(())
}

fn parse_security_schemes(
    arg: &ArgumentList<SecuritySchemeArgument>,
) -> Result<Vec<(String, super::SecurityScheme)>, syn::Error> {
    use SecuritySchemeArgument::*;
    let api_key = argue!(arg may repeat ApiKey)
        .map(|(.., v)| v)
        .map(parse_api_key_arg)
        .ok()?
        .into_iter()
        .map(|(s, v)| (s, super::SecurityScheme::ApiKey(v)));

    let oauth2 = argue!(arg may repeat OAuth2)
        .map(|(.., v)| v)
        .map(parse_oauth2_key_arg)
        .ok()?
        .into_iter()
        .map(|(s, v)| (s, super::SecurityScheme::OAuth2(v)));

    Ok(Vec::from_iter(api_key.into_iter().chain(oauth2)))
}

fn parse_oauth2_key_arg(
    arg: &ArgumentList<OAuth2SchemeArgument>,
) -> Result<(String, super::OAuth2Scheme), syn::Error> {
    use OAuth2SchemeArgument::*;

    let name = argue!(arg must have Name).map(|(.., lit)| lit.value())?;
    let flows = argue!(arg must have Flows)
        .map(|(.., v)| v)
        .and_then(parse_oauth2_flow_arg)?;

    Ok((name, super::OAuth2Scheme { flows }))
}

fn parse_oauth2_flow_arg(
    arg: &ArgumentList<FlowArgument>,
) -> Result<super::OAuth2SchemeFlows, syn::Error> {
    use FlowArgument::*;

    let implicit = argue!(arg may have Implicit)?
        .map(|(.., v)| v)
        .map(parse_implicit_flow_arg)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?;

    Ok(super::OAuth2SchemeFlows { implicit })
}

fn parse_implicit_flow_arg(
    arg: &ArgumentList<ImplicitFlowArgument>,
) -> Result<super::ImplicitOAuth2Flow, syn::Error> {
    use ImplicitFlowArgument::*;

    let authorization_url = argue!(arg must have AuthorizationUrl).map(|(.., v)| v.value())?;
    let scopes = argue!(arg may repeat Scope)
        .map(|(.., ImplicitScope(scope, _, desc))| (scope.value(), desc.value()))
        .collect::<HashMap<_, _>>();

    Ok(super::ImplicitOAuth2Flow {
        authorization_url,
        scopes,
    })
}

fn parse_api_key_arg(
    arg: &ArgumentList<ApiKeySchemeArgument>,
) -> Result<(String, super::ApiKeyScheme), syn::Error> {
    use ApiKeySchemeArgument::*;

    let name = argue!(arg must have Name).map(|(.., lit)| lit.value())?;
    let is_in = argue!(arg must have In).map(|(.., lit)| lit.value())?;

    let mut segs = name.split("_");

    Ok((name.clone(), super::ApiKeyScheme { name, is_in }))
}

fn parse_server_arg(arg: &ArgumentList<ServerArgument>) -> Result<super::Server, syn::Error> {
    use ServerArgument::*;
    let (.., url) = argue!(arg must have Url)?;
    Ok(super::Server { url: url.value() })
}

fn parse_contact_arg(arg: &ArgumentList<ContactArgument>) -> Result<super::Contact, syn::Error> {
    use ContactArgument::*;
    let (.., email) = argue!(arg must have Email)?;
    Ok::<_, syn::Error>(super::Contact {
        email: email.value(),
    })
}

fn parse_external_docs_arg(
    arg: &ArgumentList<ExternalDocsArgument>,
) -> Result<super::ExternalDocs, syn::Error> {
    use ExternalDocsArgument::*;
    let description = argue!(arg may have Description)?.map(|(.., desc)| desc.value());
    let (.., url) = argue!(arg must have Url)?;
    Ok(super::ExternalDocs {
        description,
        url: url.value(),
    })
}

fn parse_tag_arg(arg: &ArgumentList<TagArgument>) -> Result<super::Tag, syn::Error> {
    use TagArgument::*;
    let (.., name) = argue!(arg must have Name)?;
    let description = argue!(arg may have Description)?.map(|(.., desc)| desc.value());
    let external_docs = argue!(arg may have ExternalDocs)?
        .map(|(.., desc)| desc)
        .map(parse_external_docs_arg)
        .map(|r| r.map(Some))
        .map_or(Ok(None), identity)?;
    Ok(super::Tag {
        name: name.value(),
        description,
        external_docs,
    })
}
