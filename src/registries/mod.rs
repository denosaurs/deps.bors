use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Module;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

pub mod nest;
pub mod x;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryId {
  X,
  Nest,
}

impl fmt::Display for RegistryId {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RegistryId::X => write!(f, "x"),
      RegistryId::Nest => write!(f, "nest"),
    }
  }
}

impl From<&str> for RegistryId {
  fn from(string: &str) -> Self {
    match string {
      "x" => RegistryId::X,
      "nest" => RegistryId::Nest,
      _ => RegistryId::X,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryInfo {
  pub raw_url: String,
}

#[async_trait]
pub trait Registry {
  fn new(client: Client) -> Box<Self>
  where
    Self: Sized;
  fn name(&self) -> &'static str;
  fn id(&self) -> RegistryId;
  fn info(&self) -> RegistryInfo;
  async fn get_modules(&self) -> Result<HashMap<String, Module>>;
}

mod utils {
  pub fn at_version(value: &str) -> Option<&str> {
    let vec: Vec<_> = value.split('@').collect();
    vec.get(1).copied().map(|w| clean_version(w))
  }

  pub fn clean_version(value: &str) -> &str {
    if value.starts_with('v') {
      &value[1..]
    } else {
      &value[0..]
    }
  }
}
