use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};
use tracing::{debug, instrument};

use crate::config::internal::*;
use async_trait::async_trait;
use bytes::Bytes;
use lazy_static::lazy_static;
use serde_json::Value;

use http::{HeaderMap, Method, Response};

lazy_static! {
    static ref ID_COUNTER: AtomicU64 = AtomicU64::from(0);
}

impl StaticResponse {
    #[instrument(skip_all, fields(plugin.id = plugin_id, payload.id = payload_id))]
    fn make_response(
        &self,
        plugin_id: &str,
        payload_id: &str,
        request_body: &Option<&Bytes>,
    ) -> Response<Bytes> {
        let body_string = match request_body {
            None => Value::Null,
            Some(body) => Value::from(String::from_utf8(body.to_vec()).unwrap_or_default()),
        };

        let values = BTreeMap::from([
            ("uuid", Value::from(uuid::Uuid::new_v4().to_string())),
            ("id", Value::from(ID_COUNTER.fetch_add(1, Ordering::SeqCst))),
            ("dev_null_plugin_id", Value::from(plugin_id)),
            ("dev_null_payload_id", Value::from(payload_id)),
            ("request_body", body_string),
        ]);

        let handlebars = crate::HANDLEBARS.read().unwrap();
        let body = match handlebars.render(&self.handlebar_template_id, &values) {
            Ok(body) => body,
            Err(e) => format!(
                "DevNull encoundered an error rendering the response. Error {}",
                e
            ),
        };
        let body = Bytes::from(body);

        let mut response = Response::builder().status(self.status);

        {
            if let Some(headers) = response.headers_mut() {
                headers.clone_from(&self.headers);
            }
        }

        response.body(body).unwrap()
    }
}

impl StaticResponseContainer {
    pub fn get_response(&self) -> &StaticResponse {
        let response_lenght = self.responses.len();
        let value = match self
            .pointer
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                Some((value + 1) % response_lenght)
            }) {
            Ok(value) => value,
            Err(value) => value,
        };

        self.responses.get(value).unwrap()
    }
}

impl PayloadBackendConfig {
    #[instrument(skip_all, fields(container.id = self.id))]
    fn matches(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> bool {
        self.matchers
            .iter()
            .any(|x| x.request_matches(method, uri, headers, body))
    }
}

impl RequestMatcher {
    fn request_matches(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> bool {
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

        if !self.matches_headers(headers) {
            return false;
        }
        debug!("Matched headers");

        if !self.match_graphql_operation(body) {
            return false;
        }

        debug!("Matched Body");

        true
    }

    fn match_graphql_operation(&self, body: &Option<&Bytes>) -> bool {
        let matcher = match &self.graphql_operations {
            None => return true,
            Some(matcher) => matcher,
        };

        match body {
            None => false,
            Some(body) => {
                let body_text = match String::from_utf8(body.to_vec()) {
                    Ok(body) => body,
                    Err(_e) => {
                        debug!("Unable to extract string from body of request");
                        return false;
                    }
                };
                match json::parse(&body_text) {
                    Err(_) => false,
                    Ok(body) => {
                        debug!("body[operationName] = {:?}", body["operationName"]);
                        match &body["operationName"].as_str() {
                            Some(name) => matcher.is_match(name),
                            None => false,
                        }
                    }
                }
            }
        }
    }

    fn matches_headers(&self, request_headers: &HeaderMap) -> bool {
        self.headers.iter().all(|header_matcher| {
            let values = request_headers.get_all(&header_matcher.name);
            values.iter().any(|value| {
                header_matcher
                    .value
                    .is_match(value.to_str().unwrap_or_default())
            })
        })
    }
}

#[test]
fn test_request_matcher_empty() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::OPTIONS, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::GET, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::PUT, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::DELETE, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::HEAD, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::TRACE, "", &Default::default(), &None));

    assert!(matcher.request_matches(&Method::POST, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::CONNECT, "", &Default::default(), &None));
    assert!(matcher.request_matches(&Method::PATCH, "", &Default::default(), &None));

    assert!(matcher.request_matches(&Method::GET, "/foo/bar", &Default::default(), &None));
}

#[test]
fn test_request_matcher_method() {
    let matcher = RequestMatcher::new(
        &vec!["GET".to_owned()],
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::GET, "", &Default::default(), &None));
    assert!(!matcher.request_matches(&Method::PUT, "", &Default::default(), &None));
}

#[test]
fn test_request_matcher_path() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &Some("/foo/bar".to_owned()),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(&Method::GET, "/foo/bar", &Default::default(), &None));
    assert!(!matcher.request_matches(&Method::GET, "/foo/barasdfa", &Default::default(), &None));
    assert!(!matcher.request_matches(&Method::GET, "/foo/bar/", &Default::default(), &None));
}

#[derive(Debug, Clone)]
pub struct HttpStaticPlugin {
    pub config: PluginBackendConfig,
}

#[async_trait]
impl dev_null_plugin::HttpPlugin for HttpStaticPlugin {
    #[instrument(skip_all, fields(plugin.id = self.config.id))]
    async fn respond_to_request(
        &self,
        method: &Method,
        uri: &str,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>> {
        for payload in &self.config.payloads {
            if payload.matches(method, uri, headers, body) {
                return Some(payload.responses.get_response().make_response(
                    &self.config.id,
                    &payload.id,
                    body,
                ));
            }
        }

        None
    }
}
