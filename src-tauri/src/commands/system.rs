use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn open_browser(app: tauri::AppHandle) -> Result<(), String> {
    app.opener()
        .open_url("https://www.google.com", None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_file_explorer(app: tauri::AppHandle) -> Result<(), String> {
    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    app.opener()
        .open_path(home.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}
