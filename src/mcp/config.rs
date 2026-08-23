use std::collections::HashMap;
use crate::config::schema::McpServerConfig;
use crate::config;

pub fn load_config() -> Result<HashMap<String, McpServerConfig>, String> {
    let settings = config::store::load();
    if !settings.mcp_servers.is_empty() {
        return Ok(config::defaults::normalize_mcp(settings.mcp_servers));
    }
    log::warn!("[MCP] No mcpServers in settings, using defaults");
    Ok(config::defaults::mcp_servers())
}
