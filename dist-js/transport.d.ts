import type { StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio.js";
import type { Transport } from "@modelcontextprotocol/sdk/shared/transport.js";
import { type JSONRPCMessage } from "@modelcontextprotocol/sdk/types.js";
export declare function deserializeMessage(line: string): {
    method: string;
    jsonrpc: "2.0";
    id: string | number;
    params?: import("zod").objectOutputType<{
        _meta: import("zod").ZodOptional<import("zod").ZodObject<{
            progressToken: import("zod").ZodOptional<import("zod").ZodUnion<[import("zod").ZodString, import("zod").ZodNumber]>>;
        }, "passthrough", import("zod").ZodTypeAny, import("zod").objectOutputType<{
            progressToken: import("zod").ZodOptional<import("zod").ZodUnion<[import("zod").ZodString, import("zod").ZodNumber]>>;
        }, import("zod").ZodTypeAny, "passthrough">, import("zod").objectInputType<{
            progressToken: import("zod").ZodOptional<import("zod").ZodUnion<[import("zod").ZodString, import("zod").ZodNumber]>>;
        }, import("zod").ZodTypeAny, "passthrough">>>;
    }, import("zod").ZodTypeAny, "passthrough"> | undefined;
} | {
    method: string;
    jsonrpc: "2.0";
    params?: import("zod").objectOutputType<{
        _meta: import("zod").ZodOptional<import("zod").ZodObject<{}, "passthrough", import("zod").ZodTypeAny, import("zod").objectOutputType<{}, import("zod").ZodTypeAny, "passthrough">, import("zod").objectInputType<{}, import("zod").ZodTypeAny, "passthrough">>>;
    }, import("zod").ZodTypeAny, "passthrough"> | undefined;
} | {
    jsonrpc: "2.0";
    id: string | number;
    result: {
        _meta?: import("zod").objectOutputType<{}, import("zod").ZodTypeAny, "passthrough"> | undefined;
    } & {
        [k: string]: unknown;
    };
} | {
    jsonrpc: "2.0";
    id: string | number;
    error: {
        code: number;
        message: string;
        data?: unknown;
    };
};
export declare function serializeMessage(message: JSONRPCMessage): string;
export declare class TauriStdioTransport implements Transport {
    private _serverParams;
    private _serverId?;
    onclose?: () => void;
    onerror?: (error: Error) => void;
    onmessage?: (message: JSONRPCMessage) => void;
    constructor(server: StdioServerParameters);
    start(): Promise<void>;
    close(): Promise<void>;
    send(message: JSONRPCMessage): Promise<void>;
}
