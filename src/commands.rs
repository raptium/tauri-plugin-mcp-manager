use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::McpManagerExt;

#[command]
pub(crate) async fn start_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: StartRequest,
) -> Result<StartResponse> {
    app.mcp_manager().start_mcp_server(payload).await
}

#[command]
pub(crate) async fn send_to_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: SendRequest,
) -> Result<()> {
    app.mcp_manager().send_to_mcp_server(payload).await
}

#[command]
pub(crate) async fn kill_mcp_server<R: Runtime>(
    app: AppHandle<R>,
    payload: KillRequest,
) -> Result<()> {
    app.mcp_manager().kill_mcp_server(payload).await
}
