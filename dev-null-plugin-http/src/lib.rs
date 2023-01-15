use std::path::Path;

use thiserror::Error;

mod config;
mod http_static;

pub use crate::config::StaticPluginDef;

#[derive(Error, Debug)]
pub enum HttpPluginError {
    #[error(transparent)]
    HttpError(#[from] http::Error),
    #[error(transparent)]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error("No respone configured for match")]
    NoResponsesProvided,
    #[error(transparent)]
    FigmentError(#[from] figment::Error),
}

pub(crate) fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub async fn build_plugin(
    def: crate::config::StaticPluginDef,
    config_dir: &Path,
) -> Result<http_static::HttpStaticPlugin, HttpPluginError> {
    let plugin_config = config::load_http_plugin_config(def, config_dir).await?;

    Ok(http_static::HttpStaticPlugin {
        config: plugin_config,
    })
}
