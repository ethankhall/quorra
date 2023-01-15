use anyhow::bail;
use figment::{
    providers::{Format, Toml},
    Figment,
};
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

pub async fn load_config(cli: &Opts) -> Result<ServerConfig, anyhow::Error> {
    let config_path = cli.config_file.as_path();
    if !config_path.exists() {
        bail!("Unable to find file {:?}", config_path);
    }

    debug!("Loading config file {}", config_path.display());

    let figment = Figment::new().merge(Toml::file(&config_path));
    let user_config: user::UserConfig = figment.extract()?;

    debug!("Loaded config {:?}", user_config);

    let mut http_backends: Vec<crate::plugin::HttpBackend> = Vec::new();
    for plugin_config in user_config.http.plugin {
        let (backend, _plugin_figment) = crate::plugin::create_http_backend(
            config_path.parent().expect("to have a parent"),
            plugin_config,
        )?;
        http_backends.push(backend);
    }

    Ok(ServerConfig {
        http_address: user_config.http.address,
        http_backends,
    })
}
