use anyhow::Result as AnyResult;
use config::Config;
use glob::glob;
use plugins::MakeStatic;
use std::path::{Path, PathBuf};
use tracing::debug;

mod plugins;
mod user;

pub struct ConfigContainer {
    root_config_file: PathBuf,
    base_config_dir: PathBuf,
}

pub struct ParsedUserConfig {
    pub root: user::ServerRootConfig,
    pub responses: Vec<plugins::ResponseConfig<String>>,
}

impl ConfigContainer {
    pub fn new(root_config_file: &Path) -> Self {
        Self {
            root_config_file: root_config_file.to_path_buf(),
            base_config_dir: root_config_file
                .parent()
                .expect("parent to exist")
                .canonicalize()
                .expect("a good dir"),
        }
    }
    pub fn load_config(&self) -> AnyResult<ParsedUserConfig> {
        debug!("Loading root config {:?}", self.root_config_file);
        let root_config = Config::builder()
            .add_source(config::File::from(self.root_config_file.as_path()))
            .build()?;

        let parsed_root: user::ServerRootConfig = root_config.try_deserialize()?;

        let mut response_paths = Vec::new();

        for config_path in &parsed_root.responses.paths {
            let glob_path = format!("{}/{}", self.base_config_dir.display(), config_path);

            debug!("Glob path: {}", glob_path);
            let mut found_files = glob(&glob_path)?.filter_map(|x| x.ok()).collect();
            response_paths.append(&mut found_files);
        }

        let mut parsed_responses = Vec::new();
        for response_file in response_paths {
            debug!("Loading dependent config {:?}", response_file);
            let response_file = response_file.as_path();
            let response_config = Config::builder()
                .add_source(config::File::from(response_file))
                .build()?;
            let parsed = response_config
                .try_deserialize::<plugins::ResponseConfig<plugins::ResponseData>>()?;
            let parent_folder = response_file.parent().expect("partent path to exist");

            parsed_responses.push(parsed.make_static(parent_folder)?);
        }

        Ok(ParsedUserConfig {
            root: parsed_root,
            responses: parsed_responses,
        })
    }
}

pub mod prelude {
    pub use crate::plugins::static_http::*;
    pub use crate::plugins::{ResponseConfig, ResponseData};
    pub use crate::{ConfigContainer, ParsedUserConfig};
}
