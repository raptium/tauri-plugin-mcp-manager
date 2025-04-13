 # Tauri MCP Manager Example App

This is an example application demonstrating the usage of the Tauri MCP Manager plugin. The application is built with Vue 3, TypeScript, and Tauri 2.0, providing a modern desktop application experience.

## Features

- Built with Vue 3 and TypeScript
- Tauri 2.0 integration
- Model Context Protocol (MCP) SDK integration
- Modern development setup with Vite
- Code quality tools (Biome for linting, Husky for git hooks)

## Prerequisites

- [Node.js](https://nodejs.org/) (Latest LTS version recommended)
- [Rust](https://www.rust-lang.org/) (Latest stable version)
- [VS Code](https://code.visualstudio.com/) (Recommended IDE)

## Development Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm run tauri dev
   ```

## Available Scripts

- `npm run dev` - Start Vite development server
- `npm run build` - Build the application
- `npm run preview` - Preview the production build
- `npm run tauri` - Run Tauri commands
- `npm run lint` - Run Biome linter
- `npm run lint:fix` - Fix linting issues automatically

## Project Structure

- `src/` - Frontend source code
  - `components/` - Vue components
  - `App.vue` - Main application component
  - `main.js` - Application entry point
- `src-tauri/` - Rust backend code
- `public/` - Static assets
- `assets/` - Application assets

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Vue Language Features (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin)

## Dependencies

- `@modelcontextprotocol/sdk` - MCP SDK for model context management
- `@tauri-apps/api` - Tauri API for desktop application features
- `tauri-plugin-mcp-manager-api` - Custom Tauri plugin for MCP management

## License

This project is part of the Tauri MCP Manager plugin example suite.