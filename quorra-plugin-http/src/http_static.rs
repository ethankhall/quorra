use std::{collections::BTreeMap, sync::atomic::Ordering};
use tracing::{debug, instrument};

use crate::config::internal::*;
use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;
use tokio::time::sleep;

use http::{HeaderMap, Method, Response, Uri};

impl StaticResponse {
    #[instrument(skip_all, fields(payload.id = payload_id))]
    async fn make_response(
        &self,
        payload_id: &str,
        request_body: &Option<&Bytes>,
    ) -> Response<Bytes> {
        let body_string = match request_body {
            None => Value::Null,
            Some(body) => Value::from(String::from_utf8(body.to_vec()).unwrap_or_default()),
        };

        debug!("Starting a wait of {:?}", self.delay);
        sleep(self.delay).await;

        let values = BTreeMap::from([
            ("quorra_payload_id", Value::from(payload_id)),
            ("request_body", body_string),
        ]);

        let handlebars = crate::HANDLEBARS.read().unwrap();
        let body = match handlebars.render(&self.handlebar_template_id, &values) {
            Ok(body) => body,
            Err(e) => format!(
                "Quorra encoundered an error rendering the response. Error {}",
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
        uri: &Uri,
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
        uri: &Uri,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> bool {
        if !self.methods.is_empty() && !self.methods.contains(method) {
            return false;
        }
        debug!("Matched method");

        if !self.path.is_match(uri.path()) {
            return false;
        }
        debug!("Matched uri");

        if !self.matches_query(uri) {
            return false;
        }
        debug!("Matched query");

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

    fn matches_query(&self, uri: &Uri) -> bool {
        if self.query_params.is_empty() {
            return true;
        }

        let query = match uri.query() {
            Some(q) => q,
            None => return false,
        };

        let mut query_param_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let query_params = form_urlencoded::parse(query.as_bytes()).into_owned();
        for pair in query_params {
            query_param_map
                .entry(pair.0)
                .and_modify(|list| list.push(pair.1.clone()))
                .or_insert_with(|| vec![pair.1]);
        }

        self.query_params.iter().all(|query_matcher| {
            if let Some(values) = query_param_map.get(&query_matcher.name) {
                values
                    .iter()
                    .all(|value| query_matcher.value.is_match(value))
            } else {
                false
            }
        })
    }
}

#[test]
fn test_request_matcher_empty() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &".*".to_string(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    let empty_uri = "/".parse::<Uri>().unwrap();
    assert!(matcher.request_matches(&Method::OPTIONS, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::GET, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::PUT, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::DELETE, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::HEAD, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::TRACE, &empty_uri, &Default::default(), &None));

    assert!(matcher.request_matches(&Method::POST, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::CONNECT, &empty_uri, &Default::default(), &None));
    assert!(matcher.request_matches(&Method::PATCH, &empty_uri, &Default::default(), &None));

    assert!(matcher.request_matches(
        &Method::GET,
        &"/foo/bar".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
}

#[test]
fn test_request_matcher_method() {
    let matcher = RequestMatcher::new(
        &vec!["GET".to_owned()],
        &".*".to_string(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(
        &Method::GET,
        &"/".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(!matcher.request_matches(
        &Method::PUT,
        &"/".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
}

#[test]
fn test_request_matcher_path() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &"/foo/bar".to_owned(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(matcher.request_matches(
        &Method::GET,
        &"/foo/bar".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(!matcher.request_matches(
        &Method::GET,
        &"/foo/barasdfa".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(!matcher.request_matches(
        &Method::GET,
        &"/foo/bar/".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
}

#[test]
fn test_request_matcher_query() {
    let matcher = RequestMatcher::new(
        &Default::default(),
        &".*".to_string(),
        &vec![("foo".to_string(), "bar".to_string())],
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert!(!matcher.request_matches(
        &Method::GET,
        &"/foo/bar".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(matcher.request_matches(
        &Method::GET,
        &"/test?foo=bar".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(!matcher.request_matches(
        &Method::GET,
        &"/test?foo=bar&foo=baz".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
    assert!(!matcher.request_matches(
        &Method::GET,
        &"/test?foo=baz".parse::<Uri>().unwrap(),
        &Default::default(),
        &None
    ));
}

#[derive(Debug, Clone)]
pub struct HttpStaticPlugin {
    pub config: PluginBackendConfig,
}

#[async_trait]
impl quorra_plugin::HttpPlugin for HttpStaticPlugin {
    #[instrument(skip_all)]
    async fn respond_to_request(
        &self,
        method: &Method,
        uri: &Uri,
        headers: &HeaderMap,
        body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>> {
        for payload in &self.config.payloads {
            if payload.matches(method, uri, headers, body) {
                return Some(
                    payload
                        .responses
                        .get_response()
                        .make_response(&payload.id, body)
                        .await,
                );
            }
        }

        None
    }
}
