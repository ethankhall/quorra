use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The plugin {0} is requires a config file.")]
    PluginMissingConfigFile(String),
}

#[derive(Error, Debug)]
pub enum HttpStaticError {
    #[error(transparent)]
    HttpError(#[from] http::Error),
    #[error(transparent)]
    InvalidStatusCode(#[from] http::status::InvalidStatusCode),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),
    #[error("No respone configured for match")]
    NoResponsesProvided,
}
