use log::info;
use std::{collections::HashMap, convert::TryInto};

#[cfg(debug_assertions)]
use std::{env, fs};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::index;

use super::{Registry, RegistryId, RegistryInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
  pub name: String,
  pub normalized_name: String,
  pub owner: String,
  pub description: Option<String>,
  pub repository: Option<String>,
  pub latest_version: Option<String>,
  pub latest_stable_version: Option<String>,
  pub package_upload_names: Vec<String>,
  pub locked: Option<bool>,
  pub malicious: Option<bool>,
  pub unlisted: Option<bool>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
}

impl TryInto<index::Module> for Module {
  type Error = anyhow::Error;
  fn try_into(self) -> Result<index::Module> {
    let versions: Result<Vec<semver::Version>, _> = self
      .package_upload_names
      .iter()
      .filter_map(|v| super::utils::at_version(&v))
      .map(|v| semver::Version::parse(&v))
      .collect();
    let versions = versions?;
    Ok(index::Module {
      name: self.name.clone(),
      desc: self.description.clone(),
      repo: self.repository,
      reg: RegistryId::Nest,
      vers: versions,
    })
  }
}

pub struct Nest {
  client: Client,
}

#[async_trait]
impl Registry for Nest {
  fn new(client: Client) -> Box<Self> {
    Box::new(Self { client })
  }
  fn name(&self) -> &'static str {
    "nest"
  }
  fn id(&self) -> RegistryId {
    RegistryId::Nest
  }
  fn info(&self) -> RegistryInfo {
    RegistryInfo {
      raw_url: "https://x.nest.land/{name}@{version}/{...rest}".to_string(),
    }
  }
  async fn get_modules(&self) -> Result<HashMap<String, index::Module>> {
    let response = self.get_all_modules().await?;
    Ok(
      response
        .into_iter()
        .filter_map(|module| match TryInto::<index::Module>::try_into(module) {
          Ok(module) => Some((module.name.clone(), module)),
          Err(_) => None,
        })
        .collect(),
    )
  }
}

impl Nest {
  async fn fetch_all_modules(&self) -> Result<Vec<Module>> {
    info!("collecting all modules");
    let url = Url::parse("https://x.nest.land/api/packages")?;
    Ok(
      self
        .client
        .get(url)
        .send()
        .await?
        .json::<Vec<Module>>()
        .await?,
    )
  }

  #[cfg(not(debug_assertions))]
  async fn get_all_modules(&self) -> Result<Vec<Module>> {
    Ok(self.fetch_all_modules().await?)
  }

  #[cfg(debug_assertions)]
  async fn get_all_modules(&self) -> Result<Vec<Module>> {
    let cwd = env::current_dir()?;
    let cache = cwd.join(".cache");
    let file = cache.join("nest.json");

    fs::create_dir_all(&cache)?;

    let modules = if !file.exists() {
      self.fetch_all_modules().await?
    } else {
      let raw = fs::read_to_string(&file)?;
      serde_json::from_str::<Vec<Module>>(&raw)?
    };

    fs::write(&file, serde_json::to_string(&modules)?)?;

    Ok(modules)
  }
}
