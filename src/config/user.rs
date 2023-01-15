use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The root confguration for `/dev/null` providing configuration options
/// for the different server backends.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct UserConfig {
    pub http: HttpConfig,
}

/// Http server configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfig {
    /// The Address to listen on when recieving http requests. This should
    /// contain the IP address as well as the port. An example is
    /// `0.0.0.0:8080`, which will listen on all addresses on port 8080.
    pub address: String,

    /// List of plugins for the HTTP server
    pub plugin: Vec<PluginConfig>,
}

/// A plugin, which will define what the plugin is, and where additional configuration
/// may be pulled from.
#[derive(Serialize, Deserialize, Debug)]
pub struct PluginConfig {
    /// Define what kind of plugin is being defined.
    pub source: PluginSource,

    /// Where extra configuration for the plugin will be pulled from.
    pub config: Option<PathBuf>,
}

/// List of supported plugins.
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
    /// Path to the WASM file on disk
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LuaPluginDef {
    /// Path to the lua file on disk.
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticPluginDef {}
