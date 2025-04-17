import { invoke } from "@tauri-apps/api/core";
import { type Event, listen } from "@tauri-apps/api/event";

// --- Type Definitions (mirroring Rust structs) ---

export interface StdioServerParams {
	command: string;
	args: string[];
	env?: Record<string, string>;
}

export interface StartRequest {
	params: StdioServerParams;
}

export interface StartResponse {
	serverId: string; // Internal UUID
}

export interface ServerIdPayload {
	serverId: string;
}

export type KillRequest = ServerIdPayload;

export interface SendRequest {
	serverId: string; // Use serverId instead of name
	data: string;
}

// --- Server Events (mirroring Rust enum ServerEvent) ---

// Represents the payload for the 'stdout' variant of ServerEvent
interface ServerEventLine {
	type: "line";
	payload: string; // Corresponds to String
}

// Represents the payload for the 'stderr' variant of ServerEvent
interface ServerEventStderr {
	type: "stderr";
	payload: number[]; // Corresponds to Vec<u8>
}

// Represents the payload for the 'exit' variant of ServerEvent
interface ServerEventExit {
	type: "exit";
	payload: number | null; // Corresponds to Option<i32>
}

// Discriminated union type for all possible server events
export type ServerEvent = ServerEventLine | ServerEventStderr | ServerEventExit;

// --- Plugin Commands ---

const PLUGIN_NAME = "mcp-manager";

export async function startMcpServer(
	payload: StartRequest,
): Promise<StartResponse> {
	return await invoke(`plugin:${PLUGIN_NAME}|start_mcp_server`, { payload });
}

export async function sendToMcpServer(payload: SendRequest): Promise<void> {
	await invoke(`plugin:${PLUGIN_NAME}|send_to_mcp_server`, { payload });
}

export async function killMcpServer(payload: KillRequest): Promise<void> {
	await invoke(`plugin:${PLUGIN_NAME}|kill_mcp_server`, { payload });
}

// --- Event Listening ---

/**
 * Listens for messages from a specific MCP server process, identified by its internal serverId.
 *
 * @param serverId The unique internal ID of the server (from StartResponse).
 * @param onMessage A callback function to handle incoming messages.
 *                  The payload includes serverId and the message data.
 * @returns A promise that resolves to an unlisten function.
 */
export async function listenToMcpServer(
	serverId: string,
	onMessage: (payload: ServerEvent) => void,
): Promise<() => void> {
	const eventName = `mcp://message/${serverId}`;
	return await listen<ServerEvent>(eventName, (event: Event<ServerEvent>) => {
		onMessage(event.payload);
	});
}
