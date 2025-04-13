# Tauri Plugin tauri-plugin-mcp-manager

**Disclaimer:** This project was created with the assistance of an LLM and is a result of "vibe coding". It may not be stable or suitable for production use. Use at your own risk.

## Usage

This plugin provides the `TauriStdioTransport` class for the `@modelcontextprotocol/sdk`.

```typescript
// Example (e.g., in a frontend component or script)
import { Client } from "@modelcontextprotocol/sdk/client/index";
import type { StdioServerParameters } from "@modelcontextprotocol/sdk/client/stdio";
import { TauriStdioTransport } from "tauri-plugin-mcp-manager-api";

async function connectToMyMcpServer() {
  // Define how to start your MCP server process
  const serverParams: StdioServerParameters = {
    command: "path/to/your/mcp/server/executable", // Replace with the actual command
    args: ["--some-arg"], // Optional arguments for the server
    // env: { "MY_VAR": "value" } // Optional environment variables
  };

  try {
    // Create the Tauri transport using the plugin
    const transport = new TauriStdioTransport(serverParams);

    // Create an MCP client instance
    const client = new Client({ name: "MyTauriAppClient", version: "0.1.0" });

    // Connect the client using the transport
    // The plugin handles spawning and managing the server process based on serverParams
    await client.connect(transport);

    console.log("Successfully connected to the MCP server!");

    // Now you can interact with the server, e.g., list tools
    const toolList = await client.listTools();
    console.log("Available tools:", toolList);

    // ... other client interactions ...

    // Disconnecting: The transport will automatically terminate the managed
    // server process when the client disconnects or is garbage collected.
    // You typically don't need to call client.disconnect() explicitly unless
    // you want to terminate the server prematurely.

  } catch (error) {
    console.error("Failed to connect to MCP server:", error);
  }
}

// Call the function to connect when appropriate in your app
// connectToMyMcpServer();
```
