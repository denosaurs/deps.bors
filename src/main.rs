use std::{collections::HashMap, env, fs};

use anyhow::Result;
use clap::{crate_name, crate_version};
use index::{Module, Registry};
use log::info;
use reqwest::Client;

mod index;
mod nest;
mod nv;
mod x;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());


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

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .format_timestamp(None)
    .init();

  let index = env::current_dir()?;

  let client = Client::builder()
    .user_agent(APP_USER_AGENT)
    .build()?;

  let registries = vec![Registry::X, Registry::Nest];
  let mut registry_map = HashMap::new();

  info!("gathering modules from {} registries", registries.len());
  for registry in registries {
    let modules = index::get_all_modules(&registry, &client).await?;
    registry_map.insert(registry, modules);
  }

  for (registry, modules) in registry_map.iter_mut() {
    info!("generating index for `{}` registry", registry);
    let registry_index = index.join(registry.to_string());
    let mut new_modules = Vec::new();
    let mut updated_modules = Vec::new();

    for (_, new) in modules.iter_mut() {
      let path = new.index_path();
      let dist = registry_index.join(&path);

      let raw = serde_json::to_string(&new)?;

      if !dist.exists() {
        new_modules.push(new);
      } else {
        let raw = fs::read_to_string(&dist)?;
        let old = serde_json::from_str::<Module>(&raw)?;
        new.vers.retain(|v| !old.vers.contains(v));

        if new.vers.len() > 0 {
          updated_modules.push(new);
        }
      }

      fs::create_dir_all(&dist.parent().unwrap())?;
      fs::write(&dist, raw)?;
    }

    let new_modules = serde_json::to_string(&new_modules)?;
    fs::write(registry_index.join("new.json"), new_modules)?;
    let updated_modules = serde_json::to_string(&updated_modules)?;
    fs::write(registry_index.join("updated.json"), updated_modules)?;
  }

  Ok(())
}
