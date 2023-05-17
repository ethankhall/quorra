use bytes::Bytes;
use http::{header::HeaderMap, Method, Response, StatusCode, Uri};

#[derive(Debug, Default)]
pub struct EmptyReponse {}

#[async_trait::async_trait]
impl crate::HttpPlugin for EmptyReponse {
    async fn respond_to_request(
        &self,
        _method: &Method,
        _uri: &Uri,
        _headers: &HeaderMap,
        _body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>> {
        None
    }
}

#[derive(Debug, Default)]
pub struct ConstantResponse {
    body: Option<String>,
}

#[async_trait::async_trait]
impl crate::HttpPlugin for ConstantResponse {
    async fn respond_to_request(
        &self,
        _method: &Method,
        _uri: &Uri,
        _headers: &HeaderMap,
        _body: &Option<&Bytes>,
    ) -> Option<Response<Bytes>> {
        let body = self.body.clone().unwrap_or_default();
        let body = Bytes::from(format!("const `{}`", body));
        Some(
            Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .unwrap(),
        )
    }
}
