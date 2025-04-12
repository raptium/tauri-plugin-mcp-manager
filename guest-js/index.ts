import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'

// --- Type Definitions (mirroring Rust structs) ---

export interface PingRequest {
  value?: string;
}

export interface PingResponse {
  value?: string;
}

export interface StdioServerParams {
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export interface StartRequest {
  name: string;
  params: StdioServerParams;
}

export interface StartResponse {
  serverId: string; // Internal UUID
  name: string;
}

export interface ServerNamePayload {
  name: string;
}

export type KillRequest = ServerNamePayload;

export interface SendRequest {
  name: string;
  data: string;
}

export interface ServerMessagePayload {
  serverId: string; // The unique internal ID
  name: string; // The user-provided name
  data: string;
}

// --- Plugin Commands ---

const PLUGIN_NAME = "tauri-plugin-mcp-manager";

export async function ping(payload: PingRequest): Promise<PingResponse> {
  return await invoke(`${PLUGIN_NAME}:ping`, { payload });
}

export async function startMcpServer(
  payload: StartRequest
): Promise<StartResponse> {
  return await invoke(`${PLUGIN_NAME}:start_mcp_server`, { payload });
}

export async function sendToMcpServer(payload: SendRequest): Promise<void> {
  await invoke(`${PLUGIN_NAME}:send_to_mcp_server`, { payload });
}

export async function killMcpServer(payload: KillRequest): Promise<void> {
  await invoke(`${PLUGIN_NAME}:kill_mcp_server`, { payload });
}

// --- Event Listening ---

/**
 * Listens for messages from a specific MCP server process, identified by its internal serverId.
 *
 * @param serverId The unique internal ID of the server (from StartResponse).
 * @param onMessage A callback function to handle incoming messages.
 *                  The payload includes both serverId and the user-provided name.
 * @returns A promise that resolves to an unlisten function.
 */
export async function listenToMcpServer(
  serverId: string,
  onMessage: (payload: ServerMessagePayload) => void
): Promise<() => void> {
  const eventName = `mcp://message/${serverId}`;
  return await listen<ServerMessagePayload>(eventName, (event: Event<ServerMessagePayload>) => {
    onMessage(event.payload);
  });
}
