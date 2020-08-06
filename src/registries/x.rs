use std::{collections::HashMap, convert::TryInto};

use anyhow::Result;
use log::info;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::index;

#[cfg(debug_assertions)]
use std::{env, fs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  pub description: Option<String>,
  pub star_count: u32,
  pub version_info: VersionInfo,
  pub version_meta: Option<VersionMetaInfo>,
}

impl Module {
  pub fn from(search: SearchResult, info: VersionInfo) -> Self {
    Self {
      name: search.name,
      description: search.description,
      star_count: search.star_count,
      version_info: info,
      version_meta: None,
    }
  }
  pub fn from_ext(
    search: SearchResult,
    info: VersionInfo,
    meta: VersionMetaInfo,
  ) -> Self {
    Self {
      name: search.name,
      description: search.description,
      star_count: search.star_count,
      version_info: info,
      version_meta: Some(meta),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
  pub name: String,
  pub description: Option<String>,
  pub star_count: u32,
}

impl TryInto<index::Module> for Module {
  type Error = anyhow::Error;
  fn try_into(self) -> Result<index::Module> {
    let versions: Result<Vec<semver::Version>, _> = self
      .version_info
      .versions
      .iter()
      .map(|v| super::clean_version(&v))
      .map(|v| semver::Version::parse(&v))
      .collect();
    let versions = versions?;
    Ok(index::Module {
      name: self.name.clone(),
      desc: self.description.clone(),
      repo: match &self.version_meta {
        Some(meta) => Some(meta.upload_options.repository.clone()),
        None => None,
      },
      reg: index::Registry::X,
      vers: versions,
    })
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
  results: Vec<SearchResult>,
  total_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawResponse<T> {
  success: bool,
  data: T,
}

pub async fn fetch_page(
  client: &Client,
  page: &u32,
  limit: &u16,
  query: Option<&String>,
) -> Result<Page> {
  let base_url = Url::parse("https://api.deno.land/")?;
  let mut url = base_url.join("modules")?;
  {
    let mut query_pairs = url.query_pairs_mut();
    query_pairs.append_pair("page", &page.to_string());
    query_pairs.append_pair("limit", &limit.to_string());

    if let Some(query) = query {
      query_pairs.append_pair("query", &query);
    }
  }

  let response = client
    .get(url)
    .send()
    .await?
    .json::<RawResponse<Page>>()
    .await?;

  Ok(response.data)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
  latest: Option<String>,
  versions: Vec<String>,
}

pub async fn fetch_version_info(
  client: &Client,
  name: &String,
) -> Result<VersionInfo> {
  let base_url = Url::parse("https://cdn.deno.land/")?;
  let url = base_url
    .join(&format!("{}/", name))
    .and_then(|url| url.join("meta/"))
    .and_then(|url| url.join("versions.json"))?;

  let response = client.get(url).send().await?.json::<VersionInfo>().await?;

  Ok(response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirListing {
  path: String,
  #[serde(rename = "type")]
  file_type: String,
  size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadOptions {
  #[serde(rename = "type")]
  upload_tyupe: String,
  repository: String,
  subdir: Option<String>,
  #[serde(rename = "ref")]
  git_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetaInfo {
  uploaded_at: Option<String>,
  directory_listing: Vec<DirListing>,
  upload_options: UploadOptions,
}

pub async fn fetch_version_meta(
  client: &Client,
  name: &String,
  version: &String,
) -> Result<VersionMetaInfo> {
  let base_url = Url::parse("https://cdn.deno.land/")?;
  let url = base_url
    .join(&format!("{}/", name))
    .and_then(|url| url.join("versions/"))
    .and_then(|url| url.join(&format!("{}/", version)))
    .and_then(|url| url.join("meta/"))
    .and_then(|url| url.join("meta.json"))?;

  let response = client
    .get(url)
    .send()
    .await?
    .json::<VersionMetaInfo>()
    .await?;

  Ok(response)
}

async fn fetch_all_modules(client: &Client) -> Result<Vec<Module>> {
  let page_id = 1u32;
  let limit = 100;

  let total = fetch_page(client, &page_id, &limit, None)
    .await?
    .total_count;
  let total_pages = (total as f32 / limit as f32).ceil() as u32;

  info!("found {} total pages", total_pages);
  let mut modules = Vec::new();

  for page in 1..(total_pages + 1) {
    let page = fetch_page(&client, &page, &limit, None).await?;
    for search in page.results {
      info!("hydrating {}", &search.name);
      let info = fetch_version_info(client, &search.name).await;
      if let Ok(info) = info {
        modules.push(match &info.latest {
          Some(v) => match fetch_version_meta(client, &search.name, &v).await {
            Ok(meta) => Module::from_ext(search, info, meta),
            Err(_) => Module::from(search, info),
          },
          None => Module::from(search, info),
        });
      }
    }
  }

  Ok(modules)
}

#[cfg(not(debug_assertions))]
pub async fn get_all_modules(client: &Client) -> Result<Vec<Module>> {
  Ok(fetch_all_modules(client).await?)
}

#[cfg(debug_assertions)]
pub async fn get_all_modules(client: &Client) -> Result<Vec<Module>> {
  let cwd = env::current_dir()?;
  let cache = cwd.join(".cache");
  let file = cache.join("x.json");

  fs::create_dir_all(&cache)?;

  let modules = if !file.exists() {
    fetch_all_modules(client).await?
  } else {
    let raw = fs::read_to_string(&file)?;
    serde_json::from_str::<Vec<Module>>(&raw)?
  };

  fs::write(&file, serde_json::to_string(&modules)?)?;

  Ok(modules)
}

pub async fn get_module_map(
  client: &Client,
) -> Result<HashMap<String, index::Module>> {
  let modules = get_all_modules(client).await?;
  Ok(
    modules
      .into_iter()
      .filter_map(|module| match TryInto::<index::Module>::try_into(module) {
        Ok(module) => Some((module.name.clone(), module)),
        Err(_) => None,
      })
      .collect(),
  )
}
