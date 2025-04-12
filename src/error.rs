use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Command Error: {0}")]
  Command(std::io::Error),
  #[error("Could not get stdio pipe")]
  Pipe,
  #[error("Could not obtain process ID after spawning")]
  ProcessIdUnavailable,
  #[error("Server with ID '{0}' not found")]
  ServerNotFound(String),
  #[error("Server with name '{0}' already exists")]
  ServerNameExists(String),
  #[error("Failed to send kill signal to server '{0}'")]
  KillSignalFailed(String),
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
