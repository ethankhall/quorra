use crate::models::*;
use anyhow::Error;
use dev_null_config::prelude::*;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tracing::info;
use url::Url;

pub struct HarConvertor {
    path: PathBuf,
}

pub struct EntryWrapperWithBody {
    matcher: StaticMatchesConfig,
    response_config: StaticResponseConfig<()>,
    body: Option<String>,
}

pub struct EntryWrapper {
    matcher: StaticMatchesConfig,
    response_config: StaticResponseConfig<ResponseData>,
}

impl HarConvertor {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
        }
    }

    pub async fn convert(&self, dest: &Path) -> Result<(), Error> {
        let har_contents: HarFile = serde_json::from_str(&std::fs::read_to_string(&self.path)?)?;
        let mut wrapper_map: BTreeMap<
            StaticMatchesConfig,
            Vec<StaticResponseConfig<ResponseData>>,
        > = Default::default();

        for entry in har_contents.log.entries {
            let converted_config = convert_entry(&entry).await?;
            let wrapper = write_body(dest, converted_config).await?;
            wrapper_map
                .entry(wrapper.matcher)
                .and_modify(|value| value.push(wrapper.response_config.clone()))
                .or_insert_with(|| vec![wrapper.response_config]);
        }

        for (key, value) in wrapper_map {
            let config = StaticHttpConfig {
                id: unique_id(),
                matches: vec![key.clone()],
                responses: value,
            };

            let path = if key.path == "/" {
                "/root.html"
            } else {
                &key.path
            };

            let filename = format!("{}{}.yaml", key.methods[0], path.replace('/', "__"));
            let output = serde_yaml::to_string(&config)?;
            let path = dest.join(filename);
            info!("Writing file {}", path.display().to_string());
            std::fs::write(path, output)?;
        }

        Ok(())
    }
}

fn matcher_to_filename(request: &StaticMatchesConfig, unique: String) -> String {
    let path = if request.path == "/" {
        "/root.html"
    } else {
        &request.path
    };
    let path = path.replace('/', "__");
    let (path, extension) = match path.rsplit_once('.') {
        Some((name, extension)) => (name, format!(".{}", extension)),
        None => (path.as_str(), "".to_string()),
    };

    format!(
        "{}{}_{}{}",
        request.methods[0],
        path,
        unique[..6].chars().as_str(),
        extension
    )
}

async fn write_body(dir: &Path, wrapper: EntryWrapperWithBody) -> Result<EntryWrapper, Error> {
    let body: Option<StaticResponseBodyConfig<ResponseData>> = match wrapper.body {
        Some(body_text) => {
            let unique = format!("{:x}", md5::compute(&body_text));
            let filename = matcher_to_filename(&wrapper.matcher, unique);

            info!("Creating asset file {}", filename);

            let path = dir.join(&filename);
            std::fs::write(path, body_text)?;

            Some(StaticResponseBodyConfig::Raw(ResponseData::File(
                filename.into(),
            )))
        }
        None => None,
    };

    let response_config: StaticResponseConfig<ResponseData> = StaticResponseConfig {
        id: wrapper.response_config.id,
        weight: wrapper.response_config.weight,
        status: wrapper.response_config.status,
        headers: wrapper.response_config.headers,
        body,
        delay: wrapper.response_config.delay,
    };
    Ok(EntryWrapper {
        matcher: wrapper.matcher.clone(),
        response_config,
    })
}

async fn convert_entry(wrapper: &RequestWrapper) -> Result<EntryWrapperWithBody, Error> {
    let request_matcher = covert_request(&wrapper.request).await?;
    let (response_config, body) = covert_response(&wrapper.response).await?;

    Ok(EntryWrapperWithBody {
        matcher: request_matcher,
        response_config,
        body,
    })
}

async fn covert_response(
    response: &ResponseEntry,
) -> Result<(StaticResponseConfig<()>, Option<String>), Error> {
    let status = response.status;
    let mut header_map: BTreeMap<String, String> = Default::default();
    for header in &response.headers {
        if header.name == "Content-Type" || header.name == "Access-Control-Allow-Origin" {
            header_map.insert(header.name.clone(), header.value.clone());
        }
    }

    let body_text = response.content.text.clone();
    let body = if response.content.mime_type == "application/json" {
        StaticResponseBodyConfig::Json(())
    } else {
        StaticResponseBodyConfig::Raw(())
    };

    Ok((
        StaticResponseConfig {
            status: status as u16,
            weight: 1,
            headers: header_map,
            id: unique_id(),
            body: body_text.clone().map(|_| body),
            delay: 0,
        },
        body_text,
    ))
}

async fn covert_request(request: &RequestEntry) -> Result<StaticMatchesConfig, Error> {
    let mut query_params: Vec<(String, String)> = Default::default();
    for query in &request.query_string {
        query_params.push((query.name.clone(), query.value.clone()));
    }

    let method = request.method.clone();
    let url = Url::parse(&request.url)?;
    let path = url.path();

    Ok(StaticMatchesConfig {
        headers: Default::default(),
        query: query_params,
        path: path.to_string(),
        methods: vec![method],
        graphql: None,
    })
}
