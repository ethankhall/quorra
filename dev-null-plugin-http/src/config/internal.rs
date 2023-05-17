use crate::HttpPluginError;
use dev_null_config::prelude::*;
use http::{
    header::CONTENT_TYPE,
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, StatusCode,
};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use std::str::FromStr;
use std::sync::{atomic::AtomicUsize, Arc};
use std::{collections::BTreeMap, time::Duration};

#[derive(Debug, Clone)]
pub struct PluginBackendConfig {
    pub payloads: Vec<PayloadBackendConfig>,
}

impl PluginBackendConfig {
    pub fn try_from(configs: &[StaticHttpConfig<String>]) -> Result<Self, HttpPluginError> {
        let mut payloads = Vec::new();
        for http_config in configs {
            payloads.push(PayloadBackendConfig::from_http_config(http_config)?);
        }
        Ok(Self { payloads })
    }
}

#[derive(Debug, Clone)]
pub struct PayloadBackendConfig {
    pub id: String,
    pub matchers: Vec<RequestMatcher>,
    pub responses: StaticResponseContainer,
}

impl PayloadBackendConfig {
    fn from_http_config(config: &StaticHttpConfig<String>) -> Result<Self, HttpPluginError> {
        let payload_id = config.id.clone();
        let responses: StaticResponseContainer =
            StaticResponseContainer::try_from(&payload_id, &config.responses)?;
        let matchers: Result<Vec<_>, _> = config
            .matches
            .iter()
            .map(RequestMatcher::from_matches_config)
            .collect();

        Ok(Self {
            id: payload_id,
            responses,
            matchers: matchers?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RequestMatcher {
    pub methods: Vec<Method>,
    pub path: Regex,
    pub query_params: Vec<QueryMatcher>,
    pub headers: Vec<HeaderMatcher>,
    pub graphql_operations: Option<Regex>,
}

impl RequestMatcher {
    fn from_matches_config(config: &StaticMatchesConfig) -> Result<Self, HttpPluginError> {
        let gql_operations = config.graphql.clone().map(|x| x.operation_name);
        RequestMatcher::new(
            &config.methods,
            &config.path,
            &config.query,
            &config.headers,
            &gql_operations,
        )
    }

    pub fn new(
        methods: &Vec<String>,
        path: &String,
        query_params: &Vec<(String, String)>,
        headers: &BTreeMap<String, String>,
        graphql_operations: &Option<String>,
    ) -> Result<Self, HttpPluginError> {
        let matched_path = Regex::new(&format!("^{}$", &path))?;
        let mut matched_headers = Vec::new();
        for (name, value) in headers {
            matched_headers.push(HeaderMatcher::new(name, value)?);
        }

        let mut matched_query = Vec::new();
        for (name, value) in query_params {
            matched_query.push(QueryMatcher::new(name, value)?);
        }

        let mut parsed_methods = Vec::new();
        for method in methods {
            parsed_methods.push(Method::from_bytes(method.as_bytes())?);
        }

        let matched_graphql = match graphql_operations {
            None => None,
            Some(operation) => Some(Regex::new(operation)?),
        };

        Ok(Self {
            path: matched_path,
            query_params: matched_query,
            headers: matched_headers,
            methods: parsed_methods,
            graphql_operations: matched_graphql,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HeaderMatcher {
    pub name: HeaderName,
    pub value: Regex,
}

impl HeaderMatcher {
    fn new(name: &str, value: &str) -> Result<Self, HttpPluginError> {
        Ok(Self {
            name: HeaderName::from_str(name)?,
            value: Regex::new(value)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct QueryMatcher {
    pub name: String,
    pub value: Regex,
}

impl QueryMatcher {
    fn new(name: &str, value: &str) -> Result<Self, HttpPluginError> {
        Ok(Self {
            name: name.to_string(),
            value: Regex::new(value)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StaticResponseContainer {
    pub pointer: Arc<AtomicUsize>,
    pub responses: Vec<Arc<StaticResponse>>,
}

impl StaticResponseContainer {
    fn try_from(
        payload_id: &str,
        configs: &[StaticResponseConfig<String>],
    ) -> Result<Self, HttpPluginError> {
        let mut configs = configs.to_owned();
        configs.sort_by(|a, b| a.weight.cmp(&b.weight));

        let mut responses = Vec::new();

        for config in configs {
            let static_response = StaticResponse::try_from(payload_id, &config)?;
            let static_response: Arc<StaticResponse> = Arc::new(static_response);
            for _i in 0..config.weight {
                responses.push(static_response.clone());
            }
        }

        responses.shuffle(&mut thread_rng());

        if responses.is_empty() {
            return Err(HttpPluginError::NoResponsesProvided);
        }

        Ok(Self {
            pointer: Arc::new(AtomicUsize::new(0)),
            responses,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StaticResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub handlebar_template_id: String,
    pub delay: Duration,
}

impl StaticResponse {
    fn try_from(
        payload_id: &str,
        value: &StaticResponseConfig<String>,
    ) -> Result<Self, HttpPluginError> {
        let status_code = StatusCode::from_u16(value.status)?;

        let mut headers = HeaderMap::new();

        let body_text = match &value.body {
            None => "".to_string(),
            Some(StaticResponseBodyConfig::Json(data)) => {
                headers.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
                data.to_string()
            }
            Some(StaticResponseBodyConfig::Raw(data)) => data.to_string(),
        };

        let mut handlebars = crate::HANDLEBARS.write().unwrap();
        handlebars.register_template_string(&value.id, body_text)?;

        {
            for (name, value) in &value.headers {
                headers.insert(
                    HeaderName::from_bytes(name.as_bytes())?,
                    HeaderValue::from_bytes(value.as_bytes())?,
                );
            }

            add_header(&mut headers, "response-id", &value.id);
            add_header(&mut headers, "payload-id", payload_id);
        }

        Ok(StaticResponse {
            status: status_code,
            headers,
            handlebar_template_id: value.id.clone(),
            delay: Duration::from_millis(value.delay),
        })
    }
}

fn add_header(headers: &mut HeaderMap, name: &str, value: &str) {
    let value = match HeaderValue::from_str(value) {
        Ok(value) => value,
        Err(_) => HeaderValue::from_str(&format!("{} invalid header", name)).unwrap(),
    };
    headers.insert(
        HeaderName::from_str(&format!("x-dev-null-{}", name)).unwrap(),
        value,
    );
}
