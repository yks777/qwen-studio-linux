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

    let message = info.payload().downcast_ref::<&str>()
        .unwrap_or(&"Unknown panic")
        .to_string();
    let location = info.location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown".to_string());

    let backtrace = if std::env::var("RUST_BACKTRACE").is_ok() {
        format!("{}", std::backtrace::Backtrace::capture())
    } else {
        "Set RUST_BACKTRACE=1 for full backtrace".to_string()
    };

    let content = format!(
        "=== Qwem Studio Linux Crash Report ===\n\
         Time: {}\nVersion: {}\nPlatform: {} {}\n\n\
         Panic: {}\nLocation: {}\n\nBacktrace:\n{}\n",
        timestamp, env!("CARGO_PKG_VERSION"),
        std::env::consts::OS, std::env::consts::ARCH,
        message, location, backtrace
    );

    let _ = std::fs::write(&log_file, content);
    log::error!("[Crash] Logged to: {}", log_file.display());
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
        .filter(|e| e.path().extension().map(|ext| ext == "log").unwrap_or(false))
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
