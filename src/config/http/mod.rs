pub mod http_static;

use serde::{Deserialize, Serialize};
use serde_json::value::Value as JsonValue;
use std::collections::BTreeMap;

fn default_weight() -> u16 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaticResponseConfig {
    #[serde(default = "default_weight")]
    pub weight: u16,
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<StaticResponseBodyConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum StaticResponseBodyConfig {
    #[serde(rename = "raw")]
    Raw { bytes: String },
    #[serde(rename = "json")]
    Json { json: JsonValue },
}
