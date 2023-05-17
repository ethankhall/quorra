use clap::Parser;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{debug, info};

use dev_null_plugin::HyperService;
use http::{
    header::{HeaderName, AUTHORIZATION},
    HeaderValue, Request,
};
use hyper::{server::Server, service::service_fn};
use std::{iter::once, net::SocketAddr, sync::Arc};
use tower::{make::Shared, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[derive(Parser, Debug)]
pub struct ServerCommandConfig {
    #[clap(long, short, env = "CONFIG_PATH")]
    /// Location of root config file
    pub config_file: PathBuf,
}

impl ServerCommandConfig {
    pub async fn run_server(&self) -> Result<(), anyhow::Error> {
        let config_file = self.config_file.as_path();
        let server_config = crate::config::load_config(config_file).await?;
        let addr: SocketAddr = server_config.http_address.parse()?;

        let http_plugins = server_config.http_plugins;
        debug!("Found {} http plugins", http_plugins.len());
        let hyper_backend = Arc::new(RwLock::new(HyperService::new(http_plugins)));

        crate::watch_for_changes(&self.config_file, hyper_backend.clone());

        let real_service = service_fn(move |req: Request<hyper::Body>| {
            let hyper_backend = hyper_backend.clone();
            async move { hyper_backend.read().await.process_plugins(req).await }
        });

        let service = ServiceBuilder::new()
            .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
            // High level logging of requests and responses
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-dev-null"),
                HeaderValue::from_static("true"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-dev-null-plugin-id"),
                HeaderValue::from_static("null"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-dev-null-payload-id"),
                HeaderValue::from_static("null"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-dev-null-response-id"),
                HeaderValue::from_static("null"),
            ))
            // Propagate `X-Request-Id`s from requests to responses
            .layer(PropagateHeaderLayer::new(HeaderName::from_static(
                "x-request-id",
            )))
            .service(real_service);

        info!("Starting server on {}", addr);

        Server::bind(&addr)
            .serve(Shared::new(service))
            .await
            .expect("server error");

        Ok(())
    }
}
