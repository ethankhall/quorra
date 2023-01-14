// mod wasm;

use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, Method, Response};
use std::fmt::Debug;
use thiserror::Error;

mod http_backend;
mod http_static;
mod hyper_backend;

pub use http_backend::HttpBackend;
pub use hyper_backend::HyperService;

#[async_trait]
pub trait HttpPlugin: Debug + Sync + Send {
    async fn respond_to_request(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>>;
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The plugin {0} is requires a config file.")]
    PluginMissingConfigFile(String),
}

impl TryFrom<crate::config::PluginConfig> for HttpBackend {
    type Error = anyhow::Error;

    fn try_from(value: crate::config::PluginConfig) -> Result<Self, Self::Error> {
        use crate::config::PluginSource;
        let backend = match (value.source, value.config) {
            (PluginSource::LuaPlugin(_lua), _) => todo!(),
            (PluginSource::WasmPlugin(_wasm), _) => todo!(),
            (PluginSource::Static(_config), Some(path)) => {
                Box::new(http_static::HttpStaticPlugin::try_from(path)?)
            }
            (PluginSource::Static(_), None) => {
                return Err(ConfigError::PluginMissingConfigFile("static".to_string()).into())
            }
        };

        Ok(HttpBackend::new(backend))
    }
}
