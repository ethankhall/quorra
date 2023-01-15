// mod wasm;

use crate::errors::*;
use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, Method, Response};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod test {

    use bytes::Bytes;
    use http::{header::HeaderMap, Method, Response, StatusCode};

    #[derive(Debug, Default)]
    pub struct EmptyReponse {}

    #[async_trait::async_trait]
    impl crate::plugin::HttpPlugin for EmptyReponse {
        async fn respond_to_request(
            &self,
            _method: &Method,
            _uri: &str,
            _headers: &HeaderMap,
            _body: &Option<&Bytes>,
        ) -> Option<Response<Bytes>> {
            None
        }
    }

    #[derive(Debug, Default)]
    pub struct ConstantResponse {
        body: Option<String>,
    }

    #[async_trait::async_trait]
    impl crate::plugin::HttpPlugin for ConstantResponse {
        async fn respond_to_request(
            &self,
            _method: &Method,
            _uri: &str,
            _headers: &HeaderMap,
            _body: &Option<&Bytes>,
        ) -> Option<Response<Bytes>> {
            let body = self.body.clone().map(|x| x.clone()).unwrap_or_default();
            let body = Bytes::from(format!("const `{}`", body));
            Some(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(body)
                    .unwrap(),
            )
        }
    }
}

pub fn create_http_backend(
    config_dir: &Path,
    value: crate::config::PluginConfig,
) -> Result<(HttpBackend, PathBuf), anyhow::Error> {
    use crate::config::PluginSource;
    let (backend, config_plugin_path) = match (value.source, value.config) {
        (PluginSource::LuaPlugin(_lua), _) => todo!(),
        (PluginSource::WasmPlugin(_wasm), _) => todo!(),
        (PluginSource::Static(_config), Some(config_plugin_path)) => {
            let config_plugin_path = if !config_plugin_path.is_absolute() {
                config_dir.join(config_plugin_path)
            } else {
                config_plugin_path
            };
            (
                Box::new(http_static::HttpStaticPlugin::try_from(
                    config_plugin_path.clone(),
                )?),
                config_plugin_path,
            )
        }
        (PluginSource::Static(_), None) => {
            return Err(ConfigError::PluginMissingConfigFile("static".to_string()).into())
        }
    };

    Ok((HttpBackend::new(backend), config_plugin_path))
}
