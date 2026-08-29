use super::schema::McpServerConfig;
use std::collections::HashMap;

pub fn mcp_servers() -> HashMap<String, McpServerConfig> {
    let mut config = HashMap::new();
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".into());
    let projects = format!("{}/Projects", home);

    config.insert(
        "qwen-core".into(),
        McpServerConfig {
            command: "npx".into(),
            args: vec!["-y".into(), "qwen-core".into()],
            transport_type: Some("stdio".into()),
            source: Some("official".into()),
            from: Some("builtin".into()),
            disabled: false,
            ..Default::default()
        },
    );

<<<<<<< HEAD
    // Restrict Filesystem to Documents + Projects + /tmp by default (not whole home)
    let documents = format!("{}/Documents", home);
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
    config.insert(
        "Filesystem".into(),
        McpServerConfig {
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-filesystem".into(),
<<<<<<< HEAD
                documents,
                projects,
                "/tmp".into(),
=======
                home,
                "/tmp".into(),
                projects,
>>>>>>> c0c2f30 (Fix: Upload medias e username)
            ],
            transport_type: Some("stdio".into()),
            ..Default::default()
        },
    );

    config.insert(
        "Sequential-Thinking".into(),
        McpServerConfig {
            command: "npx".into(),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-sequential-thinking".into(),
            ],
            transport_type: Some("stdio".into()),
            ..Default::default()
        },
    );

    config
}

pub fn normalize_mcp(
    mut config: HashMap<String, McpServerConfig>,
) -> HashMap<String, McpServerConfig> {
    if let Some(fs_config) = config.get_mut("Filesystem") {
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp".into());
        let documents = format!("{}/Documents", home);
        let projects = format!("{}/Projects", home);

        fs_config.args = fs_config
            .args
            .iter()
            .map(|arg| {
                if arg == "/Users" || arg.starts_with("/Users/") {
                    home.clone()
                } else {
                    arg.clone()
                }
            })
            .collect();

<<<<<<< HEAD
        // Ensure at least one safe directory remains; don't auto-re-add home if user removed it
        let has_safe_dir = fs_config.args.iter().any(|a| {
            a == &documents || a == &projects || a == "/tmp" || a == &home
        });
        if !has_safe_dir {
            // User removed all safe dirs — restore documents as minimal safe default
            fs_config.args.push(documents);
=======
        if !fs_config.args.iter().any(|a| a == &home) {
            fs_config.args.push(home);
        }
        if !fs_config.args.iter().any(|a| a == &projects) {
            fs_config.args.push(projects);
        }
        if !fs_config.args.iter().any(|a| a == "/tmp") {
            fs_config.args.push("/tmp".into());
>>>>>>> c0c2f30 (Fix: Upload medias e username)
        }
    }
    config
}
