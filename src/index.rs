use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::nest;
use crate::x;

#[derive(Debug, Serialize, Deserialize)]
pub enum RegistryLink<T> {
  Sure(T),
  Possible(T),
  None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  pub description: Option<String>,
  pub tags: Vec<String>,
  pub x: RegistryLink<x::Module>,
  pub nest: RegistryLink<nest::Module>,
}

pub fn relative_index_file(name: &str) -> PathBuf {
  let name = name.to_lowercase();
  match name.len() {
    1 => Path::new("1").join(&name),
    2 => Path::new("2").join(&name),
    3 => Path::new("3").join(&name[..1]).join(&name),
    _ => Path::new(&name[0..2]).join(&name[2..4]).join(&name),
  }
}
