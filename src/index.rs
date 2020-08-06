use std::{
  collections::HashMap,
  fmt,
  path::{Path, PathBuf},
};

use anyhow::Result;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::registries::{nest, x};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Registry {
  X,
  Nest,
}

impl fmt::Display for Registry {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Registry::X => write!(f, "x"),
      Registry::Nest => write!(f, "nest"),
    }
  }
}

impl From<&str> for Registry {
  fn from(string: &str) -> Self {
    match string {
      "x" => Registry::X,
      "nest" => Registry::Nest,
      _ => Registry::X,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  pub desc: Option<String>,
  pub repo: Option<String>,
  pub reg: Registry,
  pub vers: Vec<Version>,
}

impl Module {
  pub fn index_path(&self) -> PathBuf {
    let name = &self.name;
    match name.len() {
      1 => Path::new("1").join(name),
      2 => Path::new("2").join(name),
      3 => Path::new("3").join(&name[..1]).join(name),
      _ => Path::new(&name[0..2]).join(&name[2..4]).join(name),
    }
  }
  pub fn get_snapshots(&self) -> Vec<Snapshot> {
    let mut vers = self.vers.clone();
    vers.sort();
    vers
      .into_iter()
      .map(|v| Snapshot {
        name: self.name.clone(),
        desc: self.desc.clone(),
        repo: self.repo.clone(),
        reg: self.reg.clone(),
        vers: v,
      })
      .collect()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
  pub name: String,
  pub desc: Option<String>,
  pub repo: Option<String>,
  pub reg: Registry,
  pub vers: Version,
}

impl PartialEq for Snapshot {
  fn eq(&self, other: &Self) -> bool {
    self.vers.eq(&other.vers)
  }
}
impl Eq for Snapshot {}

pub async fn get_all_modules(
  registry: &Registry,
  client: &Client,
) -> Result<HashMap<String, Module>> {
  Ok(match registry {
    Registry::X => x::get_module_map(client).await?,
    Registry::Nest => nest::get_module_map(client).await?,
  })
}
