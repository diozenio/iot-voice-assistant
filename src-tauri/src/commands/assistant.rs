use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    config::{self, Settings},
    speech,
    state::{self, AppState, Level, Snapshot},
};

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> Snapshot {
    app.state::<AppState>().snapshot()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    config::save(&app, &settings)?;

    *app.state::<AppState>().settings.lock().unwrap() = settings;
    state::log(&app, Level::Success, "Configurações salvas");

    Ok(())
}

#[tauri::command]
pub async fn list_voices(model: String) -> Result<Vec<String>, String> {
    speech::voices(&reqwest::Client::new(), &model).await
}

#[tauri::command]
pub fn clear_logs(app: AppHandle) {
    app.state::<AppState>().logs.lock().unwrap().clear();
}

#[tauri::command]
pub fn open_recordings_folder(app: AppHandle) -> Result<(), String> {
    let dir = config::recordings_dir(&app)?;
    app.opener()
        .open_path(dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_recording(app: AppHandle, path: String) -> Result<tauri::ipc::Response, String> {
    let dir = config::recordings_dir(&app)?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let file = std::path::Path::new(&path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !file.starts_with(&dir) {
        return Err("Arquivo fora da pasta de gravações".into());
    }
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}
