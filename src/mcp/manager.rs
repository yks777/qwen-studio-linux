use std::collections::HashMap;
use std::sync::Arc;
use super::bridge::Bridge;
use crate::config::schema::McpServerConfig;

pub struct McpManager {
    bridge: Option<Arc<Bridge>>,
    config: HashMap<String, McpServerConfig>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            bridge: None,
            config: HashMap::new(),
        }
    }

    pub async fn ensure_bridge(&mut self, app: &tauri::AppHandle) -> Result<Arc<Bridge>, String> {
        if let Some(ref bridge) = self.bridge {
            return Ok(Arc::clone(bridge));
        }

        let bridge = Arc::new(Bridge::new(Some(app)).await.map_err(|e| {
            log::error!("[MCP] Bridge spawn failed: {}", e);
            e.to_string()
        })?);

        let config = crate::mcp::config::load_config()?;
        bridge.send("updateConfig", serde_json::json!({ "config": &config }))
            .await
            .map_err(|e| format!("Config update: {}", e))?;

        self.bridge = Some(Arc::clone(&bridge));
        self.config = config;
        log::info!("[MCP] Bridge ready with {} servers", self.config.len());
        Ok(bridge)
    }

    pub async fn close(&mut self) {
        if let Some(bridge) = self.bridge.take() {
            bridge.shutdown().await;
        }
    }

    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool {
        self.bridge.is_some()
    }

    pub fn is_connected(&self) -> bool {
        self.bridge.is_some()
    }

    pub fn config_eq(&self, other: &HashMap<String, McpServerConfig>) -> bool {
        serde_json::to_string(&self.config).ok() == serde_json::to_string(other).ok()
    }
}
