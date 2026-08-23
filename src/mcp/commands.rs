use crate::app::state::AppState;
use crate::config::schema::{McpServerConfig, ToolCallParams, ToolListParams};
use std::collections::HashMap;
use tauri::Manager;

#[tauri::command]
pub async fn mcp_client_connect(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    log::info!("[MCP] connect");
    {
        let mut mcp = state.mcp.write().await;
        mcp.ensure_bridge(&app).await?;
    }
    log::info!("[MCP] connect done");
    Ok(())
}

#[tauri::command]
pub async fn mcp_client_close(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    log::info!("[MCP] close");
    let mut mcp = state.mcp.write().await;
    mcp.close().await;
    log::info!("[MCP] close done");
    Ok(())
}

#[tauri::command]
pub async fn mcp_client_tool_list(app: tauri::AppHandle, params: ToolListParams) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let bridge = {
        let mut mcp = state.mcp.write().await;
        mcp.ensure_bridge(&app).await?
    };
    log::info!("[MCP] tool_list {}", params.server_name);
    let r = bridge.send("listTools", serde_json::json!({ "serverName": params.server_name }))
        .await
        .map_err(|e| e.to_string());
    log::info!("[MCP] tool_list {} -> {}", params.server_name, if r.is_ok() { "ok" } else { "err" });
    r
}

#[tauri::command]
pub async fn mcp_client_tool_call(app: tauri::AppHandle, params: ToolCallParams) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let bridge = {
        let mut mcp = state.mcp.write().await;
        mcp.ensure_bridge(&app).await?
    };
    log::info!("[MCP] tool_call {}#{}", params.server_name, params.tool_name);
    let r = bridge.send("callTool", serde_json::to_value(&params).unwrap_or_default())
        .await
        .map_err(|e| e.to_string());
    log::info!("[MCP] tool_call {}#{} -> {}", params.server_name, params.tool_name, if r.is_ok() { "ok" } else { "err" });
    r
}

#[tauri::command]
pub async fn mcp_client_get_config(_app: tauri::AppHandle) -> Result<HashMap<String, McpServerConfig>, String> {
    crate::mcp::config::load_config()
}

#[tauri::command]
pub async fn mcp_client_update_config(
    app: tauri::AppHandle,
    config: HashMap<String, McpServerConfig>,
) -> Result<HashMap<String, McpServerConfig>, String> {
    let state = app.state::<AppState>();

    let file_config = crate::mcp::config::load_config().unwrap_or_default();
    let mut merged = config;
    for (name, cfg) in file_config {
        merged.entry(name).or_insert(cfg);
    }

    if let Some(qc) = crate::config::defaults::mcp_servers().get("qwen-core") {
        merged.entry("qwen-core".into()).or_insert_with(|| qc.clone());
    }

    let merged = crate::config::defaults::normalize_mcp(merged);

    let mut settings = crate::config::store::load();
    settings.mcp_servers = merged.clone();
    crate::config::store::save(&settings)?;

    log::info!("[MCP] update_config ({} servers)", merged.len());

    let bridge = {
        let mut mcp = state.mcp.write().await;
        // Debounce: if the bridge is already running with an identical config,
        // skip the redundant push (prevents repeated full reconnect storms).
        if mcp.is_connected() && mcp.config_eq(&merged) {
            log::info!("[MCP] update_config skipped (identical config)");
            return Ok(merged);
        }
        mcp.ensure_bridge(&app).await?
    };
    bridge.send("updateConfig", serde_json::json!({ "config": &merged }))
        .await
        .map_err(|e| e.to_string())?;

    log::info!("[MCP] update_config pushed");
    Ok(merged)
}
