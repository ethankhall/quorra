use anyhow::bail;
use std::path::PathBuf;
use tracing::debug;

mod cli;
pub mod http;
mod logging;
mod user;

pub use cli::Opts;
pub use logging::*;
pub use user::*;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http_address: String,
    pub http_backends: Vec<crate::plugin::HttpBackend>,
}

pub async fn load_config(cli: &Opts) -> Result<(ServerConfig, Vec<PathBuf>), anyhow::Error> {
    let config_path = cli.config_file.as_path();
    if !config_path.exists() {
        bail!("Unable to find file {:?}", config_path);
    }

    let mut paths_to_watch = Vec::new();
    paths_to_watch.push(config_path.to_path_buf());

    debug!("Loading config file {}", config_path.display());

    let config_file = std::fs::read_to_string(config_path)?;
    let user_config: user::UserConfig = toml::from_str(&config_file)?;

    debug!("Loaded config {:?}", user_config);

    let mut http_backends: Vec<crate::plugin::HttpBackend> = Vec::new();
    for plugin_config in user_config.http.plugin {
        let (converted, plugin_config_path) = crate::plugin::create_http_backend(
            config_path.parent().expect("to have a parent"),
            plugin_config,
        )?;
        http_backends.push(converted);
        paths_to_watch.push(plugin_config_path.clone());
    }

    Ok((
        ServerConfig {
            http_address: user_config.http.address,
            http_backends,
        },
        paths_to_watch,
    ))
}
