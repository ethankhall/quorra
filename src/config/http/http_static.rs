use super::StaticResponseConfig;
use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::debug;

/// Root of the static plugin configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticPluginConfig {
    /// Unique ID that is included with every response when the plugin matches. When not provided, a random one will be generated.
    #[serde(default = "crate::unique_id")]
    pub id: String,

    /// The http configuration for the static plugin.
    pub http: Vec<StaticHttpConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticHttpConfig {
    /// Unique ID that is included with every response when the plugin matches. When not provided, a random one will be generated.
    #[serde(default = "crate::unique_id")]
    pub id: String,
    pub matches: Vec<StaticMatchesConfig>,
    pub responses: Vec<StaticResponseConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticMatchesConfig {
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    #[serde(default)]
    pub methods: Vec<String>,

    #[serde(default)]
    pub graphql: Option<GraphqlStaticMatchConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct GraphqlStaticMatchConfig {
    #[serde(default)]
    pub operation_name: String,
}

pub fn parse(path: PathBuf) -> Result<StaticPluginConfig, HttpStaticError> {
    let contents = std::fs::read_to_string(path)?;
    let config: StaticPluginConfig = serde_yaml::from_str(&contents)?;

    debug!("Config was parsed as {:?}", config);

    Ok(config)
}
