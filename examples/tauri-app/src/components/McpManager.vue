<script setup lang="ts">
import { ref, type Ref } from "vue";
import { TauriStdioTransport } from "tauri-plugin-mcp-manager-api";
import { Client } from "@modelcontextprotocol/sdk/client/index";
import type { JSONRPCMessage } from "@modelcontextprotocol/sdk/types";
import type { StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio";

// Define Tool interface
interface Tool {
    name: string;
    description?: string; // Optional description
    // Add other properties if known/needed from the protocol
}

// Define interfaces for state and API calls locally if not exported
interface ServerState {
  id: string; // Unique ID for Vue key
  name: string;
  client: Client | null;
  transport: TauriStdioTransport | null; // Keep track of transport for closing
  isConnected: boolean;
  isConnecting: boolean;
  isStopping: boolean; // Added for granular stopping state
  connectionError: string | null;
  tools: any[]; // Define a specific Tool type if structure is known
  parameters: StdioServerParameters;
}

// Reactive state with types
const servers = ref<ServerState[]>([]);
const newServerName = ref<string>("");
// Remove separate command/args refs
// const newServerCommand = ref<string>("");
// const newServerArgs = ref<string>("");
// Add ref for JSON params
const newServerParamsJson = ref<string>("");
const isLoading = ref<boolean>(false);
const errorMsg = ref<string>("");

// No longer need onMounted or refreshServerList as state is local

// Helper function for validating StdioServerParameters JSON
function validateServerParams(jsonString: string): { params: StdioServerParameters | null; error: string | null } {
    try {
        const parsedJson = JSON.parse(jsonString);
        // Enhanced validation of the parsed JSON structure
        if (
            typeof parsedJson !== 'object' ||
            parsedJson === null ||
            typeof parsedJson.command !== 'string' ||
            !parsedJson.command ||
            !Array.isArray(parsedJson.args) ||
            !parsedJson.args.every((arg: any) => typeof arg === 'string') ||
            (parsedJson.env !== undefined && (typeof parsedJson.env !== 'object' || parsedJson.env === null || Array.isArray(parsedJson.env)))
        ) {
            return { params: null, error: 'Invalid JSON structure. Expected { "command": string, "args": string[], "env"?: { [key: string]: string } }.' };
        }
        // Ensure env values are strings if env exists
        if (parsedJson.env) {
            for (const key in parsedJson.env) {
                if (typeof parsedJson.env[key] !== 'string') {
                    return { params: null, error: `Invalid JSON structure. Environment variable "${key}" must be a string.` };
                }
            }
        }

        return {
            params: {
                command: parsedJson.command,
                args: parsedJson.args,
                env: parsedJson.env // Include env if present
            },
            error: null
        };
    } catch (jsonError: unknown) {
        const message = jsonError instanceof Error ? jsonError.message : String(jsonError);
        return { params: null, error: `Invalid JSON format: ${message}` };
    }
}

async function startServer(): Promise<void> {
  const name = newServerName.value.trim();
  const paramsJson = newServerParamsJson.value.trim();

  if (!name || !paramsJson) {
    errorMsg.value = "Server name and parameters JSON cannot be empty.";
    return;
  }

  // Use the validation helper function
  const validationResult = validateServerParams(paramsJson);
  if (validationResult.error) {
      errorMsg.value = validationResult.error;
      return;
  }
  const params = validationResult.params!;

  isLoading.value = true;
  errorMsg.value = ""; // Clear global error on action start
  try {
    // Generate a simple unique ID for the key
    const newServerId = `server-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

    // Add the server configuration to our local state
    // The actual process starts when connectToServer is called
    servers.value.push({
        id: newServerId, // Use generated ID
        name: name, // Use the provided name
        client: null,
        transport: null,
        isConnected: false,
        isConnecting: false,
        isStopping: false, // Initialize stopping state
        connectionError: null,
        tools: [],
        parameters: params,
    });

    // Clear the form
    newServerName.value = "";
    // Clear the JSON input instead of command/args
    // newServerCommand.value = "";
    // newServerArgs.value = "";
    newServerParamsJson.value = "";

  } catch (err: unknown) { // Type error as unknown
    console.error("Failed to start server:", err);
    // Check if err is an Error object or just a string/plain object
    const message = err instanceof Error ? err.message : JSON.stringify(err);
    errorMsg.value = `Failed to start server: ${message}`;
  } finally {
    isLoading.value = false;
  }
}

async function stopServer(index: number): Promise<void> {
    const server = servers.value[index];
    if (!server || server.isStopping) return; // Check if server exists and not already stopping

    // isLoading.value = true; // Use per-server stopping state instead
    server.isStopping = true;
    server.connectionError = null; // Clear specific server error on action start
    console.log(`Stopping server ${server.name} at index ${index}...`);

    // 1. Disconnect if connected
    if (server.isConnected || server.isConnecting) {
        console.log(`Disconnecting before stopping ${server.name}...`);
        // Reuse disconnect logic, ensuring transport is closed
        const transportToClose = server.transport;
        server.client = null;
        server.transport = null;
        server.isConnected = false;
        server.isConnecting = false;
        server.tools = [];
        server.connectionError = null; // Clear errors on manual stop

        if (transportToClose) {
            try {
                transportToClose.close();
                console.log(`Transport closed for ${server.name} during stop.`);
            } catch (err: unknown) {
                console.error(`Error closing transport during stop for ${server.name}:`, err);
                // Optionally surface this error, though we are removing the server anyway
            }
        }
    }

    // 2. Remove server from the list
    servers.value.splice(index, 1);
    console.log(`Server ${server.name} removed from list.`);
    // isLoading.value = false; // No need to reset global loading here
    // Note: isStopping is implicitly false as the server is removed
}

async function connectToServer(index: number): Promise<void> {
    const server = servers.value[index];
    // Added null check for server
    if (!server || server.isConnected || server.isConnecting || server.isStopping) return;

    server.isConnecting = true;
    server.connectionError = null;
    errorMsg.value = ""; // Clear global error on action start

    let transport: TauriStdioTransport | null = null; // Define transport in outer scope

    try {
        transport = new TauriStdioTransport(server.parameters);
        server.transport = transport; // Store transport instance

        // Try passing only the transport to the Client constructor
        const client = new Client({ name: 'TauriAppClient', version: '0.1.0' });

        // Handle potential transport closure/errors - TauriContextTransport doesn't explicitly emit 'close' or 'error' events
        // We rely on connect/send calls throwing errors, or manual disconnection state changes.
        // The previous addEventListener logic for close/error is removed as it's not supported by TauriContextTransport.

        // Call connect with client identification arguments again - *Correction*: connect doesn't take client args here
        await client.connect(transport);
        console.log(`Connected to ${server.name}`);

        server.client = client; // Store client instance
        server.isConnected = true;
        // Optionally list tools upon connection
        await listTools(index);

    } catch (err: unknown) { // Type error as unknown
        console.error(`Failed to connect to server ${server.name}:`, err);
         // Added null check for server again in catch block
        // Use the index directly, serverId is not defined here
        const currentServer = servers.value[index];
        if (currentServer) {
            const message = err instanceof Error ? err.message : JSON.stringify(err);
            currentServer.connectionError = `Connection failed: ${message}`;
            currentServer.client = null; // Ensure client is null on failed connection
            currentServer.transport = null; // Clear transport on failure
            currentServer.isConnected = false;

             // Close the transport if it was created and connection failed
            if (transport) {
                try {
                    transport.close();
                } catch (closeErr) {
                    console.error(`Error closing transport after connection failure for ${currentServer.name}:`, closeErr);
                }
            }
        }
    } finally {
         // Added null check for server again in finally block
        // Use the index directly, serverId is not defined here
        const currentServer = servers.value[index];
        if (currentServer) {
            currentServer.isConnecting = false;
        }
    }
}

async function disconnectFromServer(index: number): Promise<void> {
    const server = servers.value[index];
    // Added null check for server and connection status
    if (!server || !server.isConnected || server.isStopping) return;

    console.log(`Disconnecting from ${server.name} (Index: ${index})...`);
    const transportToClose = server.transport; // Get transport before clearing state

    // Clear state immediately
    server.client = null;
    server.transport = null;
    server.isConnected = false;
    server.isConnecting = false;
    server.tools = [];

    if (transportToClose) {
        try {
            // Use transport.close() instead of client.disconnect()
            transportToClose.close();
            console.log(`Transport closed for ${server.name}`);
            server.connectionError = null; // Clear error on successful manual disconnect
        } catch (err: unknown) { // Type error as unknown
            console.error(`Error during transport close for ${server.name}:`, err);
            const message = err instanceof Error ? err.message : JSON.stringify(err);
            // Set error state even if closing failed
            server.connectionError = `Disconnection error: ${message}`;
        }
    } else {
         console.warn(`No transport found to close for server ${server.name} during disconnect.`);
         server.connectionError = "Disconnection attempted without active transport.";
    }
}

// Re-enable listTools function
async function listTools(index: number): Promise<void> {
    const server = servers.value[index];
    // Check if server exists, is connected, and has a client instance
    if (!server || !server.isConnected || !server.client || server.isStopping) {
        console.warn(`Cannot list tools for server at index ${index}: Not connected, no client, or stopping.`);
        return;
    }
    try {
        console.log(`Listing tools for ${server.name}...`);
        server.connectionError = null; // Clear previous errors before listing
        // Expect an object, potentially like { tools: [...] }
        const result: any = await server.client.listTools();
        console.log(`Raw result from listTools for ${server.name}:`, result); // Log the raw result

        // Check if result is an object and has a 'tools' array property
        let foundTools: any[] = []; // Initialize as empty array
        if (result && typeof result === 'object' && Array.isArray(result.tools)) {
            foundTools = result.tools; // Assign the nested array
        } else if (Array.isArray(result)) {
             // Handle case where it might return just the array directly
             console.warn(`listTools returned an array directly for ${server.name}. Assigning.`);
             foundTools = result;
        } else {
            console.warn(`Unexpected format returned by listTools for ${server.name}. Expected an object with a 'tools' array or an array.`, result);
            // Keep foundTools as empty array
        }
        server.tools = foundTools; // Assign the extracted array (or empty) directly to any[]
        console.log(`Processed tools for ${server.name}:`, server.tools);
    } catch (err: unknown) {
        console.error(`Failed to list tools for ${server.name}:`, err);
        const message = err instanceof Error ? err.message : JSON.stringify(err);
        // Check if server still exists at the index before updating state
        const currentServer = servers.value[index];
        if (currentServer) {
             currentServer.connectionError = `Failed to list tools: ${message}`;
             currentServer.tools = []; // Clear tools on error
        }
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
    <form @submit.prevent="startServer" class="add-server-form" style="flex-direction: column; align-items: stretch;">
      <input
        type="text"
        v-model="newServerName"
        placeholder="Unique Server Name"
        :disabled="isLoading"
        required
      />
      <!-- Replace command/args inputs with textarea for JSON -->
      <!--
      <input
        type="text"
        v-model="newServerCommand"
        placeholder="Command (e.g., node, python)"
        :disabled="isLoading"
        required
      />
      <input
        type="text"
        v-model="newServerArgs"
        placeholder="Arguments (space-separated, e.g., server.js -p 8000)"
        :disabled="isLoading"
      />
      -->
      <textarea
        v-model="newServerParamsJson"
        placeholder='Enter server parameters as JSON, e.g.,\n{\n  "command": "node",\n  "args": ["server.js", "-p", "8080"],\n  "env": { "MY_VAR": "value" }\n}'
        rows="5"
        :disabled="isLoading"
        required
        style="font-family: monospace; white-space: pre; overflow-wrap: normal; overflow-x: scroll;"
      ></textarea>
      <!-- Add input for ENV variables if needed -->
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
          <span v-else-if="server.isStopping"> (Stopping...)</span>
          <span v-else-if="server.isConnected" style="color: green;"> (Connected)</span>
          <span v-else style="color: gray;"> (Disconnected)</span>
          <div v-if="server.connectionError" class="error-message" style="font-size: 0.8em; margin-top: 3px;">{{ server.connectionError }}</div>
        </div>

        <div class="server-actions">
          <!-- Connect/Disconnect Buttons -->
           <button
             v-if="!server.isConnected"
             @click="connectToServer(index)"
             :disabled="isLoading || server.isConnecting || server.isStopping"
           >
             {{ server.isConnecting ? 'Connecting...' : 'Connect' }}
           </button>
           <button
             v-if="server.isConnected"
             @click="disconnectFromServer(index)"
             :disabled="isLoading || server.isStopping"
           >
             Disconnect
           </button>
          <!-- Stop Button -->
          <button @click="stopServer(index)" :disabled="isLoading || server.isStopping">
             {{ server.isStopping ? 'Stopping...' : 'Stop' }}
          </button>
          <!-- List Tools Button (Optional Manual Trigger) -->
          <button
            v-if="server.isConnected && !server.isStopping"
            @click="listTools(index)"
            :disabled="isLoading || !server.client"
           >
            List Tools
          </button>
        </div>

         <!-- Display Tools -->
         <div v-if="server.isConnected && server.tools.length > 0" class="tool-list">
            <strong>Tools:</strong>
            <ul>
                <!-- Use tool.name or a unique ID if available for the key -->
                <li v-for="tool in server.tools" :key="tool.name">{{ tool.name }}{{ tool.description ? `: ${tool.description}` : '' }}</li>
            </ul>
         </div>
         <div v-if="server.isConnected && server.tools.length === 0 && !server.connectionError && !server.isConnecting" class="no-tools">
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