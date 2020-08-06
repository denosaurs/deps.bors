use std::{collections::HashMap, env, fs, io::BufReader};

use anyhow::Result;
use clap::{crate_name, crate_version};
use index::{Registry, Snapshot};
use io::Write;
use log::info;
use reqwest::Client;
use std::fs::OpenOptions;
use std::io::{self, BufRead};

mod index;
mod nest;
mod utils;
mod x;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

async fn run() -> Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .format_timestamp(None)
    .init();

  let index = env::current_dir()?;

  let client = Client::builder().user_agent(APP_USER_AGENT).build()?;

  let registries = vec![Registry::X, Registry::Nest];
  let mut registry_map = HashMap::new();

  for registry in registries {
    info!("gathering modules from {} registry", registry);
    let modules = index::get_all_modules(&registry, &client).await?;
    registry_map.insert(registry, modules);
  }

  for (registry, modules) in &mut registry_map {
    info!("generating index for `{}` registry", registry);
    let registry_index = index.join(registry.to_string());
    let mut new_modules = Vec::new();
    let mut updated_modules = Vec::new();

    for (_, module) in modules {
      let path = module.index_path();
      let dst = registry_index.join(&path);

      let mut snaps = module.get_snapshots();

      if dst.exists() {
        let file = OpenOptions::new().read(true).open(&dst)?;
        let lines = BufReader::new(file).lines();
        for line in lines {
          if let Ok(line) = line {
            let module = serde_json::from_str::<Snapshot>(&line)?;
            snaps.retain(|x| !x.eq(&module))
          }
        }
        if snaps.len() > 0 {
          updated_modules.push(module);
        }
      } else {
        fs::create_dir_all(&dst.parent().unwrap())?;
        new_modules.push(module);
      }

      let mut file = OpenOptions::new().append(true).create(true).open(&dst)?;
      for version in snaps {
        serde_json::to_writer(&mut file, &version)?;
        file.write_all(b"\n")?;
      }
    }

    let new_modules = serde_json::to_string(&new_modules)?;
    fs::write(registry_index.join("new.json"), new_modules)?;
    let updated_modules = serde_json::to_string(&updated_modules)?;
    fs::write(registry_index.join("updated.json"), updated_modules)?;
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  match run().await {
    Err(e) => eprintln!("{:#?}", e),
    _ => {}
  }
}
