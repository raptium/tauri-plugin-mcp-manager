use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Command Error: {0}")]
  Command(std::io::Error),
  #[error("Could not get stdio pipe")]
  Pipe,
  #[error("Server with ID '{0}' not found")]
  ServerNotFound(String),
  #[error("Server with name '{0}' already exists")]
  ServerNameExists(String),
  #[error(transparent)]
  Io(#[from] std::io::Error),
  // Remove mobile conditional compilation for PluginInvokeError
  // #[cfg(mobile)]
  // #[error(transparent)]
  // PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
