use crate::{index, utils};
use anyhow::Result;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryInto, env, fs};

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
      .filter_map(|v| utils::version(&v))
      .map(|v| semver::Version::parse(&v))
      .collect();
    let versions = versions?;
    Ok(index::Module {
      name: self.name.clone(),
      desc: self.description.clone(),
      repo: self.repository,
      reg: index::Registry::Nest,
      vers: versions,
    })
  }
}

#[cfg(not(debug_assertions))]
pub async fn get_all(client: &Client) -> Result<Vec<Module>> {
  info!("collecting all modules");
  let url = Url::parse("https://x.nest.land/api/packages")?;
  let modules = client.get(url).send().await?.json::<Vec<Module>>().await?;
  Ok(modules)
}

#[cfg(debug_assertions)]
pub async fn get_all(client: &Client) -> Result<Vec<Module>> {
  let cwd = env::current_dir()?;
  let cache = cwd.join(".cache");
  let file = cache.join("nest.json");

  fs::create_dir_all(&cache)?;

  let modules = if !file.exists() {
    let url = Url::parse("https://x.nest.land/api/packages")?;
    client.get(url).send().await?.json::<Vec<Module>>().await?
  } else {
    let raw = fs::read_to_string(&file)?;
    serde_json::from_str::<Vec<Module>>(&raw)?
  };

  fs::write(&file, serde_json::to_string(&modules)?)?;

  Ok(modules)
}

pub async fn get_all_modules(
  client: &Client,
) -> Result<HashMap<String, Module>> {
  let response = get_all(client).await?;
  Ok(
    response
      .into_iter()
      .map(|module| (module.name.clone(), module))
      .collect(),
  )
}
