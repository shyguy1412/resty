use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey(ApiKeyScheme),
    #[serde(rename = "oauth2")]
    OAuth2(OAuth2Scheme),
}

#[derive(Serialize, Default)]
pub struct ApiKeyScheme {
    pub name: String,
    #[serde(rename = "in")]
    pub is_in: String,
}
#[derive(Serialize)]
pub struct OAuth2Scheme {
    pub flows: OAuth2SchemeFlows,
}
#[derive(Serialize)]
pub struct OAuth2SchemeFlows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitOAuth2Flow>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]

pub struct ImplicitOAuth2Flow {
    pub authorization_url: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub scopes: BTreeMap<String, String>,
}
