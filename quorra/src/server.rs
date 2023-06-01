use clap::Parser;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{debug, info};

use http::{
    header::{HeaderName, AUTHORIZATION},
    HeaderValue, Request,
};
use hyper::{server::Server, service::service_fn};
use quorra_config::ConfigContainer;
use quorra_plugin::HyperService;
use std::{iter::once, net::SocketAddr, sync::Arc};
use tower::{make::Shared, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[derive(Parser, Debug)]
pub struct ServerCommandConfig {
    #[clap(long = "config", short, env = "CONFIG_PATH")]
    /// Location of root config file
    pub config_file: PathBuf,

    #[clap(
        long = "listen",
        short,
        env = "SERVER_LISTEN",
        default_value("127.0.0.1:3000")
    )]
    pub listen_address: String,
}

impl ServerCommandConfig {
    pub async fn run_server(&self) -> Result<(), anyhow::Error> {
        let config_container = ConfigContainer::new(&self.config_file);
        let user_config = config_container.load_config()?;
        let http_plugins = crate::config::build_backends(&user_config).await?;
        debug!("Found {} http plugins", http_plugins.len());

        let addr: SocketAddr = self.listen_address.parse()?;
        let service = HyperService::new(http_plugins);
        let hyper_backend = Arc::new(RwLock::new(service));

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
                HeaderName::from_static("x-quorra"),
                HeaderValue::from_static("true"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-quorra-plugin-id"),
                HeaderValue::from_static("null"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-quorra-payload-id"),
                HeaderValue::from_static("null"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("x-quorra-response-id"),
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
