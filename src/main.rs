use clap::Parser;
use std::process::ExitCode;
use tracing::{debug, error, info};

mod config;
mod errors;
mod plugin;

use config::Opts;
use http::{
    header::{HeaderName, AUTHORIZATION},
    HeaderValue, Request,
};
use hyper::{server::Server, service::service_fn};
use plugin::HyperService;
use std::{iter::once, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower::{make::Shared, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

pub fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::main]
async fn main() -> ExitCode {
    human_panic::setup_panic!();
    dotenv::dotenv().ok();

    let opt = Opts::parse();
    config::configure_logging(&opt.logging_opts, &opt.runtime_metrics);

    let result = run_app(opt).await;

    let exit_code = match result {
        Err(e) => {
            error!("Exiting because of {}", e);
            1
        }
        Ok(_) => 0,
    };

    ExitCode::from(exit_code)
}

async fn run_app(cli: Opts) -> Result<(), anyhow::Error> {
    let server_config = config::load_config(&cli).await?;
    let addr: SocketAddr = server_config.http_address.parse()?;

    let http_backends = server_config.http_backends;
    debug!("Found {} http backends", http_backends.len());
    let http_backends = Arc::new(RwLock::new(http_backends));
    let hyper_backend = Arc::new(HyperService::new(http_backends.clone()));

    let real_service = service_fn(move |req: Request<hyper::Body>| {
        let hyper_backend = hyper_backend.clone();
        async move { hyper_backend.process_plugins(req).await }
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
