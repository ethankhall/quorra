use std::{
    fmt::Debug,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::debug;

pub mod static_http;

pub trait MakeStatic<T> {
    fn make_static(&self, file_path: &Path) -> anyhow::Result<T>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseData {
    #[serde(rename = "data")]
    Data(String),
    #[serde(rename = "file")]
    File(PathBuf),
}

impl MakeStatic<String> for ResponseData {
    fn make_static(&self, config_file_path: &Path) -> anyhow::Result<String> {
        match self {
            ResponseData::File(path) => {
                let file_to_load = config_file_path.join(path);
                debug!("Loading data file {:?}", file_to_load);
                Ok(read_to_string(file_to_load)?)
            }
            ResponseData::Data(data) => Ok(data.clone()),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", bound = "T: Serialize + DeserializeOwned")]
pub enum ResponseConfig<T> {
    #[serde(rename = "static-http")]
    StaticHttp(static_http::StaticHttpConfig<T>),
}

impl MakeStatic<ResponseConfig<String>> for ResponseConfig<ResponseData> {
    fn make_static(&self, file_path: &Path) -> anyhow::Result<ResponseConfig<String>> {
        match self {
            Self::StaticHttp(plugin) => {
                Ok(ResponseConfig::StaticHttp(plugin.make_static(file_path)?))
            }
        }
    }
}
