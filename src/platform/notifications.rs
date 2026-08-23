#![allow(dead_code)]

pub fn show(title: &str, message: &str) {
    #[cfg(target_os = "linux")]
    {
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(message)
            .appname("Qwem Studio Linux")
            .show();
    }
    #[cfg(not(target_os = "linux"))]
    {
        log::info!("[Notification] {}: {}", title, message);
    }
}

pub fn show_update(version: &str, notes: &str) {
    let message = if notes.is_empty() {
        format!("Version {} is available", version)
    } else {
        format!("Version {} is available\n\n{}", version, notes)
    };
    show("Qwem Studio Linux Update", &message);
}
