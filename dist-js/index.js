import { JSONRPCMessageSchema } from '@modelcontextprotocol/sdk/types.js';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// --- Plugin Commands ---
const PLUGIN_NAME = "mcp-manager";
async function startMcpServer(payload) {
    return await invoke(`plugin:${PLUGIN_NAME}|start_mcp_server`, { payload });
}
async function sendToMcpServer(payload) {
    await invoke(`plugin:${PLUGIN_NAME}|send_to_mcp_server`, { payload });
}
async function killMcpServer(payload) {
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
async function listenToMcpServer(serverId, onMessage) {
    const eventName = `mcp://message/${serverId}`;
    return await listen(eventName, (event) => {
        onMessage(event.payload);
    });
}

function deserializeMessage(line) {
    return JSONRPCMessageSchema.parse(JSON.parse(line));
}
function serializeMessage(message) {
    return `${JSON.stringify(message)}\n`;
}
class TauriStdioTransport {
    constructor(server) {
        this._serverParams = server;
    }
    async start() {
        if (this._serverId) {
            throw new Error("Transport already started");
        }
        const response = await startMcpServer({
            params: {
                command: this._serverParams.command,
                args: this._serverParams.args || [],
                env: this._serverParams.env || {},
            },
        });
        this._serverId = response.serverId;
        const decoder = new TextDecoder();
        // Listen for server events (stdout, stderr, exit)
        listenToMcpServer(this._serverId, (event) => {
            console.info("Received event", event);
            switch (event.type) {
                case "line":
                    if (event.payload) {
                        const message = deserializeMessage(event.payload);
                        this.onmessage?.(message);
                    }
                    break;
                case "stderr":
                    // Assuming UTF-8 encoding for the Vec<u8> payload
                    console.error(`[${this._serverId}] STDERR:`, decoder.decode(new Uint8Array(event.payload)));
                    break;
                case "exit":
                    console.info(`[${this._serverId}] Process exited with code:`, event.payload);
                    this._serverId = undefined;
                    // Call the close handler if it exists
                    this.onclose?.();
                    break;
            }
        });
    }
    async close() {
        if (!this._serverId) {
            throw new Error("Transport not started");
        }
        try {
            await killMcpServer({
                serverId: this._serverId,
            });
        }
        catch (error) {
            console.error("Error killing MCP server", error);
        }
        this._serverId = undefined;
    }
    async send(message) {
        if (!this._serverId) {
            throw new Error("Transport not started");
        }
        await sendToMcpServer({
            serverId: this._serverId,
            data: serializeMessage(message),
        });
    }
}

export { TauriStdioTransport };
