use thiserror::Error;

mod config;
mod http_static;

use dev_null_config::prelude::StaticHttpConfig;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use std::sync::RwLock;
use tracing::debug;

lazy_static! {
    pub(crate) static ref HANDLEBARS: RwLock<Box<Handlebars<'static>>> = {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        RwLock::new(Box::new(handlebars))
    };
}

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
    TemplateError(#[from] handlebars::TemplateError),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

#[derive(Debug, Default)]
pub struct HttpStaticPluginBuilder {
    configs: Vec<StaticHttpConfig<String>>,
}

impl HttpStaticPluginBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_config(&mut self, config: &StaticHttpConfig<String>) {
        self.configs.push(config.clone());
    }

    pub fn build(self) -> Result<http_static::HttpStaticPlugin, HttpPluginError> {
        debug!("{} responses loaded", self.configs.len());
        let plugin_config = crate::config::internal::PluginBackendConfig::try_from(&self.configs)?;

        Ok(http_static::HttpStaticPlugin {
            config: plugin_config,
        })
    }
}
