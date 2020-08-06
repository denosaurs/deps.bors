pub fn version(value: &str) -> Option<&str> {
  let vec: Vec<_> = value.split("@").collect();
  vec.get(1).copied()
}
