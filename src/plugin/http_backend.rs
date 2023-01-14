use crate::plugin::HttpPlugin;
use bytes::Bytes;
use http::{HeaderMap, Method, Response, Uri};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, instrument};

#[derive(Debug, Clone)]
pub struct HttpBackend {
    http_plugin: Arc<Box<dyn HttpPlugin>>,
}

#[derive(Error, Debug)]
pub enum HttpBackendErr {
    #[error(transparent)]
    UnableToReadBody(#[from] http::Error),
}

impl HttpBackend {
    pub fn new(plugin: Box<dyn HttpPlugin>) -> Self {
        Self {
            http_plugin: Arc::new(plugin),
        }
    }

    #[instrument(skip_all, fields(http.uri = %uri, http.method = %method))]
    pub async fn handle_request(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        body: &Bytes,
    ) -> Result<Option<Response<Bytes>>, HttpBackend> {
        let body = Some(body).filter(|x| !x.is_empty());

        if let Some(response) = self
            .http_plugin
            .respond_to_request(&method, &uri, &headers, &body)
            .await
        {
            return Ok(Some(response));
        }
        Ok(None)
    }
}
