use std::fs::OpenOptions;
use std::{env, fs, io::BufReader};
use std::{
  io::{self, BufRead},
  time::Instant,
};

use anyhow::Result;
use clap::{crate_name, crate_version};
use index::Snapshot;
use io::Write;
use log::info;
use reqwest::Client;

use deps::index;

static APP_USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());

async fn run() -> Result<()> {
  let now = Instant::now();

  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .format_timestamp(None)
    .init();

  let index = env::current_dir()?;

  let client = Client::builder().user_agent(APP_USER_AGENT).build()?;

  let registries = index::registries(&client);

  for registry in registries {
    info!("gathering modules from {} registry", registry.name());
    let mut modules = registry.get_modules().await?;

    info!("generating index for `{}` registry", registry.name());
    let registry_index = index.join(registry.id().to_string());
    let mut new_modules = Vec::new();
    let mut updated_modules = Vec::new();

    for module in modules.values_mut() {
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
        if !snaps.is_empty() {
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

    let serialized = serde_json::to_string(&new_modules)?;
    fs::write(registry_index.join("new.json"), serialized)?;
    let serialized = serde_json::to_string(&updated_modules)?;
    fs::write(registry_index.join("updated.json"), serialized)?;
    let serialized = serde_json::to_string(&registry.info())?;
    fs::write(registry_index.join("registry.json"), serialized)?;
  }

  info!("done, in {}s", now.elapsed().as_secs());

  Ok(())
}

#[tokio::main]
async fn main() {
  if let Err(e) = run().await {
    eprintln!("{:#?}", e)
  }
}
