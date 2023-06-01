use std::sync::Arc;

use crate::HttpPlugin;
use http::StatusCode;
use hyper::{Body, Request, Response};
use tracing::{debug, info, instrument};

use crate::http_backend::HttpBackend;

#[derive(Clone)]
pub struct HyperService {
    backends: Arc<Vec<HttpBackend>>,
}

impl HyperService {
    pub fn new(plugins: Vec<Arc<Box<dyn HttpPlugin>>>) -> Self {
        let mut backends = Vec::new();
        for plugin in plugins.iter() {
            backends.push(HttpBackend::new(plugin));
        }
        Self {
            backends: Arc::new(backends),
        }
    }

    #[instrument(skip_all, fields(http.uri = %req.uri(), http.method = %req.method()))]
    pub async fn process_plugins(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, anyhow::Error> {
        let headers = req.headers().clone();
        let uri = req.uri().clone();
        let method = req.method().clone();
        let body = match hyper::body::to_bytes(req.into_body()).await {
            Ok(body) => body,
            Err(e) => {
                info!("Unable to extract body from HTTP request. Error: {}", e);

                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("x-quorra-source", "internal")
                    .body(Body::empty())?);
            }
        };

        debug!("Incoming request");

        for backend in self.backends.iter() {
            match backend.handle_request(&method, &uri, &headers, &body).await {
                Ok(None) => continue,
                Ok(Some(response)) => {
                    return Ok(response.map(Body::from));
                }
                Err(e) => {
                    info!(
                        "Backend raised an error, swallowing response and continueing. Error: {:?}",
                        e
                    );
                    continue;
                }
            }
        }

        debug!("No backend was configured to process request");

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())?)
    }
}

#[tokio::test]
#[allow(clippy::box_default)]
async fn test_will_match_first() {
    let service = HyperService::new(vec![
        Arc::new(Box::new(crate::test_models::EmptyReponse::default())),
        Arc::new(Box::new(crate::test_models::ConstantResponse::default())),
    ]);
    let req = Request::builder().body(Body::empty()).unwrap();

    let response = service.process_plugins(req).await;
    assert!(response.is_ok());
    let body = hyper::body::to_bytes(response.unwrap().into_body())
        .await
        .unwrap();
    assert_eq!(&String::from_utf8(body.to_vec()).unwrap(), "const ``");
}
