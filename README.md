# Tauri Plugin tauri-plugin-mcp-manager

This Tauri plugin enables seamless integration of Model Context Protocol (MCP) servers using stdio transport in your Tauri applications. It provides a bridge between your Tauri frontend and MCP-compatible AI/ML services, making it easy to spawn, manage, and communicate with MCP server processes.

**Disclaimer:** 🎵 This project was born from the mystical art of vibe coding, where hallucinations meet reality in a dance of semicolons. Some say it was written during a full moon while listening to Baby Shark on repeat (doo doo doo doo doo doo). The code might work, it might not, it might transform into a unicorn - we're not making any promises! Use at your own risk, and remember: just like Baby Shark, this code is catchy but comes with no warranty. 🦈

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