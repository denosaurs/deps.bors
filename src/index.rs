use std::{
  collections::HashMap,
  fmt,
  path::{Path, PathBuf},
};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{nest, x};

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
  pub vers: String,
}

impl Version {
  pub fn stdv(&self) -> &str {
    if self.vers.starts_with('v') {
      &self.vers[1..]
    } else {
      &self.vers
    }
  }
}

impl PartialEq for Version {
  fn eq(&self, other: &Self) -> bool {
    self.stdv() == other.stdv()
  }
}
impl Eq for Version {}

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
}

pub async fn get_all_modules(
  registry: &Registry,
  client: &Client,
) -> Result<HashMap<String, Module>> {
  Ok(match registry {
    Registry::X => x::get_all_modules(client)
      .await?
      .into_iter()
      .map(|(name, module)| (name, module.into()))
      .collect(),
    Registry::Nest => nest::get_all_modules(client)
      .await?
      .into_iter()
      .map(|(name, module)| (name, module.into()))
      .collect(),
  })
}
