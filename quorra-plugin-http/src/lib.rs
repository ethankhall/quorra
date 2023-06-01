use thiserror::Error;

mod config;
mod http_static;

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use lazy_static::lazy_static;
use quorra_config::prelude::StaticHttpConfig;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::RwLock,
};
use tracing::debug;

lazy_static! {
    static ref ID_COUNTER: AtomicU64 = AtomicU64::from(0);
}

fn uuid_generator(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write(&uuid::Uuid::new_v4().to_string())?;
    Ok(())
}

fn id_generator(
    _h: &Helper,
    _: &Handlebars,
    _: &Context,
    _rc: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    out.write(&ID_COUNTER.fetch_add(1, Ordering::SeqCst).to_string())?;
    Ok(())
}

lazy_static! {
    pub(crate) static ref HANDLEBARS: RwLock<Box<Handlebars<'static>>> = {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        handlebars.register_helper("uuid", Box::new(uuid_generator));
        handlebars.register_helper("id", Box::new(id_generator));
        RwLock::new(Box::new(handlebars))
    };
}

#[derive(Error, Debug)]
pub enum HttpPluginError {
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
    #[error(transparent)]
    TemplateError(#[from] handlebars::TemplateError),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

#[derive(Debug, Default)]
pub struct HttpStaticPluginBuilder {
    configs: Vec<StaticHttpConfig<String>>,
}

impl HttpStaticPluginBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_config(&mut self, config: &StaticHttpConfig<String>) {
        self.configs.push(config.clone());
    }

    pub fn build(self) -> Result<http_static::HttpStaticPlugin, HttpPluginError> {
        debug!("{} responses loaded", self.configs.len());
        let plugin_config = crate::config::internal::PluginBackendConfig::try_from(&self.configs)?;

        Ok(http_static::HttpStaticPlugin {
            config: plugin_config,
        })
    }
}

#[test]
fn test_unique_uuid() {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_helper("uuid", Box::new(uuid_generator));

    handlebars
        .register_template_string("uuid", "{{ uuid }}\n{{ uuid }}")
        .unwrap();
    let rendered = handlebars.render("uuid", &serde_json::Value::Null).unwrap();

    let lines: Vec<&str> = rendered.split('\n').collect();
    assert_eq!(2, lines.len());
    assert_ne!(lines[0], lines[1]);
}

#[test]
fn test_unique_id() {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_helper("id", Box::new(id_generator));

    let start = ID_COUNTER.load(Ordering::SeqCst);

    handlebars
        .register_template_string("id", "{{ id }}\n{{ id }}")
        .unwrap();
    let rendered = handlebars.render("id", &serde_json::Value::Null).unwrap();

    let lines: Vec<&str> = rendered.split('\n').collect();
    assert_eq!(2, lines.len());
    assert_eq!(start, lines[0].parse::<u64>().unwrap());
    assert_eq!(start + 1, lines[1].parse::<u64>().unwrap());

    let rendered = handlebars.render("id", &serde_json::Value::Null).unwrap();

    let lines: Vec<&str> = rendered.split('\n').collect();
    assert_eq!(2, lines.len());
    assert_eq!(start + 2, lines[0].parse::<u64>().unwrap());
    assert_eq!(start + 3, lines[1].parse::<u64>().unwrap());
}
