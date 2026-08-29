use tauri_plugin_dialog::DialogExt;

use crate::config::schema::{ChatExport, ChatMessage};

#[tauri::command]
pub async fn export_chat(
    app: tauri::AppHandle,
    title: String,
    messages: Vec<ChatMessage>,
    format: String,
) -> Result<String, String> {
    let now = timestamp_now();
    let export = ChatExport {
        title: title.clone(),
        messages,
        exported_at: now,
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

    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_title("Export Chat")
        .add_filter(filter_name, &[filter_ext])
        .set_file_name(&default_name)
        .save_file(move |f| {
            let _ = tx.send(f.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())));
        });

    let path = rx
        .recv()
        .map_err(|e| e.to_string())?
        .ok_or("No file selected")?;
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
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
            self.title,
            self.title,
            self.exported_at,
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
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .replace(' ', "_")
}

fn timestamp_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
