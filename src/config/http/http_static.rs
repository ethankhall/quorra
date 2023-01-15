use super::StaticResponseConfig;
use crate::errors::*;
use figment::{
    providers::{Format, YamlExtended},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::debug;

/// Root of the static plugin configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticPluginConfig {
    /// Unique ID that is included with every response when the plugin matches.
    /// When not provided, a random one will be generated.
    #[serde(default = "crate::unique_id")]
    pub id: String,

    /// The http configuration for the static plugin.
    pub http: Vec<StaticHttpConfig>,
}

/// A static response configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticHttpConfig {
    /// Unique ID that is included with every response when the plugin matches.
    /// When not provided, a random one will be generated.
    #[serde(default = "crate::unique_id")]
    pub id: String,

    /// A list of ways that the request can be matched against.
    pub matches: Vec<StaticMatchesConfig>,

    /// A list of possible responses.
    pub responses: Vec<StaticResponseConfig>,
}

/// The possible options to match against. All fields are optional. When all
/// fields are missing, the request will match.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticMatchesConfig {
    /// A regex to be used to match against the request path.
    #[serde(default)]
    pub path: Option<String>,

    /// A map of key-value pairs. The key is the header name, the
    /// value used to match the header against.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// A list of methods the request should be.
    #[serde(default)]
    pub methods: Vec<String>,

    /// Configuration for GraphQL body matchers
    #[serde(default)]
    pub graphql: Option<GraphqlStaticMatchConfig>,
}

/// GraphQL body matcher
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphqlStaticMatchConfig {
    /// The name of the GraphQL operation to respond to
    #[serde(default)]
    pub operation_name: String,
}

pub fn parse(path: PathBuf) -> Result<Figment, HttpStaticError> {
    debug!("Loading static plugin file {}", path.display());
    let figment = Figment::new().join(YamlExtended::file(&path));
    debug!("Config was parsed as {:?}", figment);

    Ok(figment)
}
