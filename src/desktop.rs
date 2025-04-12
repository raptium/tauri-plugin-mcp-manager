use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Emitter, Runtime};
use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncReadExt},
    process::{Child, ChildStdin, Command},
    sync::Mutex,
};
use uuid::Uuid;

use crate::models::*;
use std::{collections::HashMap, process::Stdio, sync::Arc};
// Add sysinfo imports
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

// Helper struct stores only stdin handle and PID
#[derive(Debug, Clone)]
struct ManagedProcess {
    pid: u32,
    stdin: Arc<Mutex<ChildStdin>>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<McpManager<R>> {
    Ok(McpManager {
        app_handle: app.clone(),
        // Update the HashMap value type
        servers: Arc::new(Mutex::new(HashMap::<String, ManagedProcess>::new())),
    })
}

/// Access to the tauri-plugin-mcp-manager APIs.
#[derive(Debug)]
pub struct McpManager<R: Runtime> {
    app_handle: AppHandle<R>,
    // Update the HashMap value type
    servers: Arc<Mutex<HashMap<String, ManagedProcess>>>, // Holds PID + stdin
}

impl<R: Runtime> McpManager<R> {
    pub async fn start_mcp_server(&self, payload: StartRequest) -> crate::Result<StartResponse> {
        let server_id = Uuid::new_v4().to_string();
        let mut cmd = Command::new(&payload.params.command);
        cmd.args(&payload.params.args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        if let Some(env_vars) = &payload.params.env {
            cmd.envs(env_vars);
        }

        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let mut child = cmd.spawn().map_err(|e| crate::Error::Command(e))?;

        let pid = child.id().ok_or(crate::Error::ProcessIdUnavailable)?;
        let stdout = child.stdout.take().ok_or(crate::Error::Pipe)?; // Take stdout
        let stderr = child.stderr.take().ok_or(crate::Error::Pipe)?; // Take stderr
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or(crate::Error::Pipe)?)); // Take and wrap stdin

        // Store only PID and stdin handle in the map
        {
            let mut servers_guard = self.servers.lock().await;
            let managed_process = ManagedProcess {
                pid,
                stdin: stdin.clone(), // Clone Arc for map
            };
            servers_guard.insert(server_id.clone(), managed_process);
        }

        // Spawn IO listeners
        let app_handle_stdout = self.app_handle.clone();
        let server_id_stdout = server_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            Self::listen_to_stream(app_handle_stdout, server_id_stdout, reader, "stdout").await;
        });

        let app_handle_stderr = self.app_handle.clone();
        let server_id_stderr = server_id.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            Self::listen_to_stream(app_handle_stderr, server_id_stderr, reader, "stderr").await;
        });

        // Spawn task to own the Child and wait for exit
        let app_handle_exit = self.app_handle.clone();
        let server_id_exit = server_id.clone();
        let servers_clone_for_cleanup = self.servers.clone(); // Clone Arc<Mutex<Map>> for cleanup

        tokio::spawn(async move {
            // `child` is moved into this task
            match child.wait().await { // Wait for the process to exit
                Ok(status) => {
                    println!("[{}] Process exited naturally with status: {}", server_id_exit, status);
                    let event_name = format!("mcp://message/{}", server_id_exit);
                    // Emit exit event
                    app_handle_exit.emit(&event_name, ServerEvent::Exit(status.code()))
                        .map_err(|e| eprintln!("[{}] Failed to emit exit event: {}", server_id_exit, e))
                        .ok();
                }
                Err(e) => {
                    eprintln!("[{}] Failed to wait for process exit: {}", server_id_exit, e);
                    let event_name = format!("mcp://message/{}", server_id_exit);
                    // Emit exit event with None status code on wait error
                    app_handle_exit.emit(&event_name, ServerEvent::Exit(None))
                        .map_err(|e_emit| eprintln!("[{}] Failed to emit error exit event: {}", server_id_exit, e_emit))
                        .ok();
                }
            }

            // Remove the server entry AFTER the process has exited.
            // This prevents sending to a process that's terminating but hasn't fully exited.
            {
                let mut servers_guard = servers_clone_for_cleanup.lock().await;
                if servers_guard.remove(&server_id_exit).is_some() {
                    println!("[{}] Removed server entry after process exit.", server_id_exit);
                } else {
                    // This could happen if kill_mcp_server was called and removed the entry first.
                    println!("[{}] Server entry already removed before wait task cleanup (likely via kill).", server_id_exit);
                }
            }
        });

        println!("Started server (PID: {}, ID: {}).", pid, server_id);

        Ok(StartResponse { server_id })
    }

    // Reads from the stream in chunks and emits events eagerly.
    async fn listen_to_stream<T: tokio::io::AsyncRead + Unpin>(
        app_handle: AppHandle<R>,
        server_id: String,
        mut stream: T,
        stream_name: &str,
    ) {
        let mut buf = [0; 1024]; // Read buffer

        loop {
            match stream.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data_chunk_bytes = &buf[0..n]; // Reference the raw byte slice
                    // Log raw bytes using debug format
                    println!("[{}:{}] {:?}", server_id, stream_name, data_chunk_bytes);
                    let event_name = format!("mcp://message/{}", server_id);
                    let event_payload = match stream_name {
                        // Clone the byte slice into a Vec<u8>
                        "stdout" => ServerEvent::Stdout(data_chunk_bytes.to_vec()),
                        "stderr" => ServerEvent::Stderr(data_chunk_bytes.to_vec()),
                        _ => unreachable!(),
                    };
                    app_handle.emit(&event_name, event_payload)
                        .map_err(|e| eprintln!("[{}] Failed to emit {} event: {}", server_id, stream_name, e))
                        .ok();
                }
                Err(e) => {
                    eprintln!("[{}] Error reading from {}: {}", server_id, stream_name, e);
                    break; // Stop reading on error
                }
            }
        }
        println!("[{}:{}] Stream closed.", server_id, stream_name);
        // Cleanup is handled by the separate wait task.
    }

    pub async fn send_to_mcp_server(&self, payload: SendRequest) -> crate::Result<()> {
        let stdin_arc = { // Scoped lock to get stdin Arc
            let servers_guard = self.servers.lock().await;
            servers_guard.get(&payload.server_id).map(|p| p.stdin.clone()) // Clone stdin Arc if found
        };

        if let Some(stdin_arc) = stdin_arc {
            let mut stdin_guard = stdin_arc.lock().await; // Lock stdin
            let mut data = payload.data;
            if !data.ends_with('\n') {
                data.push('\n'); // Ensure newline for stdin
            }
            // Write to stdin
            match stdin_guard.write_all(data.as_bytes()).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    // If write fails, the process likely exited.
                    // Map BrokenPipe or other relevant IO errors to ServerNotFound.
                    if e.kind() == std::io::ErrorKind::BrokenPipe {
                        println!("[{}] Send failed: Pipe broken (process likely exited).", payload.server_id);
                        Err(crate::Error::ServerNotFound(payload.server_id))
                    } else {
                        eprintln!("[{}] Error writing to stdin: {}", payload.server_id, e);
                        Err(crate::Error::Io(e))
                    }
                }
            }
        } else {
            // Not found in map, could be it exited and was cleaned up, or never existed.
            println!("[{}] Send failed: Server not found in map.", payload.server_id);
            Err(crate::Error::ServerNotFound(payload.server_id))
        }
    }

    pub async fn kill_mcp_server(&self, payload: KillRequest) -> crate::Result<()> {
        let managed_process = {
            let mut servers_guard = self.servers.lock().await; // Lock map
            // Remove the entry to prevent further sends and get PID
            servers_guard.remove(&payload.server_id)
        };

        if let Some(process_info) = managed_process {
            let pid_to_kill = Pid::from_u32(process_info.pid);
            println!("[{}] Attempting to kill process with PID: {}", payload.server_id, pid_to_kill);

            // Use sysinfo to kill the process by PID
            let mut sys = System::new();
            // Refresh specific process info for potentially better performance than refresh_all()
            // If this fails (e.g., process already gone), kill will likely fail too.
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid_to_kill]), true);

            let killed = match sys.process(pid_to_kill) {
                Some(process) => {
                    if process.kill() {
                         println!("[{}] Kill signal sent successfully to PID: {}. Wait task will handle exit event.", payload.server_id, pid_to_kill);
                         true
                    } else {
                         eprintln!("[{}] Failed to send kill signal to PID: {}. Process might have exited already.", payload.server_id, pid_to_kill);
                         false // Kill signal failed
                    }
                }
                None => {
                    eprintln!("[{}] Process with PID: {} not found by sysinfo. Already exited?", payload.server_id, pid_to_kill);
                    false // Process not found
                }
            };

            if killed {
                Ok(())
            } else {
                // Even if kill failed, the process might be gone. The wait task handles cleanup.
                // Return an error to indicate the kill command itself wasn't definitively successful.
                // We could potentially return Ok here if process not found, depends on desired semantics.
                Err(crate::Error::KillSignalFailed(payload.server_id.clone()))
            }

        } else {
            // Process not found in map, likely already exited naturally and cleaned up by wait task.
             println!("[{}] Kill failed: Server not found in map (already exited/removed?).", payload.server_id);
            Err(crate::Error::ServerNotFound(payload.server_id))
        }
    }
}
