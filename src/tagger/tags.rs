use std::fs;

use anyhow::Result;

use super::Tag;

pub fn load_tags() -> Result<Vec<Tag>> {
  let tags = fs::read_to_string("tags.json")?;
  let tags = serde_json::from_str::<Vec<Tag>>(&tags)?;
  Ok(tags)
}
