use clap::Parser;
use quorra_config::ConfigContainer;
use std::{path::Path, process::ExitCode, time::Duration};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

mod config;
mod convert_har;
mod server;

use config::Opts;
use quorra_plugin::HyperService;
use std::sync::Arc;

pub fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::main]
async fn main() -> ExitCode {
    human_panic::setup_panic!();
    dotenv::dotenv().ok();

    let opt = Opts::parse();
    config::configure_logging(&opt.logging_opts, &opt.runtime_metrics);

    let result = match opt.command {
        config::SubCommands::Server(server_opts) => server_opts.run_server().await,
        config::SubCommands::ConvertHar(options) => options.run_convert().await,
    };

    let exit_code = match result {
        Err(e) => {
            error!("Exiting because of {}", e);
            1
        }
        Ok(_) => 0,
    };

    ExitCode::from(exit_code)
}

fn watch_for_changes(root_config: &Path, shared_config: Arc<RwLock<HyperService>>) {
    let root_config = root_config.to_path_buf();
    let config_container = ConfigContainer::new(&root_config);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            debug!("Reloading configuration");

            let loaded_config = match config_container.load_config() {
                Ok(loaded_config) => loaded_config,
                Err(e) => {
                    warn!("Unable to load config: {:?}", e);
                    continue;
                }
            };

            let http_plugins = match crate::config::build_backends(&loaded_config).await {
                Ok(loaded_config) => loaded_config,
                Err(e) => {
                    warn!("Unable to convert config: {:?}", e);
                    continue;
                }
            };
            debug!("Found {} http plugins", http_plugins.len());
            let service = HyperService::new(http_plugins);
            {
                *shared_config.write().await = service;
            }
        }
    });
}
