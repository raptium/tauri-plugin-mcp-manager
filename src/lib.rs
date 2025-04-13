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

use desktop::McpManager;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mcp-manager APIs.
pub trait McpManagerExt<R: Runtime> {
    fn mcp_manager(&self) -> &McpManager<R>;
}

impl<R: Runtime, T: Manager<R>> crate::McpManagerExt<R> for T {
    fn mcp_manager(&self) -> &McpManager<R> {
        self.state::<McpManager<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mcp-manager")
        .invoke_handler(tauri::generate_handler![
            commands::start_mcp_server,
            commands::send_to_mcp_server,
            commands::kill_mcp_server
        ])
        .setup(|app, api| {
            let mcp_manager = desktop::init(app, api)?;
            app.manage(mcp_manager);
            Ok(())
        })
        .build()
}
