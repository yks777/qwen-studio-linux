use std::sync::OnceLock;

static INIT_SCRIPT: OnceLock<String> = OnceLock::new();
static PICKER_INIT_SCRIPT: OnceLock<String> = OnceLock::new();

pub fn build_init_script() -> String {
    INIT_SCRIPT
        .get_or_init(|| {
            let pre_load_script = r#"
        (function() {
            try { document.documentElement.style.backgroundColor = '#0f1115'; } catch (e) {}
            var hostname = window.location.hostname;
            var pathname = window.location.pathname;
            var isLoginPage = pathname.includes('login') || pathname.includes('auth') || pathname.includes('callback') || pathname.includes('oauth');
            if (hostname !== 'chat.qwen.ai' || isLoginPage) return;
            try {
                var raw = localStorage.getItem("LOCAL_MCP_SERVER");
                if (raw && raw.length > 1000000) return;
                var qwenCoreEntry = {
                    id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                    name: "qwen-core",
                    description: "Essential tools for file operations, search, and bash execution.",
                    type: "stdio",
                    params: { command: "npx", args: ["-y", "qwen-core"] },
                    enabled: true,
                    default: false,
                    connectionStatus: "available",
                    errorMessage: "",
                    tools: []
                };
                var existing = null;
                try {
                    if (raw) { existing = JSON.parse(raw); if (!Array.isArray(existing)) existing = null; }
                } catch(e) { existing = null; }
                if (existing && existing.length > 0) {
                    var hasQwenCore = false;
                    for (var i = 0; i < existing.length; i++) {
                        if (existing[i].name === "qwen-core") {
                            hasQwenCore = true;
                            qwenCoreEntry.enabled = existing[i].enabled !== false;
                            existing[i] = qwenCoreEntry;
                            break;
                        }
                    }
                    if (!hasQwenCore) existing.unshift(qwenCoreEntry);
                    localStorage.setItem("LOCAL_MCP_SERVER", JSON.stringify(existing));
                } else {
                    localStorage.setItem("LOCAL_MCP_SERVER", JSON.stringify([qwenCoreEntry]));
                }
            } catch(e) { if(window.__QWEN_DEBUG) console.error("[PreLoad] MCP config injection failed:", e); }
        })();
    "#;

<<<<<<< HEAD
            let debug_flag = if cfg!(debug_assertions) {
                "window.__QWEN_DEBUG = true;"
            } else {
                "window.__QWEN_DEBUG = false;"
            };

            let modules = [
                debug_flag,
                pre_load_script,
                include_str!("core_bridge.js"),
                include_str!("platform_bridge.js"),
                include_str!("settings_injector.js"),
            ];
            modules.join("\n\n")
        })
        .clone()
}

/// Lightweight init script for profile picker (only core bridge, no clipboard/zoom/settings)
pub fn build_picker_init_script() -> String {
    PICKER_INIT_SCRIPT
        .get_or_init(|| {
            let debug_flag = if cfg!(debug_assertions) {
                "window.__QWEN_DEBUG = true;"
            } else {
                "window.__QWEN_DEBUG = false;"
            };
            [debug_flag, include_str!("core_bridge.js")].join("\n\n")
        })
        .clone()
=======
    let debug_flag = if cfg!(debug_assertions) {
        "window.__QWEN_DEBUG = true;"
    } else {
        "window.__QWEN_DEBUG = false;"
    };

    let modules = [
        debug_flag,
        pre_load_script,
        include_str!("core_bridge.js"),
        include_str!("platform_bridge.js"),
        include_str!("settings_injector.js"),
    ];
    modules.join("\n\n")
>>>>>>> 0f81055 (Melhorias)
}
