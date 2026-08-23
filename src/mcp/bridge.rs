use anyhow::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tauri::Manager;

type PendingMap = HashMap<u64, tokio::sync::oneshot::Sender<Result<serde_json::Value>>>;

pub struct Bridge {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    request_id: Arc<AtomicU64>,
    pending: Arc<Mutex<PendingMap>>,
    _child: tokio::process::Child,
}

impl Bridge {
    pub async fn new(app: Option<&tauri::AppHandle>) -> Result<Self> {
        let bridge_path = resolve_bridge_path(app)?;

        let mut child = Command::new("node")
            .arg(&bridge_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        log::info!("[Bridge] Spawned PID {}", child.id().unwrap_or(0));

        let pending: Arc<Mutex<PendingMap>> = Arc::new(Mutex::new(HashMap::new()));
        let request_id = Arc::new(AtomicU64::new(0));

        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| anyhow::anyhow!("No stdin"))?));
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("No stdout"))?;

        let pending_read = Arc::clone(&pending);
        tokio::spawn(Self::read_loop(stdout, pending_read));

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(Self::stderr_loop(stderr));
        }

        Ok(Self {
            stdin,
            request_id,
            pending,
            _child: child,
        })
    }

    pub async fn send(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        log::debug!("[Bridge] send #{} method={}", id, method);
        let msg = serde_json::json!({ "id": id, "method": method, "params": params });
        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');

        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            self.pending.lock().await.insert(id, tx);
        }
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await?;
            stdin.flush().await?;
        }

        match tokio::time::timeout(Duration::from_secs(60), rx).await {
            Ok(Ok(result)) => {
                log::debug!("[Bridge] recv #{} method={}", id, method);
                result.map_err(|e| anyhow::anyhow!("{}", e))
            }
            Ok(Err(_)) => Err(anyhow::anyhow!("Channel closed")),
            Err(_) => {
                log::warn!("[Bridge] TIMEOUT #{} method={}", id, method);
                Err(anyhow::anyhow!("Timeout"))
            }
        }
    }

    pub async fn shutdown(&self) {
        let mut pending = self.pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(anyhow::anyhow!("Bridge closed")));
        }
    }

    async fn read_loop(
        stdout: tokio::process::ChildStdout,
        pending: Arc<Mutex<PendingMap>>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    Self::drain_pending(&pending).await;
                    return;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        Self::dispatch_response(&pending, &msg).await;
                    }
                }
                Err(_) => {
                    Self::drain_pending(&pending).await;
                    return;
                }
            }
        }
    }

    async fn dispatch_response(pending: &Arc<Mutex<PendingMap>>, msg: &serde_json::Value) {
        let id = msg.get("id").and_then(|v| v.as_u64());
        let id = match id {
            Some(i) => i,
            None => return,
        };
        let mut pending = pending.lock().await;
        if let Some(tx) = pending.remove(&id) {
            if let Some(result) = msg.get("result") {
                let _ = tx.send(Ok(result.clone()));
            } else if let Some(error) = msg.get("error") {
                let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("unknown");
                let _ = tx.send(Err(anyhow::anyhow!("{}", msg)));
            }
        } else {
            log::debug!("[Bridge] dispatch #{} no pending waiter (stale/duplicate)", id);
        }
    }

    async fn drain_pending(pending: &Arc<Mutex<PendingMap>>) {
        let mut pending = pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(anyhow::anyhow!("Bridge died")));
        }
    }

    async fn stderr_loop(stderr: tokio::process::ChildStderr) {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    let t = line.trim();
                    if !t.is_empty() {
                        log::debug!("[Bridge] {}", t);
                    }
                }
                Err(_) => break,
            }
        }
    }
}

fn resolve_bridge_path(app: Option<&tauri::AppHandle>) -> Result<std::path::PathBuf> {
    if let Some(app) = app {
        if let Ok(dir) = app.path().resource_dir() {
            let p = dir.join("mcp-bridge.mjs");
            if p.exists() {
                return Ok(p);
            }
        }
    }

    let manifest = std::path::PathBuf::from(
        concat!(env!("CARGO_MANIFEST_DIR"), "/", "mcp-bridge.mjs")
    );
    if manifest.exists() {
        return Ok(manifest);
    }

    Err(anyhow::anyhow!("MCP bridge not found"))
}
