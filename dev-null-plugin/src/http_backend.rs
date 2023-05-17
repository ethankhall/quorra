use crate::HttpPlugin;
use bytes::Bytes;
use http::{HeaderMap, Method, Response, Uri};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, instrument};

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
    pub fn new(plugin: &Arc<Box<dyn HttpPlugin>>) -> Self {
        Self {
            http_plugin: plugin.clone(),
        }
    }

    #[instrument(skip_all, fields(http.uri = %uri, http.method = %method))]
    pub async fn handle_request(
        &self,
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        body: &Bytes,
    ) -> Result<Option<Response<Bytes>>, HttpBackend> {
        let body = Some(body).filter(|x| !x.is_empty());

        if let Some(response) = self
            .http_plugin
            .respond_to_request(method, uri, headers, &body)
            .await
        {
            return Ok(Some(response));
        }
        Ok(None)
    }
}

#[tokio::test]
#[allow(clippy::box_default)]
async fn test_no_match_found() {
    let backend = HttpBackend {
        http_plugin: Arc::new(Box::new(crate::test_models::EmptyReponse {})),
    };

    let response = backend
        .handle_request(&Method::GET, &"/".parse::<Uri>().unwrap(), &Default::default(), &Bytes::new())
        .await;
    assert!(response.is_ok());
    assert!(response.unwrap().is_none());
}

#[tokio::test]
#[allow(clippy::box_default)]
async fn test_match_found() {
    let backend = HttpBackend {
        http_plugin: Arc::new(Box::new(crate::test_models::ConstantResponse::default())),
    };

    let response = backend
        .handle_request(&Method::GET, &"/".parse::<Uri>().unwrap(), &Default::default(), &Bytes::new())
        .await;
    assert!(response.is_ok());
    assert!(response.unwrap().is_some());
}
