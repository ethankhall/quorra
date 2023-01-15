// mod wasm;

use crate::errors::*;
use async_trait::async_trait;
use bytes::Bytes;
use figment::Figment;
use http::{HeaderMap, Method, Response};
use rand::{seq::SliceRandom, thread_rng};
use std::fmt::Debug;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tracing::instrument;

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
            let body = self.body.clone().unwrap_or_default();
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
) -> Result<(HttpBackend, Figment), anyhow::Error> {
    use crate::config::PluginSource;
    let (backend, figment) = match (value.source, value.config) {
        (PluginSource::LuaPlugin(_lua), _) => todo!(),
        (PluginSource::WasmPlugin(_wasm), _) => todo!(),
        (PluginSource::Static(_config), Some(config_plugin_path)) => {
            let config_plugin_path = if !config_plugin_path.is_absolute() {
                config_dir.join(config_plugin_path)
            } else {
                config_plugin_path
            };

            let config = crate::config::http::http_static::parse(config_plugin_path)?;
            (
                Box::new(http_static::HttpStaticPlugin::try_from(&config)?),
                config,
            )
        }
        (PluginSource::Static(_), None) => {
            return Err(ConfigError::PluginMissingConfigFile("static".to_string()).into())
        }
    };

    Ok((HttpBackend::new(backend), figment))
}

#[derive(Debug, Clone)]
pub struct StaticResponseContainer {
    pub pointer: Arc<AtomicUsize>,
    pub responses: Vec<Arc<StaticResponse>>,
}

impl StaticResponseContainer {
    pub fn get_response(&self) -> &StaticResponse {
        let response_lenght = self.responses.len();
        let value = match self
            .pointer
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                Some((value + 1) % response_lenght)
            }) {
            Ok(value) => value,
            Err(value) => value,
        };

        self.responses.get(value).unwrap()
    }
}

impl TryFrom<Vec<crate::config::http::StaticResponseConfig>> for StaticResponseContainer {
    type Error = HttpStaticError;
    fn try_from(
        configs: Vec<crate::config::http::StaticResponseConfig>,
    ) -> Result<Self, Self::Error> {
        let mut configs = configs;
        configs.sort_by(|a, b| a.weight.cmp(&b.weight));

        let mut responses = Vec::new();

        for config in &configs {
            let static_response = config.try_into()?;
            let static_response: Arc<StaticResponse> = Arc::new(static_response);
            for _i in 0..config.weight {
                responses.push(static_response.clone());
            }
        }

        responses.shuffle(&mut thread_rng());

        if responses.is_empty() {
            return Err(HttpStaticError::NoResponsesProvided);
        }

        Ok(Self {
            pointer: Arc::new(AtomicUsize::new(0)),
            responses,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StaticResponse {
    status: http::StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl TryFrom<&crate::config::http::StaticResponseConfig> for StaticResponse {
    type Error = HttpStaticError;

    fn try_from(value: &crate::config::http::StaticResponseConfig) -> Result<Self, Self::Error> {
        use crate::config::http::*;

        use bytes::{BufMut, BytesMut};
        use http::{header::HeaderName, header::CONTENT_TYPE, HeaderValue, StatusCode};

        use std::io::Write;

        let status_code = StatusCode::from_u16(value.status)?;

        let mut headers = HeaderMap::new();

        let response_body = {
            let mut writer = BytesMut::new().writer();
            match &value.body {
                None => {}
                Some(StaticResponseBodyConfig::Json { json }) => {
                    headers.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    let body = serde_json::to_vec(&json)?;
                    writer.write_all(&body)?;
                }
                Some(StaticResponseBodyConfig::Raw { bytes }) => {
                    writer.write_all(bytes.as_bytes())?;
                }
            }
            writer.into_inner().freeze()
        };

        {
            for (name, value) in &value.headers {
                headers.insert(
                    HeaderName::from_bytes(name.as_bytes())?,
                    HeaderValue::from_bytes(value.as_bytes())?,
                );
            }

            let plugin_id = match HeaderValue::from_str(&value.id) {
                Ok(value) => value,
                Err(_) => HeaderValue::from_static("response id invalid header"),
            };
            headers.insert(HeaderName::from_static("x-dev-null-response-id"), plugin_id);
        }

        Ok(StaticResponse {
            status: status_code,
            headers,
            body: response_body,
        })
    }
}

impl StaticResponse {
    #[instrument(skip_all, fields(plugin.id = plugin_id, payload.id = payload_id))]
    fn make_response(&self, plugin_id: &str, payload_id: &str) -> Response<Bytes> {
        use http::header::{HeaderName, HeaderValue};
        let mut builder = Response::builder().status(self.status);

        {
            if let Some(headers) = builder.headers_mut() {
                headers.clone_from(&self.headers);
                let plugin_id = match HeaderValue::from_bytes(plugin_id.as_bytes()) {
                    Ok(value) => value,
                    Err(_) => HeaderValue::from_static("plugin id invalid header"),
                };
                headers.insert(HeaderName::from_static("x-dev-null-plugin-id"), plugin_id);

                let payload_id = match HeaderValue::from_bytes(payload_id.as_bytes()) {
                    Ok(value) => value,
                    Err(_) => HeaderValue::from_static("payload id invalid header"),
                };
                headers.insert(HeaderName::from_static("x-dev-null-payload-id"), payload_id);
            }
        }

        builder.body(self.body.clone()).unwrap()
    }
}
