const COMMANDS: &[&str] = &[
  "start_mcp_server",
  "send_to_mcp_server",
  "kill_mcp_server",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS).build();
}
