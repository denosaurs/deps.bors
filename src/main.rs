use std::{collections::HashMap, env, fs};

use anyhow::Result;
use clap::{app_from_crate, crate_name, crate_version};
use log::info;
use tagger::Tag;

mod deps;
mod index;
mod nest;
mod tagger;
mod x;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!(),);

#[tokio::main]
async fn main() -> Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .init();

  let _matches = app_from_crate!().get_matches();

  let cwd = env::current_dir()?;
  let cache = cwd.join(".cache");
  let index = cwd.join("index");
  fs::create_dir_all(&cache)?;
  fs::create_dir_all(&index)?;

  let x_cache = cache.join("x.json");
  let nest_cache = cache.join("nest.json");

  let client = reqwest::Client::builder()
    .user_agent(APP_USER_AGENT)
    .build()?;

  let mut x_modules;
  if x_cache.exists() {
    info!("reading x_modules from cache");
    let raw_modules = fs::read_to_string(&x_cache)?;
    x_modules = serde_json::from_str::<Vec<x::Module>>(&raw_modules)?;
  } else {
    info!("reading x_modules from live api");
    x_modules = x::get_all_modules(&client).await?;
    let x_modules = serde_json::to_string(&x_modules)?;
    fs::write(x_cache, x_modules)?;
  }

  let nest_modules;
  if nest_cache.exists() {
    info!("reading nest_modules from cache");
    let raw_modules = fs::read_to_string(&nest_cache)?;
    nest_modules = serde_json::from_str::<Vec<nest::Module>>(&raw_modules)?;
  } else {
    info!("reading nest_modules from live api");
    nest_modules = nest::get_all_modules(&client).await?;
    let nest_modules = serde_json::to_string(&nest_modules)?;
    fs::write(nest_cache, nest_modules)?;
  }

  let mut nest_hash: HashMap<_, _> = nest_modules
    .iter()
    .cloned()
    .map(|module| (module.name.clone(), module))
    .collect();

  // is there a way to optimise all this boilerplate ^ ?
  // also i am using it just for testing...

  let mut new = Vec::new();

  info!("generating index for {} records", x_modules.len());
  for module in x_modules {
    let path = index::relative_index_file(&module.name);

    let mut module: index::Module = module.into();

    let nest = nest_hash.remove(&module.name);
    if let Some(nest) = nest {
      module.nest = index::RegistryLink::Possible(nest);
    }

    let module = serde_json::to_string(&module)?;
    let dist = index.join(&path);

    if dist.exists() {
      new.push(module.clone())
    }

    fs::create_dir_all(&dist.parent().unwrap())?;
    fs::write(&dist, module)?;
  }

  for module in nest_hash.into_iter().map(|(_, module)| module) {
    let path = index::relative_index_file(&module.name);

    let module: index::Module = module.into();

    let module = serde_json::to_string(&module)?;
    let dist = index.join(&path);

    if dist.exists() {
      new.push(module.clone())
    }

    fs::create_dir_all(&dist.parent().unwrap())?;
    fs::write(&dist, module)?;
  }

  Ok(())
}
