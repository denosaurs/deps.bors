use std::{collections::HashMap, env, fs};

use anyhow::Result;
use clap::{app_from_crate, crate_name, crate_version};
use index::{Module, Registry};
use log::info;

mod index;
mod nest;
mod nv;
// mod swc;
mod x;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!(),);

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Debug)
    .format_timestamp(None)
    .init();

  let _matches = app_from_crate!().get_matches();

  let cwd = env::current_dir()?;
  let cache = cwd.join(".cache");
  let index = cwd.join("index");
  fs::create_dir_all(&cache)?;
  fs::create_dir_all(&index)?;

  let client = reqwest::Client::builder()
    .user_agent(APP_USER_AGENT)
    .build()?;

  let supported_registries = vec![Registry::X, Registry::Nest];
  let mut registry_map = HashMap::new();

  for registry in supported_registries {
    let module_cache = cache.join(format!("{}.json", registry));
    let modules;
    if module_cache.exists() {
      info!("[{}] reading from cache", registry);
      let raw_modules = fs::read_to_string(&module_cache)?;
      modules = serde_json::from_str::<HashMap<String, Module>>(&raw_modules)?;
    } else {
      info!("[{}] reading from live api", registry);
      modules = index::get_all_modules(&registry, &client).await?;
      let x_modules = serde_json::to_string(&modules)?;
      fs::write(module_cache, x_modules)?;
    }
    registry_map.insert(registry, modules);
  }

  info!("flattening modules...");

  for (registry, modules) in registry_map.iter_mut() {
    let registry_index = index.join(registry.to_string());
    let mut new_modules = Vec::new();
    let mut updated_modules = Vec::new();

    for (name, module) in modules.iter_mut() {
      let path = module.index_path();
      let dist = registry_index.join(&path);

      let raw = serde_json::to_string(&module)?;

      if !dist.exists() {
        new_modules.push(module);
        info!("new module {}", name)
      } else {
        let raw = fs::read_to_string(&dist)?;
        let original = serde_json::from_str::<Module>(&raw)?;
        let original = original.vers;
        module.vers.retain(|v| !original.contains(v));

        if module.vers.len() > 0 {
          updated_modules.push(module);
          info!("updated module {}", name)
        }
      }

      fs::create_dir_all(&dist.parent().unwrap())?;
      fs::write(&dist, raw)?;
    }
  }

  Ok(())
}
