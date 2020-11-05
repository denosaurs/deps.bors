use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Eq, Clone)]
pub struct Version {
  inner: semver::Version,
  v: bool,
}

impl Version {
  pub fn parse(v: &str) -> Result<Self, semver::SemVerError> {
    let v = v.trim();
    let base = if v.contains('@') {
      let vec: Vec<_> = v.split('@').collect();
      vec.get(1).copied()
    } else {
      Some(v)
    }
    .ok_or_else(|| {
      semver::SemVerError::ParseError("Error parsing @ version".to_string())
    })?;

    Ok(if base.starts_with('v') {
      Self {
        inner: semver::Version::parse(&base[1..])?,
        v: true,
      }
    } else {
      Self {
        inner: semver::Version::parse(&base[0..])?,
        v: false,
      }
    })
  }
  pub fn semver(&self) -> &semver::Version {
    &self.inner
  }
}

impl fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.v {
      write!(f, "v")?;
    }
    write!(f, "{}", self.inner)
  }
}

impl Serialize for Version {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for Version {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct VersionVisitor;

    // Deserialize Version from a string.
    impl<'de> de::Visitor<'de> for VersionVisitor {
      type Value = Version;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a SemVer version as a string")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Version::parse(v).map_err(de::Error::custom)
      }
    }

    deserializer.deserialize_str(VersionVisitor)
  }
}

impl Ord for Version {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.inner.cmp(&other.inner)
  }
}

impl PartialOrd for Version {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.inner.partial_cmp(&other.inner)
  }
}

impl PartialEq for Version {
  fn eq(&self, other: &Self) -> bool {
    self.inner.eq(&other.inner)
  }
}

#[cfg(test)]
mod tests {
  use super::Version;
  use semver;

  #[test]
  fn version_parse_error() {
    fn parse_error(e: &str) -> Result<Version, semver::SemVerError> {
      return Err(semver::SemVerError::ParseError(e.to_string()));
    }
    assert_eq!(
      Version::parse(""),
      parse_error("expected more input")
    );
  }

  #[test]
  fn version_parse_ok() {
    fn parse_error(e: &str) -> Result<Version, semver::SemVerError> {
      return Err(semver::SemVerError::ParseError(e.to_string()));
    }
    assert_eq!(
      Version::parse(""),
      parse_error("expected more input")
    );
  }

  #[test]
  fn version_serialization() {}
}
