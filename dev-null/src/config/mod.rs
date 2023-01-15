use anyhow::bail;
use dev_null_plugin::HttpPlugin;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use std::{path::Path, sync::Arc};
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
    pub http_plugins: Vec<Arc<Box<dyn HttpPlugin>>>,
}

pub async fn load_config(cli: &Opts) -> Result<ServerConfig, anyhow::Error> {
    let config_path = cli.config_file.as_path();
    if !config_path.exists() {
        bail!("Unable to find file {:?}", config_path);
    }

    debug!("Loading config file {}", config_path.display());

    let figment = Figment::new().merge(Toml::file(config_path));
    let user_config: user::UserConfig = figment.extract()?;

    debug!("Loaded config {:?}", user_config);

    let mut http_plugins: Vec<_> = Vec::new();
    for plugin_config in user_config.http.plugin {
        let plugin = create_http_backend(
            config_path.parent().expect("to have a parent"),
            plugin_config,
        )
        .await?;
        http_plugins.push(Arc::new(plugin));
    }

    Ok(ServerConfig {
        http_address: user_config.http.address,
        http_plugins,
    })
}

pub async fn create_http_backend(
    config_dir: &Path,
    value: crate::config::PluginConfig,
) -> Result<Box<dyn HttpPlugin>, anyhow::Error> {
    let backend = match value.source {
        PluginSource::LuaPlugin(_lua) => todo!(),
        PluginSource::WasmPlugin(_wasm) => todo!(),
        PluginSource::Static(config) => {
            Box::new(dev_null_plugin_http::build_plugin(config, config_dir).await?)
        }
    };

    Ok(backend)
}
