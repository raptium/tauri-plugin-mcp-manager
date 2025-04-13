<script setup lang="ts">
import { Client } from "@modelcontextprotocol/sdk/client/index";
import type { StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio";
import { TauriStdioTransport } from "tauri-plugin-mcp-manager-api";
import { ref } from "vue";

// Define Tool interface
interface Tool {
  name: string;
  description?: string;
}

// Simplified ServerState interface
interface ServerState {
  id: string;
  name: string;
  client: Client | null;
  isConnected: boolean;
  isConnecting: boolean;
  connectionError: string | null;
  tools: Tool[];
  parameters: StdioServerParameters;
}

// Reactive state with types
const servers = ref<ServerState[]>([]);
const newServerName = ref<string>("");
const newServerParamsJson = ref<string>("");
const isLoading = ref<boolean>(false);
const errorMsg = ref<string>("");

// No longer need onMounted or refreshServerList as state is local

// Helper function for validating StdioServerParameters JSON
function validateServerParams(jsonString: string): {
  params: StdioServerParameters | null;
  error: string | null;
} {
  try {
    const parsedJson = JSON.parse(jsonString);
    if (
      typeof parsedJson !== "object" ||
      parsedJson === null ||
      typeof parsedJson.command !== "string" ||
      !parsedJson.command ||
      !Array.isArray(parsedJson.args) ||
      !parsedJson.args.every((arg: string) => typeof arg === "string") ||
      (parsedJson.env !== undefined &&
        (typeof parsedJson.env !== "object" ||
          parsedJson.env === null ||
          Array.isArray(parsedJson.env)))
    ) {
      return {
        params: null,
        error:
          'Invalid JSON structure. Expected { "command": string, "args": string[], "env"?: { [key: string]: string } }.',
      };
    }
    if (parsedJson.env) {
      for (const key in parsedJson.env) {
        if (typeof parsedJson.env[key] !== "string") {
          return {
            params: null,
            error: `Invalid JSON structure. Environment variable "${key}" must be a string.`,
          };
        }
      }
    }

    return {
      params: {
        command: parsedJson.command,
        args: parsedJson.args,
        env: parsedJson.env,
      },
      error: null,
    };
  } catch (jsonError: unknown) {
    const message =
      jsonError instanceof Error ? jsonError.message : String(jsonError);
    return { params: null, error: `Invalid JSON format: ${message}` };
  }
}

async function addServer(): Promise<void> {
  const name = newServerName.value.trim();
  const paramsJson = newServerParamsJson.value.trim();

  if (!name || !paramsJson) {
    errorMsg.value = "Server name and parameters JSON cannot be empty.";
    return;
  }

  const validationResult = validateServerParams(paramsJson);
  if (validationResult.error) {
    errorMsg.value = validationResult.error;
    return;
  }
  const params = validationResult.params;
  if (!params) {
    errorMsg.value = "Failed to parse server parameters.";
    return;
  }

  isLoading.value = true;
  errorMsg.value = "";

  try {
    const newServerId = `server-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

    servers.value.push({
      id: newServerId,
      name: name,
      client: null,
      isConnected: false,
      isConnecting: false,
      connectionError: null,
      tools: [],
      parameters: params,
    });

    // Clear the form
    newServerName.value = "";
    newServerParamsJson.value = "";
  } catch (err: unknown) {
    console.error("Failed to add server:", err);
    const message = err instanceof Error ? err.message : JSON.stringify(err);
    errorMsg.value = `Failed to add server: ${message}`;
  } finally {
    isLoading.value = false;
  }
}

async function removeServer(index: number): Promise<void> {
  const server = servers.value[index];
  if (!server) return;

  // Disconnect if connected
  if (server.isConnected && server.client) {
    try {
      await disconnectFromServer(index);
    } catch (err) {
      console.error(
        `Error disconnecting server ${server.name} during removal:`,
        err,
      );
    }
  }

  // Remove server from the list
  servers.value.splice(index, 1);
}

async function connectToServer(index: number): Promise<void> {
  const server = servers.value[index];
  if (!server || server.isConnected || server.client || server.isConnecting)
    return;

  server.connectionError = null;
  errorMsg.value = "";
  server.isConnecting = true;

  try {
    const client = new Client({ name: "TauriAppClient", version: "0.1.0" });
    await client.connect(new TauriStdioTransport(server.parameters));
    console.log(`Connected to ${server.name}`);

    server.client = client;
    server.isConnected = true;
    await listTools(index);
  } catch (err: unknown) {
    console.error(`Failed to connect to server ${server.name}:`, err);
    const message = err instanceof Error ? err.message : JSON.stringify(err);
    server.connectionError = `Connection failed: ${message}`;
    server.client = null;
    server.isConnected = false;
  } finally {
    server.isConnecting = false;
  }
}

async function disconnectFromServer(index: number): Promise<void> {
  const server = servers.value[index];
  if (!server || !server.isConnected || !server.client) return;

  console.log(`Disconnecting from ${server.name}...`);

  try {
    // Let the client handle the disconnection
    server.client = null;
    server.isConnected = false;
    server.tools = [];
    server.connectionError = null;
  } catch (err: unknown) {
    console.error(`Error during disconnect for ${server.name}:`, err);
    const message = err instanceof Error ? err.message : JSON.stringify(err);
    server.connectionError = `Disconnection error: ${message}`;
  }
}

async function listTools(index: number): Promise<void> {
  const server = servers.value[index];
  if (!server || !server.isConnected || !server.client) {
    console.warn(
      `Cannot list tools for server at index ${index}: Not connected or no client.`,
    );
    return;
  }

  try {
    console.log(`Listing tools for ${server.name}...`);
    server.connectionError = null;

    const result = await server.client.listTools();
    console.log(`Raw result from listTools for ${server.name}:`, result);

    let foundTools: Tool[] = [];
    if (result && typeof result === "object" && Array.isArray(result.tools)) {
      foundTools = result.tools as Tool[];
    } else if (Array.isArray(result)) {
      foundTools = result as Tool[];
    } else {
      console.warn(
        `Unexpected format returned by listTools for ${server.name}. Expected an object with a 'tools' array or an array.`,
        result,
      );
    }
    server.tools = foundTools;
    console.log(`Processed tools for ${server.name}:`, server.tools);
  } catch (err: unknown) {
    console.error(`Failed to list tools for ${server.name}:`, err);
    const message = err instanceof Error ? err.message : JSON.stringify(err);
    server.connectionError = `Failed to list tools: ${message}`;
    server.tools = [];
  }
}

// async function getServerStatus(serverId: string): Promise<void> {
//     const server = servers.value[serverId];
//     if (!server || !server.isConnected || !server.client) return;
//     try {
//         // Assuming a status method exists and defining its return type
//         // interface ServerStatus { online: boolean; uptime?: number; }
//         // const status: ServerStatus = await server.client.getStatus();
//         const status: any = await server.client.getStatus(); // Use any if type unknown
//         console.log(`Status for ${server.name}:`, status);
//         // Update some state based on status if needed
//     } catch (err: unknown) {
//         console.error(`Failed to get status for ${server.name}:`, err);
//         const message = err instanceof Error ? err.message : JSON.stringify(err);
//         if (servers.value[serverId]) { // Check if server still exists
//              servers.value[serverId].connectionError = `Failed to get status: ${message}`;
//         }
//     }
// }

// Remove MCP client/transport related logic and imports - This comment is now outdated
</script>

<template>
  <div class="mcp-manager">
    <h2>MCP Server Manager (Plugin API)</h2>

    <!-- Start Server Form -->
    <form @submit.prevent="addServer" class="add-server-form" style="flex-direction: column; align-items: stretch;">
      <input
        type="text"
        v-model="newServerName"
        placeholder="Unique Server Name"
        :disabled="isLoading"
        required
      />
      <textarea
        v-model="newServerParamsJson"
        placeholder='Enter server parameters as JSON, e.g.,\n{\n  "command": "node",\n  "args": ["server.js", "-p", "8080"],\n  "env": { "MY_VAR": "value" }\n}'
        rows="5"
        :disabled="isLoading"
        required
        style="font-family: monospace; white-space: pre; overflow-wrap: normal; overflow-x: scroll;"
      ></textarea>
      <button type="submit" :disabled="isLoading || !newServerName.trim() || !newServerParamsJson.trim()">
        {{ isLoading ? "Starting..." : "Start Server" }}
      </button>
    </form>

    <div v-if="errorMsg" class="error-message">{{ errorMsg }}</div>
    <div v-if="isLoading && !Object.keys(servers).length" class="loading-message">Loading...</div>

    <!-- Server List -->
    <div class="server-list">
      <p v-if="servers.length > 0">Running Servers:</p>
      <div v-for="(server, index) in servers" :key="server.id" class="server-item">
        <div class="server-info">
          <strong>{{ server.name }}</strong>
          <span v-if="server.isConnecting"> (Connecting...)</span>
          <span v-else-if="server.isConnected" style="color: green;"> (Connected)</span>
          <span v-else style="color: gray;"> (Disconnected)</span>
          <div v-if="server.connectionError" class="error-message" style="font-size: 0.8em; margin-top: 3px;">{{ server.connectionError }}</div>
        </div>

        <div class="server-actions">
          <button
            v-if="!server.isConnected"
            @click="connectToServer(index)"
            :disabled="isLoading || server.isConnected || server.isConnecting"
          >
            {{ server.isConnecting ? 'Connecting...' : 'Connect' }}
          </button>
          <button
            v-if="server.isConnected"
            @click="disconnectFromServer(index)"
            :disabled="isLoading || !server.client"
          >
            Disconnect
          </button>
          <button @click="removeServer(index)" :disabled="isLoading">
            Remove
          </button>
        </div>

         <!-- Display Tools -->
         <div v-if="server.isConnected && server.tools.length > 0" class="tool-list">
            <strong>Tools:</strong>
            <ul>
                <li v-for="tool in server.tools" :key="tool.name">{{ tool.name }}{{ tool.description ? `: ${tool.description}` : '' }}</li>
            </ul>
         </div>
         <div v-if="server.isConnected && server.tools.length === 0 && !server.connectionError" class="no-tools">
            No tools available or yet listed for this server.
         </div>

      </div>
      <div v-if="!isLoading && servers.length === 0 && !errorMsg" class="no-servers">
        No servers currently running (managed by this UI).
      </div>
    </div>
  </div>
</template>

<style scoped>
.mcp-manager {
  margin-top: 20px;
  padding: 15px;
  border: 1px solid #ccc;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.add-server-form {
  margin-bottom: 15px;
  display: flex;
  gap: 10px;
}

.add-server-form input {
  flex-grow: 1;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.add-server-form textarea {
  flex-grow: 1;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  resize: vertical; /* Allow vertical resizing */
  font-family: monospace; /* Ensure monospace for JSON */
  white-space: pre; /* Prevent auto-wrapping */
  overflow-wrap: normal; /* Prevent breaking words */
  overflow-x: scroll; /* Allow horizontal scroll if needed */
}

.add-server-form button {
  padding: 8px 15px;
  cursor: pointer;
}

.server-list {
  margin-top: 15px;
}

.server-item {
  border: 1px solid #eee;
  background-color: #fff;
  padding: 10px;
  margin-bottom: 10px;
  border-radius: 4px;
}

.server-info {
  margin-bottom: 10px;
}

.server-actions {
  display: flex;
  gap: 5px;
  flex-wrap: wrap; /* Allow buttons to wrap on smaller screens */
  align-items: center; /* Align items vertically */
  margin-bottom: 10px;
}

.server-actions button {
    padding: 5px 10px;
    cursor: pointer;
}

.error-message {
  color: #e53e3e; /* Tailwind red-600 */
  font-size: 0.9em;
  margin-top: 5px;
  margin-bottom: 10px;
}

.loading-message, .no-servers {
    margin-top: 10px;
    color: #666;
    font-style: italic;
}

button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.tool-list {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed #eee;
  font-size: 0.9em;
}

.tool-list ul {
  list-style: disc;
  margin-left: 20px;
  padding-left: 0;
}

.no-tools {
    margin-top: 5px;
    color: #666;
    font-style: italic;
    font-size: 0.9em;
}
</style> 