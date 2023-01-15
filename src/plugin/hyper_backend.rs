use http::StatusCode;
use hyper::{Body, Request, Response};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use super::HttpBackend;

#[derive(Clone)]
pub struct HyperService {
    plugins: Arc<RwLock<Vec<HttpBackend>>>,
}

impl HyperService {
    pub fn new(backends: Arc<RwLock<Vec<HttpBackend>>>) -> Self {
        Self { plugins: backends }
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

        for plugin in self.plugins.read().await.iter() {
            match plugin.handle_request(&method, &uri, &headers, &body).await {
                Ok(None) => continue,
                Ok(Some(response)) => {
                    return Ok(response.map(Body::from));
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

#[tokio::test]
#[allow(clippy::box_default)]
async fn test_will_match_first() {
    let empty_backend = HttpBackend::new(Box::new(crate::plugin::test::EmptyReponse::default()));
    let respond_backend =
        HttpBackend::new(Box::new(crate::plugin::test::ConstantResponse::default()));

    let service = HyperService::new(Arc::new(RwLock::new(vec![empty_backend, respond_backend])));
    let req = Request::builder().body(Body::empty()).unwrap();

    let response = service.process_plugins(req).await;
    assert!(response.is_ok());
    let body = hyper::body::to_bytes(response.unwrap().into_body())
        .await
        .unwrap();
    assert_eq!(&String::from_utf8(body.to_vec()).unwrap(), "const ``");
}
