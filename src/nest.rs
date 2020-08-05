use crate::index;
use anyhow::Result;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  #[serde(rename = "normalizedName")]
  pub normalized_name: String,
  pub owner: String,
  pub description: Option<String>,
  pub repository: Option<String>,
  #[serde(rename = "latestVersion")]
  pub latest_version: Option<String>,
  #[serde(rename = "latestStableVersion")]
  pub latest_stable_version: Option<String>,
  #[serde(rename = "packageUploadNames")]
  pub package_upload_names: Vec<String>,
  pub locked: Option<bool>,
  pub malicious: Option<bool>,
  pub unlisted: Option<bool>,
  #[serde(rename = "createdAt")]
  pub created_at: Option<String>,
  #[serde(rename = "updatedAt")]
  pub updated_at: Option<String>,
}

impl Into<index::Module> for Module {
  fn into(self) -> index::Module {
    index::Module {
      name: self.name.clone(),
      description: self.description.clone(),
      tags: Vec::new(),
      x: index::RegistryLink::None,
      nest: index::RegistryLink::Sure(self.clone()),
    }
  }
}

pub async fn get_all_modules(client: &Client) -> Result<Vec<Module>> {
  let url = Url::parse("https://x.nest.land/api/packages")?;
  let response = client.get(url).send().await?.json::<Vec<Module>>().await?;
  Ok(response)
}
