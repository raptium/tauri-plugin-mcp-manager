import { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import { JSONRPCMessage, JSONRPCMessageSchema } from "@modelcontextprotocol/sdk/types.js";
import { type StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio.js";

import { startMcpServer, sendToMcpServer, listenToMcpServer, killMcpServer, ServerEvent } from "./commands";

export class ReadBuffer {
    private _buffer: Uint8Array | undefined;
    private _decoder = new TextDecoder("utf8"); // Use TextDecoder for browser compatibility

    append(chunk: Uint8Array) {
        if (!this._buffer) {
            this._buffer = chunk;
        } else {
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
            console.log("no newline found");
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
export function deserializeMessage(line: string) {
    return JSONRPCMessageSchema.parse(JSON.parse(line));
}
export function serializeMessage(message: JSONRPCMessage) {
    return JSON.stringify(message) + "\n";
}


export class TauriStdioTransport implements Transport {
    private _serverParams: StdioServerParameters;
    private _serverId?: string;
    private _readBuffer: ReadBuffer;

    onclose?: () => void;
    onerror?: (error: Error) => void;
    onmessage?: (message: JSONRPCMessage) => void;

    constructor(server: StdioServerParameters) {
        this._serverParams = server;
        this._readBuffer = new ReadBuffer();
    }

    async start(): Promise<void> {
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
        listenToMcpServer(this._serverId, (event: ServerEvent) => {
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

    async close(): Promise<void> {
        if (!this._serverId) {
            throw new Error("Transport not started");
        }

        try {
            await killMcpServer({
                serverId: this._serverId
            });
        } catch (error) {
            console.error("Error killing MCP server", error);
        }

        this._serverId = undefined;
        this._readBuffer.clear();
    }

    async send(message: JSONRPCMessage): Promise<void> {
        if (!this._serverId) {
            throw new Error("Transport not started");
        }

        await sendToMcpServer({
            serverId: this._serverId,
            data: serializeMessage(message)
        });
    }

    private processReadBuffer() {
        while (true) {
            try {
                const message = this._readBuffer.readMessage();
                console.log("message", message);
                if (message === null) {
                    break;
                }
                console.info("Received message", message);
                this.onmessage?.(message);
            } catch (error) {
                this.onerror?.(error as Error);
            }
        }
    }
}
