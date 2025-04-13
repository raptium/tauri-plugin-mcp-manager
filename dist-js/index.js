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

class ReadBuffer {
    constructor() {
        this._decoder = new TextDecoder("utf8"); // Use TextDecoder for browser compatibility
    }
    append(chunk) {
        if (!this._buffer) {
            this._buffer = chunk;
        }
        else {
            const newBuffer = new Uint8Array(this._buffer.length + chunk.length);
            newBuffer.set(this._buffer, 0);
            newBuffer.set(chunk, this._buffer.length);
            this._buffer = newBuffer;
        }
    }
    readMessage() {
        if (!this._buffer) {
            return null;
        }
        // Find the index of the newline character (byte value 10)
        const index = this._buffer.indexOf(10);
        if (index === -1) {
            return null;
        }
        // Decode the relevant part of the buffer to a string
        const lineBytes = this._buffer.subarray(0, index);
        // Remove trailing \r if present (byte value 13) before decoding
        const line = this._decoder.decode(lineBytes[lineBytes.length - 1] === 13 ? lineBytes.subarray(0, -1) : lineBytes);
        this._buffer = this._buffer.subarray(index + 1);
        // Handle empty buffer case after subarray
        if (this._buffer.length === 0) {
            this._buffer = undefined;
        }
        return deserializeMessage(line);
    }
    clear() {
        this._buffer = undefined;
    }
}
function deserializeMessage(line) {
    return JSONRPCMessageSchema.parse(JSON.parse(line));
}
function serializeMessage(message) {
    return JSON.stringify(message) + "\n";
}
class TauriStdioTransport {
    constructor(server) {
        this._serverParams = server;
        this._readBuffer = new ReadBuffer();
    }
    async start() {
        if (this._serverId) {
            throw new Error("Transport already started");
        }
        const response = await startMcpServer({
            params: {
                command: this._serverParams.command,
                args: this._serverParams.args || [],
                env: this._serverParams.env || {}
            }
        });
        this._serverId = response.serverId;
        // Listen for server events (stdout, stderr, exit)
        listenToMcpServer(this._serverId, (event) => {
            console.info("Received event", event);
            switch (event.type) {
                case "stdout":
                    // Append the raw byte data (Uint8Array) to the read buffer
                    this._readBuffer.append(new Uint8Array(event.payload));
                    // Process the buffer to extract potential JSON-RPC messages
                    this.processReadBuffer();
                    break;
                case "stderr":
                    // Assuming UTF-8 encoding for the Vec<u8> payload
                    const stderrString = new TextDecoder().decode(new Uint8Array(event.payload));
                    console.error(`[${this._serverId}] STDERR:`, stderrString);
                    // Optionally, call an onerror handler if available
                    // this.onerror?.(new Error(stderrString));
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
                serverId: this._serverId
            });
        }
        catch (error) {
            console.error("Error killing MCP server", error);
        }
        this._serverId = undefined;
        this._readBuffer.clear();
    }
    async send(message) {
        if (!this._serverId) {
            throw new Error("Transport not started");
        }
        await sendToMcpServer({
            serverId: this._serverId,
            data: serializeMessage(message)
        });
    }
    processReadBuffer() {
        while (true) {
            try {
                const message = this._readBuffer.readMessage();
                console.log("message", message);
                if (message === null) {
                    break;
                }
                console.info("Received message", message);
                this.onmessage?.(message);
            }
            catch (error) {
                this.onerror?.(error);
            }
        }
    }
}

export { TauriStdioTransport };
