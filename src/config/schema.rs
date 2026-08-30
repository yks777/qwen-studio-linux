use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub performance: PerformanceSettings,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralSettings {
    #[serde(default = "default_true")]
    pub check_updates: bool,
    #[serde(default)]
    pub theme: String,
    /// Zoom level persistido por origem (1.0 = 100%), clamp 0.5..3.0
    #[serde(default = "default_zoom")]
    pub zoom: f64,
}

<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
fn default_zoom() -> f64 {
    1.0
=======
fn default_true() -> bool {
    true
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
fn default_true() -> bool {
    true
>>>>>>> c0c2f30 (Fix: Upload medias e username)
}

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> ce2f600 (optimization)
=======
>>>>>>> ce2f600 (optimization)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceSettings {
    #[serde(default)]
    pub force_software_rendering: bool,
}

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> 5877c22 (restore)
=======
>>>>>>> ce2f600 (optimization)
=======
>>>>>>> 5877c22 (restore)
=======
>>>>>>> ce2f600 (optimization)
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(
        rename = "transportType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transport_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(rename = "from", default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(rename = "fromId", default, skip_serializing_if = "Option::is_none")]
    pub from_id: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "toolName")]
    pub tool_name: String,
    #[serde(rename = "toolArguments", skip_serializing_if = "Option::is_none")]
    pub tool_arguments: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolListParams {
    #[serde(rename = "serverName")]
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogOptions {
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_id: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatExport {
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub exported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub available: bool,
    pub latest_version: String,
    pub release_notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
}
