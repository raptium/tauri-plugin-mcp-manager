use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::TauriPluginMcpManagerExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.tauri_plugin_mcp_manager().ping(payload)
}

#[command]
pub(crate) async fn start_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: StartRequest,
) -> Result<StartResponse> {
    app.tauri_plugin_mcp_manager().start_mcp_server(payload).await
}

#[command]
pub(crate) async fn send_to_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: SendRequest,
) -> Result<()> {
    app.tauri_plugin_mcp_manager().send_to_mcp_server(payload).await
}

#[command]
pub(crate) async fn kill_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: KillRequest,
) -> Result<()> {
    app.tauri_plugin_mcp_manager().kill_mcp_server(payload).await
}
