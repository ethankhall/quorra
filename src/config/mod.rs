use anyhow::bail;
use tracing::debug;

mod cli;
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

pub async fn load_config(cli: &Opts) -> Result<ServerConfig, anyhow::Error> {
    let config_file = cli.config_file.as_path();
    if !config_file.exists() {
        bail!("Unable to find file {:?}", config_file);
    }

    debug!("Loading config file {}", config_file.display());

    let config_file = std::fs::read_to_string(config_file)?;
    let user_config: user::UserConfig = toml::from_str(&config_file)?;

    debug!("Loaded config {:?}", user_config);

    let mut http_backends: Vec<crate::plugin::HttpBackend> = Vec::new();
    for plugin_config in user_config.http.plugin {
        let converted = plugin_config.try_into()?;
        http_backends.push(converted);
    }

    Ok(ServerConfig {
        http_address: user_config.http.address,
        http_backends,
    })
}
