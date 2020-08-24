use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepsError {
  #[error("invalid data provided")]
  Invalid,

  #[error(transparent)]
  Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}
