use std::panic;
use std::path::PathBuf;

pub fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        log_crash_to_file(info);
        default_hook(info);
    }));
}

fn log_crash_to_file(info: &panic::PanicHookInfo) {
    let log_dir = get_crash_log_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let log_file = log_dir.join(format!("crash-{}.log", timestamp));

    // Payload can be &str or String — handle both.
    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic (non-string payload)".to_string()
    };
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".to_string());

    // Always capture backtrace — force_capture gives useful output even without RUST_BACKTRACE.
    let backtrace = std::backtrace::Backtrace::force_capture();
    let bt_str = format!("{}", backtrace);
    let backtrace_text = if bt_str.contains("disabled") || bt_str.trim().is_empty() {
        "Backtrace unavailable (build without debuginfo)".to_string()
    } else {
        bt_str
    };

    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("<unnamed>");

    let content = format!(
        "=== Qwem Studio Linux Crash Report ===\n\
         Time: {}\nVersion: {}\nPlatform: {} {}\nThread: {}\n\n\
         Panic: {}\nLocation: {}\n\nBacktrace:\n{}\n",
        timestamp,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        thread_name,
        message,
        location,
        backtrace_text
    );

    // Best-effort write + also log to stderr so coredumpctl/journald captures it.
    if let Err(e) = std::fs::write(&log_file, &content) {
        eprintln!("[Crash] Failed to write log {}: {}", log_file.display(), e);
    }
    eprintln!("{}", content);
    log::error!(
        "[Crash] Panic at {}: {} — logged to {}",
        location,
        message,
        log_file.display()
    );
    log::error!("[Crash] Backtrace:\n{}", backtrace_text);
}

fn get_crash_log_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qwen-studio-linux")
        .join("crash-logs");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn list_crash_logs() -> Vec<String> {
    let dir = get_crash_log_dir();
    let mut logs: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    logs.sort_unstable();
    logs.reverse();
    logs
}

pub fn read_crash_log(filename: &str) -> Result<String, String> {
    let path = get_crash_log_dir().join(filename);
    if !path.exists() {
        return Err("Crash log not found".into());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}
