use dev_null_config::prelude::{ParsedUserConfig, ResponseConfig};
use dev_null_plugin::HttpPlugin;
use dev_null_plugin_http::HttpStaticPluginBuilder;

use std::sync::Arc;

mod cli;
mod logging;

pub use cli::{Opts, SubCommands};
pub use logging::*;

pub async fn build_backends(
    container: &ParsedUserConfig,
) -> Result<Vec<Arc<Box<dyn HttpPlugin>>>, anyhow::Error> {
    let mut http_static_builder = HttpStaticPluginBuilder::new();
    for response_config in &container.responses {
        match response_config {
            ResponseConfig::StaticHttp(http) => http_static_builder.load_config(http),
        }
    }

    let boxed = Box::new(http_static_builder.build()?);

    Ok(vec![Arc::new(boxed)])
}
