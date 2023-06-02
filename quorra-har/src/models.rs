use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HarFile {
    pub log: LogEntry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameValueEntry {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub entries: Vec<RequestWrapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestEntry {
    pub method: String,
    pub url: String,
    pub headers: Vec<NameValueEntry>,
    pub cookies: Vec<NameValueEntry>,
    pub query_string: Vec<NameValueEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub mime_type: String,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEntry {
    pub status: i32,
    pub status_text: String,
    pub headers: Vec<NameValueEntry>,
    pub cookies: Vec<NameValueEntry>,
    pub content: Content,
    #[serde(rename = "redirectURL")]
    pub redirect_url: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestWrapper {
    pub started_date_time: String,
    pub request: RequestEntry,
    pub response: ResponseEntry,
}
