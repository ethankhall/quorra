use super::{MakeStatic, ResponseData};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::BTreeMap, path::Path};

/// A static response configuration
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct StaticHttpConfig<T> {
    /// Unique ID that is included with every response when the plugin matches.
    /// When not provided, a random one will be generated.
    #[serde(default = "unique_id")]
    pub id: String,

    /// A list of ways that the request can be matched against.
    pub matches: Vec<StaticMatchesConfig>,

    /// A list of possible responses.
    pub responses: Vec<StaticResponseConfig<T>>,
}

impl MakeStatic<StaticHttpConfig<String>> for StaticHttpConfig<ResponseData> {
    fn make_static(&self, file_path: &Path) -> anyhow::Result<StaticHttpConfig<String>> {
        let mut responses: Vec<_> = Default::default();
        for resp in &self.responses {
            responses.push(resp.make_static(file_path)?);
        }

        Ok(StaticHttpConfig {
            id: self.id.clone(),
            matches: self.matches.clone(),
            responses,
        })
    }
}

/// The possible options to match against. All fields are optional. When all
/// fields are missing, the request will match.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub struct StaticMatchesConfig {
    /// A regex to be used to match against the request path.
    #[serde(default)]
    pub path: String,

    /// A list of String=String pairs that will be matched against the query params
    #[serde_as(as = "serde_with::Map<_, _>")]
    #[serde(default)]
    pub query: Vec<(String, String)>,

    /// A map of key-value pairs. The key is the header name, the
    /// value used to match the header against.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// A list of methods the request should be.
    #[serde(default)]
    pub methods: Vec<String>,

    /// Configuration for GraphQL body matchers
    #[serde(default)]
    pub graphql: Option<GraphqlStaticMatchConfig>,
}

/// GraphQL body matcher
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub struct GraphqlStaticMatchConfig {
    /// The name of the GraphQL operation to respond to
    #[serde(default)]
    pub operation_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct StaticResponseConfig<T> {
    #[serde(default = "unique_id")]
    pub id: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<StaticResponseBodyConfig<T>>,
    #[serde(default)]
    pub delay: u64,
}

impl MakeStatic<StaticResponseConfig<String>> for StaticResponseConfig<ResponseData> {
    fn make_static(&self, file_path: &Path) -> anyhow::Result<StaticResponseConfig<String>> {
        let body = match &self.body {
            None => None,
            Some(body) => Some(body.make_static(file_path)?),
        };
        Ok(StaticResponseConfig {
            id: self.id.clone(),
            weight: self.weight,
            status: self.status,
            headers: self.headers.clone(),
            body,
            delay: self.delay,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(tag = "type", bound = "T: Serialize + DeserializeOwned")]
pub enum StaticResponseBodyConfig<T> {
    #[serde(rename = "raw")]
    Raw(T),
    #[serde(rename = "json")]
    Json(T),
    Empty,
}

impl MakeStatic<StaticResponseBodyConfig<String>> for StaticResponseBodyConfig<ResponseData> {
    fn make_static(&self, file_path: &Path) -> anyhow::Result<StaticResponseBodyConfig<String>> {
        match self {
            StaticResponseBodyConfig::Json(json) => {
                Ok(StaticResponseBodyConfig::Json(json.make_static(file_path)?))
            }
            StaticResponseBodyConfig::Raw(raw) => {
                Ok(StaticResponseBodyConfig::Raw(raw.make_static(file_path)?))
            }
            StaticResponseBodyConfig::Empty => Ok(StaticResponseBodyConfig::Empty),
        }
    }
}

fn default_weight() -> u16 {
    1
}

pub fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
