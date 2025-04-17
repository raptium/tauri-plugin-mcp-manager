import type { StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio.js";
import type { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import {
	type JSONRPCMessage,
	JSONRPCMessageSchema,
} from "@modelcontextprotocol/sdk/types.js";

import {
	type ServerEvent,
	killMcpServer,
	listenToMcpServer,
	sendToMcpServer,
	startMcpServer,
} from "./commands";

export function deserializeMessage(line: string) {
	return JSONRPCMessageSchema.parse(JSON.parse(line));
}

export function serializeMessage(message: JSONRPCMessage) {
	return `${JSON.stringify(message)}\n`;
}

export class TauriStdioTransport implements Transport {
	private _serverParams: StdioServerParameters;
	private _serverId?: string;

	onclose?: () => void;
	onerror?: (error: Error) => void;
	onmessage?: (message: JSONRPCMessage) => void;

	constructor(server: StdioServerParameters) {
		this._serverParams = server;
	}

	async start(): Promise<void> {
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
		listenToMcpServer(this._serverId, (event: ServerEvent) => {
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
					console.error(
						`[${this._serverId}] STDERR:`,
						decoder.decode(new Uint8Array(event.payload)),
					);
					break;
				case "exit":
					console.info(
						`[${this._serverId}] Process exited with code:`,
						event.payload,
					);
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
				serverId: this._serverId,
			});
		} catch (error) {
			console.error("Error killing MCP server", error);
		}

		this._serverId = undefined;
	}

	async send(message: JSONRPCMessage): Promise<void> {
		if (!this._serverId) {
			throw new Error("Transport not started");
		}

		await sendToMcpServer({
			serverId: this._serverId,
			data: serializeMessage(message),
		});
	}
}
