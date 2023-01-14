use http::StatusCode;
use hyper::{Body, Request, Response};
use std::sync::Arc;
use tracing::{debug, info, instrument};

use super::HttpBackend;

#[derive(Clone)]
pub struct HyperService {
    plugins: Arc<Vec<HttpBackend>>,
}

impl HyperService {
    pub fn new(backends: Vec<HttpBackend>) -> Self {
        Self {
            plugins: Arc::new(backends),
        }
    }

    #[instrument(skip_all, fields(http.uri = %req.uri(), http.method = %req.method()))]
    pub async fn process_plugins(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, anyhow::Error> {
        let headers = req.headers().clone();
        let uri = req.uri().to_string();
        let method = req.method().clone();
        let body = match hyper::body::to_bytes(req.into_body()).await {
            Ok(body) => body,
            Err(e) => {
                info!("Unable to extract body from HTTP request. Error: {}", e);

                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("x-dev-null-source", "internal")
                    .body(Body::empty())?);
            }
        };

        debug!("Incoming request");

        for plugin in self.plugins.iter() {
            match plugin.handle_request(&method, &uri, &headers, &body).await {
                Ok(None) => continue,
                Ok(Some(response)) => {
                    return Ok(response.map(|x| Body::from(x)));
                }
                Err(e) => {
                    info!(
                        "Plugin raised an error, swallowing response and continueing. Error: {:?}",
                        e
                    );
                    continue;
                }
            }
        }

        debug!("No plugin was configured to process request");

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?)
    }
}
