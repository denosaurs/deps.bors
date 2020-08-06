use crate::{index, nv};
use anyhow::Result;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl Into<index::Module> for Module {
  fn into(self) -> index::Module {
    index::Module {
      name: self.name,
      desc: self.description,
      repo: self.repository,
      reg: index::Registry::Nest,
      vers: self
        .package_upload_names
        .iter()
        .filter_map(|upload_name| nv::version(upload_name.as_str()))
        .map(|version| index::Version {
          vers: version.to_string(),
        })
        .collect(),
    }
  }
}

pub async fn get_all_modules(
  client: &Client,
) -> Result<HashMap<String, Module>> {
  let url = Url::parse("https://x.nest.land/api/packages")?;
  let response = client.get(url).send().await?.json::<Vec<Module>>().await?;
  Ok(
    response
      .into_iter()
      .map(|module| (module.name.clone(), module))
      .collect(),
  )
}
