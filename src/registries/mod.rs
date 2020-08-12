pub mod nest;
pub mod x;

mod utils {
  pub fn at_version(value: &str) -> Option<&str> {
    let vec: Vec<_> = value.split('@').collect();
    vec.get(1).copied().map(|w| clean_version(w))
  }

  pub fn clean_version(value: &str) -> &str {
    if value.starts_with('v') {
      &value[1..]
    } else {
      &value[0..]
    }
  }
}
