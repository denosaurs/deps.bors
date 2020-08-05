use std::convert::Into;

use anyhow::Result;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::index;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
  pub name: String,
  pub description: Option<String>,
  pub star_count: u32,
}

impl Into<index::Module> for Module {
  fn into(self) -> index::Module {
    index::Module {
      name: self.name.clone(),
      description: self.description.clone(),
      tags: Vec::new(),
      x: index::RegistryLink::Sure(self.clone()),
      nest: index::RegistryLink::None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
  results: Vec<Module>,
  total_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawResponse<T> {
  success: bool,
  data: T,
}

pub async fn get_page(
  client: &Client,
  page: u32,
  limit: u16,
  query: Option<String>,
) -> Result<Page> {
  let base_url = Url::parse("https://api.deno.land/")?;
  let mut url = base_url.join("modules")?;
  {
    let mut query_pairs = url.query_pairs_mut();
    query_pairs.append_pair("page", &page.to_string());
    query_pairs.append_pair("limit", &limit.to_string());

    if let Some(query) = query {
      query_pairs.append_pair("query", &query);
    }
  }

  let response = client
    .get(url)
    .send()
    .await?
    .json::<RawResponse<Page>>()
    .await?;

  Ok(response.data)
}

pub async fn get_all_modules(client: &Client) -> Result<Vec<Module>> {
  let page_id = 1u32;
  let limit = 100;

  let total = get_page(client, page_id, limit, None).await?.total_count;
  let total_pages = (total as f32 / limit as f32).ceil() as u32;

  let mut pages = Vec::new();

  for page in 1..(total_pages + 1) {
    pages.push(get_page(&client, page, limit, None).await?)
  }

  Ok(
    pages
      .into_iter()
      .map(|page| page.results)
      .flatten()
      .collect(),
  )
}
