use clap::Parser;
use std::{path::Path, process::ExitCode, time::Duration};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

mod config;
mod server;

use config::Opts;
use dev_null_plugin::HyperService;
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
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            debug!("Reloading configuration");

            match config::load_config(&root_config).await {
                Ok(server_config) => {
                    let service = HyperService::new(server_config.http_plugins);
                    *shared_config.write().await = service;
                }
                Err(e) => {
                    warn!("Unable to reload config because {:?}", e);
                }
            }
        }
    });
}
