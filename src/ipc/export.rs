use tauri_plugin_dialog::DialogExt;

use crate::config::schema::{ChatExport, ChatMessage};

#[tauri::command]
pub async fn export_chat(
    app: tauri::AppHandle,
    title: String,
    messages: Vec<ChatMessage>,
    format: String,
) -> Result<String, String> {
    if title.len() > 512 {
        return Err("Title too long".into());
    }
    if messages.len() > 100_000 {
        return Err("Too many messages".into());
    }
    // Content size cap: sum of message lengths
    let total_len: usize = messages.iter().map(|m| m.content.len()).sum();
    if total_len > 10 * 1024 * 1024 {
        return Err("Export too large (10 MB limit)".into());
    }
    let now = timestamp_now();
    let export = ChatExport {
        title: title.clone(),
        messages,
        exported_at: now,
<<<<<<< HEAD
<<<<<<< HEAD
=======
    };

    let (content, default_name, filter_name, filter_ext) = match format.as_str() {
        "json" => (
            export.to_json()?,
            format!("{}.json", sanitize(&title)),
            "JSON",
            "json",
        ),
        "html" => (
            export.to_html(),
            format!("{}.html", sanitize(&title)),
            "HTML",
            "html",
        ),
        _ => (
            export.to_markdown(),
            format!("{}.md", sanitize(&title)),
            "Markdown",
            "md",
        ),
>>>>>>> c0c2f30 (Fix: Upload medias e username)
    };

    let (content, default_name, filter_name, filter_ext) = match format.as_str() {
        "json" => (
            export.to_json()?,
            format!("{}.json", sanitize(&title)),
            "JSON",
            "json",
        ),
        "html" => (
            export.to_html(),
            format!("{}.html", sanitize(&title)),
            "HTML",
            "html",
        ),
        _ => (
            export.to_markdown(),
            format!("{}.md", sanitize(&title)),
            "Markdown",
            "md",
        ),
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
=======
    };

    let (content, default_name, filter_name, filter_ext) = match format.as_str() {
        "json" => (
            export.to_json()?,
            format!("{}.json", sanitize(&title)),
            "JSON",
            "json",
        ),
        "html" => (
            export.to_html(),
            format!("{}.html", sanitize(&title)),
            "HTML",
            "html",
        ),
        _ => (
            export.to_markdown(),
            format!("{}.md", sanitize(&title)),
            "Markdown",
            "md",
        ),
    };

<<<<<<< HEAD
    let (tx, rx) = std::sync::mpsc::channel();
<<<<<<< HEAD
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
    let (tx, rx) = tokio::sync::oneshot::channel();
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
    app.dialog()
        .file()
        .set_title("Export Chat")
        .add_filter(filter_name, &[filter_ext])
        .set_file_name(&default_name)
        .save_file(move |f| {
            let _ = tx.send(f.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())));
        });

    let path = rx
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No file selected")?;
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| e.to_string())?;
=======
=======
>>>>>>> c0c2f30 (Fix: Upload medias e username)
        .recv()
        .map_err(|e| e.to_string())?
        .ok_or("No file selected")?;
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No file selected")?;
    tokio::fs::write(&path, &content)
        .await
        .map_err(|e| e.to_string())?;
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
    Ok(path)
}

impl ChatExport {
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# {}\n\n*Exported at: {}*\n\n---\n\n",
            self.title, self.exported_at
        );
        for msg in &self.messages {
            let label = match msg.role.as_str() {
                "user" => "**You**",
                "assistant" => "**Qwen**",
                r => r,
            };
            md.push_str(&format!("### {}\n\n{}\n\n---\n\n", label, msg.content));
        }
        md
    }

    pub fn to_html(&self) -> String {
        format!(
            r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>{}</title>
        <style>body{{font-family:sans-serif;max-width:800px;margin:auto;padding:20px;background:#1a1a1a;color:#e0e0e0;}}</style>
        </head><body><h1>{}</h1><p>Exported: {}</p><hr>{}</body></html>"#,
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
            html_escape(&self.title),
            html_escape(&self.title),
            html_escape(&self.exported_at),
=======
            self.title,
            self.title,
            self.exported_at,
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
            html_escape(&self.title),
            html_escape(&self.title),
            html_escape(&self.exported_at),
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
            self.title,
            self.title,
            self.exported_at,
>>>>>>> c0c2f30 (Fix: Upload medias e username)
            self.messages
                .iter()
                .map(|m| {
                    let (cls, lbl) = match m.role.as_str() {
                        "user" => ("user", "You"),
                        _ => ("assistant", "Qwen"),
                    };
                    format!(
                        "<div class='{}'><b>{}</b><pre>{}</pre></div>",
                        cls,
                        lbl,
                        html_escape(&m.content)
                    )
                })
                .collect::<String>()
        )
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn sanitize(name: &str) -> String {
    let mut s: String = name
        .chars()
=======
}

fn sanitize(name: &str) -> String {
    name.chars()
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn sanitize(name: &str) -> String {
    let mut s: String = name
        .chars()
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
}

fn sanitize(name: &str) -> String {
    name.chars()
>>>>>>> c0c2f30 (Fix: Upload medias e username)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
        .replace(' ', "_");
    if s.is_empty() {
        s = "chat".to_string();
    }
    if s.len() > 100 {
        s.truncate(100);
    }
    // Avoid hidden files / dot only
    if s.starts_with('.') {
        s = format!("chat{}", s);
    }
    s
<<<<<<< HEAD
=======
        .replace(' ', "_")
>>>>>>> c0c2f30 (Fix: Upload medias e username)
=======
>>>>>>> f88f2ac (Otimiza performance e corrige menu Arch Wayland)
=======
        .replace(' ', "_")
>>>>>>> c0c2f30 (Fix: Upload medias e username)
}

fn timestamp_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
