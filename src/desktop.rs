use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Emitter, Runtime};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::Mutex,
};
use uuid::Uuid;

use crate::models::*;
use std::{collections::HashMap, process::Stdio, sync::Arc};

// Helper struct to store process information
#[derive(Debug)]
struct ManagedProcess {
    child: Child,
    stdin: ChildStdin,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<TauriPluginMcpManager<R>> {
    Ok(TauriPluginMcpManager {
        app_handle: app.clone(),
        // Map key: server_id (UUID string), Map value: (name, ManagedProcess)
        servers: Arc::new(Mutex::new(HashMap::new())),
    })
}

/// Access to the tauri-plugin-mcp-manager APIs.
#[derive(Debug)]
pub struct TauriPluginMcpManager<R: Runtime> {
    app_handle: AppHandle<R>,
    servers: Arc<Mutex<HashMap<String, (String, ManagedProcess)>>>,
}

impl<R: Runtime> TauriPluginMcpManager<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    // Renamed from spawn_mcp_server
    pub async fn start_mcp_server(&self, payload: StartRequest) -> crate::Result<StartResponse> {
        // Check if name already exists
        {
            let servers_guard = self.servers.lock().await;
            if servers_guard.values().any(|(name, _)| name == &payload.name) {
                return Err(crate::Error::ServerNameExists(payload.name.clone())); // Clone name for error
            }
        }

        let server_id = Uuid::new_v4().to_string();
        // Access command, args, env via payload.params
        let mut cmd = Command::new(&payload.params.command);
        cmd.args(&payload.params.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        if let Some(env_vars) = &payload.params.env {
            cmd.envs(env_vars);
        }

        #[cfg(windows)]
        cmd.creation_flags(0x08000000);

        let mut child = cmd.spawn().map_err(|e| crate::Error::Command(e))?;

        let stdout = child.stdout.take().ok_or(crate::Error::Pipe)?;
        let stderr = child.stderr.take().ok_or(crate::Error::Pipe)?;
        let stdin = child.stdin.take().ok_or(crate::Error::Pipe)?;

        let managed_process = ManagedProcess {
            child,
            stdin,
        };

        let server_name = payload.name.clone();

        {
            let mut servers_guard = self.servers.lock().await;
            servers_guard.insert(server_id.clone(), (server_name.clone(), managed_process));
        }

        // Spawn background tasks
        let app_handle_stdout = self.app_handle.clone();
        let server_id_stdout = server_id.clone();
        let server_name_stdout = server_name.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            Self::listen_to_stream(
                app_handle_stdout,
                server_id_stdout,
                server_name_stdout, // Pass name
                reader,
                "stdout",
            )
            .await;
        });

        let app_handle_stderr = self.app_handle.clone();
        let server_id_stderr = server_id.clone();
        let server_name_stderr = server_name.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            Self::listen_to_stream(
                app_handle_stderr,
                server_id_stderr,
                server_name_stderr, // Pass name
                reader,
                "stderr",
            )
            .await;
        });

        Ok(StartResponse { server_id, name: server_name })
    }

    async fn listen_to_stream<T: tokio::io::AsyncRead + Unpin>(
        app_handle: AppHandle<R>,
        server_id: String,
        name: String, // Accept name
        stream: T,
        stream_name: &str,
    ) {
        let mut reader = BufReader::new(stream).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            println!("[{}:{}] {}", name, stream_name, line); // Log with name
            let event_name = format!("mcp://message/{}", server_id); // Event name still uses unique ID
            app_handle
                .emit(
                    &event_name,
                    ServerMessagePayload {
                        server_id: server_id.clone(),
                        name: name.clone(), // Include name in payload
                        data: line,
                    },
                )
                .map_err(|e| eprintln!("Failed to emit event: {}", e))
                .ok();
        }
        println!("[{}:{}] Stream closed.", name, stream_name);
        // Clean up the server entry when a stream closes? Maybe on kill command?
    }

    // Updated to find by name
    pub async fn send_to_mcp_server(&self, payload: SendRequest) -> crate::Result<()> {
        let mut servers_guard = self.servers.lock().await;
        // Find the server_id and stdin by name
        let server_info = servers_guard
            .iter_mut()
            .find(|(_, (name, _))| name == &payload.name);

        if let Some((_server_id, (_name, process))) = server_info {
            let mut data = payload.data;
            if !data.ends_with('\n') {
                data.push('\n');
            }
            process
                .stdin
                .write_all(data.as_bytes())
                .await
                .map_err(crate::Error::Io)?;
            Ok(())
        } else {
            Err(crate::Error::ServerNotFound(payload.name)) // Error uses name
        }
    }

    // Updated to find by name
    pub async fn kill_mcp_server(&self, payload: KillRequest) -> crate::Result<()> {
        let mut servers_guard = self.servers.lock().await;
        let server_id_to_remove = servers_guard
            .iter()
            .find(|(_, (name, _))| name == &payload.name)
            .map(|(id, _)| id.clone());

        if let Some(server_id) = server_id_to_remove {
            if let Some((name, mut managed_process)) = servers_guard.remove(&server_id) {
                managed_process
                    .child
                    .kill()
                    .await
                    .map_err(crate::Error::Io)?;
                println!("Killed server {} (ID: {})", name, server_id);
                Ok(())
            } else {
                // Should technically not happen if ID was found just before
                Err(crate::Error::ServerNotFound(payload.name))
            }
        } else {
            Err(crate::Error::ServerNotFound(payload.name)) // Error uses name
        }
    }
}
