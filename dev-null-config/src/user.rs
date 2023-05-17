use serde::{Deserialize, Serialize};

/// The root confguration for `/dev/null` providing configuration options
/// for the different server backends.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ServerRootConfig {
    pub responses: ResponsesConfig,
}

/// Container of response configs
#[derive(Serialize, Deserialize, Debug)]
pub struct ResponsesConfig {
    /// Glob of paths to load responses from
    pub paths: Vec<String>,
}
