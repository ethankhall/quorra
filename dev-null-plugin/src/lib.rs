use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, Method, Response};
pub use hyper_backend::HyperService;
use std::{fmt::Debug, sync::Arc};

mod http_backend;
mod hyper_backend;
#[cfg(test)]
mod test_models;

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

pub async fn make_hyper_backend(plugins: Vec<Arc<Box<dyn HttpPlugin>>>) -> HyperService {
    HyperService::new(plugins)
}
