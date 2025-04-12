use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

mod desktop;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

use desktop::TauriPluginMcpManager;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the tauri-plugin-mcp-manager APIs.
pub trait TauriPluginMcpManagerExt<R: Runtime> {
  fn tauri_plugin_mcp_manager(&self) -> &TauriPluginMcpManager<R>;
}

impl<R: Runtime, T: Manager<R>> crate::TauriPluginMcpManagerExt<R> for T {
  fn tauri_plugin_mcp_manager(&self) -> &TauriPluginMcpManager<R> {
    self.state::<TauriPluginMcpManager<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("tauri-plugin-mcp-manager")
    .invoke_handler(tauri::generate_handler![
        commands::ping,
        commands::start_mcp_server,
        commands::send_to_mcp_server,
        commands::kill_mcp_server
    ])
    .setup(|app, api| {
      let tauri_plugin_mcp_manager = desktop::init(app, api)?;
      app.manage(tauri_plugin_mcp_manager);
      Ok(())
    })
    .build()
}
