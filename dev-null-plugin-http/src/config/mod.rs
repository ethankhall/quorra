pub(crate) mod internal;
mod public;
use std::path::Path;

use figment::{
    providers::{Format, YamlExtended},
    Figment,
};
use tracing::debug;

pub use public::*;

pub async fn load_http_plugin_config(
    plugin_def: StaticPluginDef,
    config_dir: &Path,
) -> Result<internal::PluginBackendConfig, crate::HttpPluginError> {
    let config_path = if plugin_def.config_path.is_absolute() {
        plugin_def.config_path
    } else {
        config_dir.join(plugin_def.config_path)
    };

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Unable to find file {:?}",
            config_path.display().to_string()
        )
        .into());
    }

    debug!("Loading HTTP plugin config from {}", config_path.display());
    let figment = Figment::new().merge(YamlExtended::file(config_path));
    debug!("Loaded config to be {:?}", figment);
    let public_config: StaticPluginConfig = figment.extract()?;

    internal::PluginBackendConfig::try_from(public_config)
}
