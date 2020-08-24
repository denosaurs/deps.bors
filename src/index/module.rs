use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{snapshot::Snapshot, Version};
use crate::registries::RegistryId;

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  pub desc: Option<String>,
  pub repo: Option<String>,
  pub reg: RegistryId,
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
