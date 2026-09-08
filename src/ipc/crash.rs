#[tauri::command]
pub fn list_crash_logs() -> Vec<String> {
    crate::app::panic::list_crash_logs()
}

#[tauri::command]
pub fn read_crash_log(filename: String) -> Result<String, String> {
    crate::app::panic::read_crash_log(&filename)
}
