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
    let config_path = cli.config_file.as_path();
    if !config_path.exists() {
        bail!("Unable to find file {:?}", config_path);
    }

    debug!("Loading config file {}", config_path.display());

    let config_file = std::fs::read_to_string(config_path)?;
    let user_config: user::UserConfig = toml::from_str(&config_file)?;

    debug!("Loaded config {:?}", user_config);

    let mut http_backends: Vec<crate::plugin::HttpBackend> = Vec::new();
    for plugin_config in user_config.http.plugin {
        let converted = crate::plugin::create_http_backend(
            config_path.parent().expect("to have a parent"),
            plugin_config,
        )?;
        http_backends.push(converted);
    }

    Ok(ServerConfig {
        http_address: user_config.http.address,
        http_backends,
    })
}
