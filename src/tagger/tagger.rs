extern crate naivebayes;

use naivebayes::NaiveBayes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Tag {
  pub label: String,
  pub words: Vec<String>,
}

pub struct Tagger {
  nb: NaiveBayes,
}

impl Tagger {
  pub fn new() -> Tagger {
    Tagger {
      nb: NaiveBayes::new(),
    }
  }

  pub fn add_tag(&mut self, tag: Tag) {
    self.nb.train(&tag.words, &tag.label);
  }

  pub fn add_tags(&mut self, tags: Vec<Tag>) {
    for tag in tags.iter() {
      self.add_tag(tag.clone());
    }
  }

  pub fn classify(&mut self, text: &str, top_n: usize) -> Vec<String> {
    let labels = self
      .nb
      .log_classify(&text.split(' ').map(|s| s.to_string()).collect());

    let mut all: Vec<(&String, &f64)> = labels.iter().collect();
    all.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let all_tags: Vec<String> =
      all.iter().map(|v| v.0).map(|s| s.to_string()).collect();

    all_tags[0..top_n].to_vec()
  }
}
