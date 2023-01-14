use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct UserConfig {
    pub http: HttpConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfig {
    pub address: String,
    pub plugin: Vec<PluginConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginConfig {
    pub source: PluginSource,
    pub config: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PluginSource {
    #[serde(rename = "wasm")]
    WasmPlugin(WasmPluginDef),
    #[serde(rename = "lua")]
    LuaPlugin(LuaPluginDef),
    #[serde(rename = "static")]
    Static(StaticPluginDef),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WasmPluginDef {
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuaPluginDef {
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticPluginDef {}
