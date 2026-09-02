<<<<<<< HEAD
<<<<<<< HEAD
pub use crate::js::build_init_script;
=======
pub fn build_init_script() -> String {
    let pre_load_script = r#"
        (function() {
            try { document.documentElement.style.backgroundColor = '#0f1115'; } catch (e) {}
            var hostname = window.location.hostname;
            var pathname = window.location.pathname;
            var isLoginPage = pathname.includes('login') || pathname.includes('auth') || pathname.includes('callback') || pathname.includes('oauth');
            if (hostname !== 'chat.qwen.ai' || isLoginPage) return;
            try {
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
                    var raw = localStorage.getItem("LOCAL_MCP_SERVER");
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
            } catch(e) { console.error("[PreLoad] MCP config injection failed:", e); }
        })();
    "#;

    let modules = [
        pre_load_script,
        include_str!("../js/core_bridge.js"),
        include_str!("../js/platform_bridge.js"),
        include_str!("../js/settings_injector.js"),
    ];
    modules.join("\n\n")
}
>>>>>>> ce2f600 (optimization)
=======
pub use crate::js::build_init_script;
>>>>>>> 0f81055 (Melhorias)
