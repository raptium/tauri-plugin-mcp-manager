export interface StdioServerParams {
    command: string;
    args: string[];
    env?: Record<string, string>;
}
export interface StartRequest {
    params: StdioServerParams;
}
export interface StartResponse {
    serverId: string;
}
export interface ServerIdPayload {
    serverId: string;
}
export type KillRequest = ServerIdPayload;
export interface SendRequest {
    serverId: string;
    data: string;
}
interface ServerEventStdout {
    type: "stdout";
    payload: number[];
}
interface ServerEventStderr {
    type: "stderr";
    payload: number[];
}
interface ServerEventExit {
    type: "exit";
    payload: number | null;
}
export type ServerEvent = ServerEventStdout | ServerEventStderr | ServerEventExit;
export declare function startMcpServer(payload: StartRequest): Promise<StartResponse>;
export declare function sendToMcpServer(payload: SendRequest): Promise<void>;
export declare function killMcpServer(payload: KillRequest): Promise<void>;
/**
 * Listens for messages from a specific MCP server process, identified by its internal serverId.
 *
 * @param serverId The unique internal ID of the server (from StartResponse).
 * @param onMessage A callback function to handle incoming messages.
 *                  The payload includes serverId and the message data.
 * @returns A promise that resolves to an unlisten function.
 */
export declare function listenToMcpServer(serverId: string, onMessage: (payload: ServerEvent) => void): Promise<() => void>;
export {};
