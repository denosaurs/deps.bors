use reqwest::Client;

use crate::registries::{nest, x, Registry};

mod module;
mod snapshot;
mod version;

pub use module::Module;
pub use snapshot::Snapshot;
pub use version::Version;

pub fn registries(client: &Client) -> Vec<Box<dyn Registry>> {
  vec![x::X::new(client.clone()), nest::Nest::new(client.clone())]
}
