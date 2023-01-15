use std::{collections::BTreeMap, path::PathBuf, str::FromStr};
use tracing::{debug, instrument};

use super::StaticResponseContainer;
use crate::config::http::http_static::*;
use crate::errors::*;
use crate::plugin::HttpPlugin;
use async_trait::async_trait;
use bytes::Bytes;

use http::{header::HeaderName, HeaderMap, Method, Response};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct StaticContainer {
    id: String,
    matchers: Vec<RequestMatcher>,
    responses: StaticResponseContainer,
}

impl StaticContainer {
    #[instrument(skip_all, fields(container.id = self.id))]
    fn matches(&self, method: &Method, uri: &str, headers: &HeaderMap) -> bool {
        self.matchers
            .iter()
            .any(|x| x.request_matches(method, uri, headers))
    }
}

impl TryFrom<StaticHttpConfig> for StaticContainer {
    type Error = HttpStaticError;

    fn try_from(config: StaticHttpConfig) -> Result<Self, Self::Error> {
        let responses: StaticResponseContainer = config.response.try_into()?;
        let matchers: Result<Vec<_>, _> = config
            .matches
            .iter()
            .map(RequestMatcher::try_from)
            .collect();

        Ok(Self {
            id: config.id,
            responses,
            matchers: matchers?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestMatcher {
    pub methods: Vec<Method>,
    pub path: Option<Regex>,
    pub headers: Vec<HeaderMatcher>,
}

impl RequestMatcher {
    pub fn new(
        methods: &Vec<String>,
        path: &Option<String>,
        headers: &BTreeMap<String, String>,
    ) -> Result<Self, HttpStaticError> {
        let matched_path = match &path {
            Some(path) => Some(Regex::new(&format!("^{}$", &path))?),
            None => None,
        };
        let mut matched_headers = Vec::new();
        for (name, value) in headers {
            matched_headers.push(HeaderMatcher::new(name, value)?);
        }

        let mut parsed_methods = Vec::new();
        for method in methods {
            parsed_methods.push(Method::from_bytes(method.as_bytes())?);
        }

        Ok(Self {
            path: matched_path,
            headers: matched_headers,
            methods: parsed_methods,
        })
    }

    fn request_matches(&self, method: &Method, uri: &str, headers: &HeaderMap) -> bool {
        if !self.methods.is_empty() && !self.methods.contains(method) {
            return false;
        }

        debug!("Matched method");

        if let Some(path) = &self.path {
            if !path.is_match(uri) {
                return false;
            }
        }

        debug!("Matched uri");

        if self.headers.is_empty() {
            return true;
        }

        self.headers.iter().all(|header_matcher| {
            let values = headers.get_all(&header_matcher.name);
            values.iter().any(|value| {
                header_matcher
                    .value
                    .is_match(value.to_str().unwrap_or_default())
            })
        })
    }
}

impl TryFrom<&StaticMatchesConfig> for RequestMatcher {
    type Error = HttpStaticError;

    fn try_from(config: &StaticMatchesConfig) -> Result<Self, Self::Error> {
        RequestMatcher::new(&config.methods, &config.path, &config.headers)
    }
}

#[test]
fn test_request_matcher_empty() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::OPTIONS, "", &Default::default()));
    assert!(matcher.request_matches(&Method::GET, "", &Default::default()));
    assert!(matcher.request_matches(&Method::PUT, "", &Default::default()));
    assert!(matcher.request_matches(&Method::DELETE, "", &Default::default()));
    assert!(matcher.request_matches(&Method::HEAD, "", &Default::default()));
    assert!(matcher.request_matches(&Method::TRACE, "", &Default::default()));

    assert!(matcher.request_matches(&Method::POST, "", &Default::default()));
    assert!(matcher.request_matches(&Method::CONNECT, "", &Default::default()));
    assert!(matcher.request_matches(&Method::PATCH, "", &Default::default()));

    assert!(matcher.request_matches(&Method::GET, "/foo/bar", &Default::default()));
}

#[test]
fn test_request_matcher_method() {
    let matcher = RequestMatcher::new(
        &vec!["GET".to_owned()],
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::GET, "", &Default::default()));
    assert!(!matcher.request_matches(&Method::PUT, "", &Default::default()));
}

#[test]
fn test_request_matcher_path() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &Some("/foo/bar".to_owned()),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::GET, "/foo/bar", &Default::default()));
    assert!(!matcher.request_matches(&Method::GET, "/foo/barasdfa", &Default::default()));
    assert!(!matcher.request_matches(&Method::GET, "/foo/bar/", &Default::default()));
}

#[derive(Debug, Clone)]
pub struct HeaderMatcher {
    name: HeaderName,
    value: Regex,
}

impl HeaderMatcher {
    fn new(name: &str, value: &str) -> Result<Self, HttpStaticError> {
        Ok(Self {
            name: HeaderName::from_str(name)?,
            value: Regex::new(value)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpStaticPlugin {
    id: String,
    static_containers: Vec<StaticContainer>,
}

impl TryFrom<PathBuf> for HttpStaticPlugin {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let config = crate::config::http::http_static::parse(path)?;
        let mut static_containers = Vec::new();

        for static_config in config.http {
            static_containers.push(StaticContainer::try_from(static_config)?);
        }

        debug!(
            "Static Response {} Config {:?}",
            config.id, static_containers
        );
        Ok(Self {
            id: config.id,
            static_containers,
        })
    }
}

#[async_trait]
impl HttpPlugin for HttpStaticPlugin {
    #[instrument(skip_all, fields(plugin.id = self.id))]
    async fn respond_to_request(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        _body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>> {
        for container in &self.static_containers {
            if container.matches(method, uri, headers) {
                return Some(
                    container
                        .responses
                        .get_response()
                        .make_response(&container.id),
                );
            }
        }

        None
    }
}
