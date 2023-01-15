use crate::errors::*;
use bytes::{BufMut, BytesMut};
use http::{
    header::{HeaderName, CONTENT_TYPE},
    HeaderMap, HeaderValue, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::value::Value as JsonValue;
use std::collections::BTreeMap;
use std::{io::Write, path::PathBuf};
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
    pub response: StaticHttpResponseConfig,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticHttpResponseConfig {
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<StaticHttpResponseBodyConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StaticHttpResponseBodyConfig {
    #[serde(rename = "raw")]
    Raw { bytes: String },
    #[serde(rename = "json")]
    Json { json: JsonValue },
}

pub fn parse(path: PathBuf) -> Result<StaticPluginConfig, HttpStaticError> {
    let contents = std::fs::read_to_string(path)?;
    let config: StaticPluginConfig = serde_yaml::from_str(&contents)?;

    debug!("Config was parsed as {:?}", config);

    Ok(config)
}
