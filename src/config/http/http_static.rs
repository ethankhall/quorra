use super::StaticResponseConfig;
use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct StaticPluginConfig {
    #[serde(default = "crate::unique_id")]
    pub id: String,
    pub http: Vec<StaticHttpConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticHttpConfig {
    #[serde(default = "crate::unique_id")]
    pub id: String,
    pub matches: Vec<StaticMatchesConfig>,
    pub response: StaticResponseConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticMatchesConfig {
    #[serde(default)]
    pub path: Option<String>,

    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    #[serde(default)]
    pub methods: Vec<String>,
}

pub fn parse(path: PathBuf) -> Result<StaticPluginConfig, HttpStaticError> {
    let contents = std::fs::read_to_string(path)?;
    let config: StaticPluginConfig = serde_yaml::from_str(&contents)?;

    debug!("Config was parsed as {:?}", config);

    Ok(config)
}
