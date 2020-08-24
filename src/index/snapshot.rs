use serde::{Deserialize, Serialize};

use super::Version;
use crate::registries::RegistryId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
  pub name: String,
  pub desc: Option<String>,
  pub repo: Option<String>,
  pub reg: RegistryId,
  pub vers: Version,
}

impl PartialEq for Snapshot {
  fn eq(&self, other: &Self) -> bool {
    self.vers.eq(&other.vers)
  }
}
impl Eq for Snapshot {}
