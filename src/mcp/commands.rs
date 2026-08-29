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
pub async fn mcp_client_tool_list(
    app: tauri::AppHandle,
    params: ToolListParams,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    // Fast path: read lock if already connected
    let bridge = {
        let is_connected = state.mcp.read().await.is_connected();
        if is_connected {
            let mcp = state.mcp.read().await;
            // Need to clone Arc while holding read lock; ensure_bridge not needed
            // but we need to get the bridge Arc — do it via read, then drop
            // To avoid double-lock, we check again with write only if None
            // Simpler: try read, if bridge exists clone it
            if let Some(b) = mcp.bridge_clone() {
                b
            } else {
                drop(mcp);
                let mut mcp_w = state.mcp.write().await;
                mcp_w.ensure_bridge(&app).await?
            }
        } else {
            let mut mcp = state.mcp.write().await;
            mcp.ensure_bridge(&app).await?
        }
    };
    log::info!("[MCP] tool_list {}", params.server_name);
    let r = bridge
        .send(
            "listTools",
            serde_json::json!({ "serverName": params.server_name }),
        )
        .await
        .map_err(|e| e.to_string());
    log::info!(
        "[MCP] tool_list {} -> {}",
        params.server_name,
        if r.is_ok() { "ok" } else { "err" }
    );
    r
}

#[tauri::command]
pub async fn mcp_client_tool_call(
    app: tauri::AppHandle,
    params: ToolCallParams,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let bridge = {
        let is_connected = state.mcp.read().await.is_connected();
        if is_connected {
            let mcp = state.mcp.read().await;
            if let Some(b) = mcp.bridge_clone() {
                b
            } else {
                drop(mcp);
                let mut mcp_w = state.mcp.write().await;
                mcp_w.ensure_bridge(&app).await?
            }
        } else {
            let mut mcp = state.mcp.write().await;
            mcp.ensure_bridge(&app).await?
        }
    };
    log::info!(
        "[MCP] tool_call {}#{}",
        params.server_name,
        params.tool_name
    );
    let r = bridge
        .send(
            "callTool",
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
            serde_json::to_value(&params).map_err(|e| e.to_string())?,
=======
            serde_json::to_value(&params).unwrap_or_default(),
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
            serde_json::to_value(&params).map_err(|e| e.to_string())?,
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
            serde_json::to_value(&params).unwrap_or_default(),
>>>>>>> c0c2f30 (Fix: Upload medias e username)
        )
        .await
        .map_err(|e| e.to_string());
    log::info!(
        "[MCP] tool_call {}#{} -> {}",
        params.server_name,
        params.tool_name,
        if r.is_ok() { "ok" } else { "err" }
    );
    r
}

#[tauri::command]
pub async fn mcp_client_get_config(
    _app: tauri::AppHandle,
) -> Result<HashMap<String, McpServerConfig>, String> {
    crate::mcp::config::load_config()
}

fn validate_mcp_config(config: &HashMap<String, McpServerConfig>) -> Result<(), String> {
    const MAX_SERVERS: usize = 32;
    const MAX_ARGS: usize = 64;
    const MAX_ARG_LEN: usize = 1024;
    const ALLOWED_COMMANDS: &[&str] = &["npx", "node", "nodejs", "python3", "python", "uvx", "bunx", "deno"];

    if config.len() > MAX_SERVERS {
        return Err(format!("Too many servers: {}", config.len()));
    }
    for (name, cfg) in config {
        if name.is_empty() || name.len() > 128 {
            return Err(format!("Invalid server name: {}", name));
        }
        if name.chars().any(|c| c == '/' || c == '\\' || c == '\0') {
            return Err(format!("Invalid server name characters: {}", name));
        }
        if cfg.command.is_empty() || cfg.command.len() > 512 {
            return Err(format!("Invalid command for {}", name));
        }
        // Allow absolute paths or allowed commands
        let is_allowed = ALLOWED_COMMANDS.contains(&cfg.command.as_str())
            || cfg.command.starts_with('/')
            || cfg.command.starts_with("./");
        if !is_allowed {
            return Err(format!(
                "Command '{}' not allowed for {} (allowed: {} or absolute path)",
                cfg.command,
                name,
                ALLOWED_COMMANDS.join(", ")
            ));
        }
        if cfg.args.len() > MAX_ARGS {
            return Err(format!("Too many args for {}", name));
        }
        for arg in &cfg.args {
            if arg.len() > MAX_ARG_LEN {
                return Err(format!("Arg too long for {}", name));
            }
            if arg.contains('\0') {
                return Err(format!("Invalid arg for {}", name));
            }
        }
        if let Some(url) = &cfg.url {
            if url.len() > 2048 {
                return Err(format!("URL too long for {}", name));
            }
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err(format!("URL must be http(s) for {}", name));
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn mcp_client_update_config(
    app: tauri::AppHandle,
    config: HashMap<String, McpServerConfig>,
) -> Result<HashMap<String, McpServerConfig>, String> {
    validate_mcp_config(&config)?;

    let state = app.state::<AppState>();

    // New behavior: respect explicit deletions. Only keep file_config entries that are
    // not intentionally removed by the UI. If UI sends empty map, it means "delete all" — respect it.
    // We still auto-inject qwen-core only if config is empty (first run), not on every update.
    let mut merged = config;
    let had_qwen_core = merged.contains_key("qwen-core");

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    // Do not merge back deleted servers: only insert qwen-core if this is initial setup (merged empty)
    if merged.is_empty() {
        if let Some(qc) = crate::config::defaults::mcp_servers().get("qwen-core") {
            merged
                .entry("qwen-core".into())
                .or_insert_with(|| qc.clone());
        }
    } else if !had_qwen_core {
        // User explicitly removed qwen-core, respect choice — do not re-add
<<<<<<< HEAD
=======
    if let Some(qc) = crate::config::defaults::mcp_servers().get("qwen-core") {
        merged
            .entry("qwen-core".into())
            .or_insert_with(|| qc.clone());
<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
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
    bridge
        .send("updateConfig", serde_json::json!({ "config": &merged }))
        .await
        .map_err(|e| e.to_string())?;

    log::info!("[MCP] update_config pushed");
    Ok(merged)
}
