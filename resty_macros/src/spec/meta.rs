use std::collections::BTreeMap;

use proc_macro_argue::{ArgumentList, ParseArgument, argue};

use crate::{
    ResultIterator,
    spec::{SPEC, Spec, definition, lit_value},
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

#[rustfmt::skip]
pub fn apply_meta(
    (_, args): (&syn::Ident, &ArgumentList<syn::MetaList>),
) -> Result<(), syn::Error> {
    use MetaArgument::*;
    let args = &**args;
    let args: ArgumentList<MetaArgument> = syn::parse2(quote::quote! {#args})?;

    let mut spec = SPEC.get();
    let info = &mut spec.info;

    info.title              = argue!(args may have Title)?          .parse(lit_value)?;
    info.description        = argue!(args may have Description)?    .parse(lit_value)?;
    info.terms_of_service   = argue!(args may have TermsOfService)? .parse(lit_value)?;
    info.contact            = argue!(args may have Contact)?        .parse(contact)?;
    info.license            = argue!(args may have License)?        .parse(license)?;
    info.version            = argue!(args may have Version)?        .parse(lit_value)?;
    spec.external_docs      = argue!(args may have ExternalDocs)?   .parse(external_docs)?;
    spec.tags               = argue!(args may repeat Tag)           .parse_iter(tag).ok()?;
    spec.servers            = argue!(args may repeat Server)        .parse_iter(server).ok()?;

    spec.components.security_schemes = argue!(args may have SecuritySchemes)?
        .parse(security_schemes)?
        .unwrap_or_default();

    Ok(())
}

fn security_schemes(
    arg: &ArgumentList<SecuritySchemeArgument>,
) -> Result<BTreeMap<String, definition::SecurityScheme>, syn::Error> {
    use SecuritySchemeArgument::*;
    let api_key = argue!(arg may repeat ApiKey)
        .parse_iter(api_key)
        .map(|r| r.map(|(s, v)| (s, definition::SecurityScheme::ApiKey(v))))
        .ok()?;

    let oauth2 = argue!(arg may repeat OAuth2)
        .parse_iter(oauth2)
        .map(|r| r.map(|(s, v)| (s, definition::SecurityScheme::OAuth2(v))))
        .ok()?;

    Ok(BTreeMap::from_iter(Vec::from_iter(
        api_key.into_iter().chain(oauth2),
    )))
}

fn oauth2(
    arg: &ArgumentList<OAuth2SchemeArgument>,
) -> Result<(String, definition::OAuth2Scheme), syn::Error> {
    use OAuth2SchemeArgument::*;

    let name = argue!(arg must have Name).map(|(.., lit)| lit.value())?;
    let flows = argue!(arg must have Flows)
        .map(|(.., v)| v)
        .and_then(oauth2_flow)?;

    Ok((name, definition::OAuth2Scheme { flows }))
}

fn oauth2_flow(
    arg: &ArgumentList<FlowArgument>,
) -> Result<definition::OAuth2SchemeFlows, syn::Error> {
    use FlowArgument::*;

    let implicit = argue!(arg may have Implicit)?.parse(implicit_flow)?;

    Ok(definition::OAuth2SchemeFlows { implicit })
}

fn implicit_flow(
    arg: &ArgumentList<ImplicitFlowArgument>,
) -> Result<definition::ImplicitOAuth2Flow, syn::Error> {
    use ImplicitFlowArgument::*;

    let authorization_url = argue!(arg must have AuthorizationUrl).map(|(.., v)| v.value())?;
    let scopes = argue!(arg may repeat Scope)
        .map(|(.., ImplicitScope(scope, _, desc))| (scope.value(), desc.value()))
        .collect::<BTreeMap<_, _>>();

    Ok(definition::ImplicitOAuth2Flow {
        authorization_url,
        scopes,
    })
}

fn api_key(
    arg: &ArgumentList<ApiKeySchemeArgument>,
) -> Result<(String, definition::ApiKeyScheme), syn::Error> {
    use ApiKeySchemeArgument::*;

    let name = argue!(arg must have Name).map(|(.., lit)| lit.value())?;
    let is_in = argue!(arg must have In).map(|(.., lit)| lit.value())?;

    Ok((name.clone(), definition::ApiKeyScheme { name, is_in }))
}

fn server(arg: &ArgumentList<ServerArgument>) -> Result<definition::Server, syn::Error> {
    use ServerArgument::*;
    let url = argue!(arg must have Url).map(|(.., v)| v.value())?;
    let description = argue!(arg may have Description)?.map(|(.., v)| v.value());

    Ok(definition::Server {
        url,
        description,
        variables: Default::default(),
    })
}

fn license(arg: &ArgumentList<LicenseArgument>) -> Result<definition::License, syn::Error> {
    use LicenseArgument::*;
    let name = argue!(arg must have Name).map(|(.., v)| v.value())?;
    let url = argue!(arg may have Url)?.map(|(.., v)| v.value());

    Ok(definition::License { name, url })
}

fn contact(arg: &ArgumentList<ContactArgument>) -> Result<definition::Contact, syn::Error> {
    use ContactArgument::*;
    let email = argue!(arg may have Email)?.map(|(.., v)| v.value());

    Ok(definition::Contact {
        email: email,
        name: None,
        url: None,
    })
}

fn external_docs(
    arg: &ArgumentList<ExternalDocsArgument>,
) -> Result<definition::ExternalDocs, syn::Error> {
    use ExternalDocsArgument::*;
    let description = argue!(arg may have Description)?.parse(lit_value)?;
    let url = argue!(arg must have Url).map(|(.., v)| v.value())?;

    Ok(definition::ExternalDocs {
        description,
        url: url,
    })
}

fn tag(arg: &ArgumentList<TagArgument>) -> Result<definition::Tag, syn::Error> {
    use TagArgument::*;
    let name = argue!(arg must have Name).map(|(.., v)| v.value())?;
    let description = argue!(arg may have Description)?.parse(lit_value)?;
    let external_docs = argue!(arg may have ExternalDocs)?.parse(external_docs)?;

    Ok(definition::Tag {
        name: name,
        description,
        external_docs,
    })
}
