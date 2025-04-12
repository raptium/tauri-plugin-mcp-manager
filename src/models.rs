use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StdioServerParams {
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StartRequest {
    pub name: String,
    pub params: StdioServerParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartResponse {
    pub server_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerNamePayload {
    pub name: String,
}

pub type KillRequest = ServerNamePayload;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendRequest {
    pub name: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerMessagePayload {
    pub server_id: String,
    pub name: String,
    pub data: String,
}
