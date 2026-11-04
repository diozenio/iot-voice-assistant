use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    config::{self, Settings},
    server,
    state::{self, AppState, Level, Snapshot},
};

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> Snapshot {
    app.state::<AppState>().snapshot()
}

#[tauri::command]
pub async fn start_server(app: AppHandle) -> Result<(), String> {
    server::start(app).await
}

#[tauri::command]
pub fn stop_server(app: AppHandle) -> Result<(), String> {
    server::stop(&app)
}

#[tauri::command]
pub async fn restart_server(app: AppHandle) -> Result<(), String> {
    let running = app.state::<AppState>().status.lock().unwrap().running;
    if running {
        server::stop(&app)?;
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    server::start(app).await
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    config::save(&app, &settings)?;

    let running = {
        let app_state = app.state::<AppState>();
        *app_state.settings.lock().unwrap() = settings;
        app_state.status.lock().unwrap().running
    };

    state::log(&app, Level::Success, "Configurações salvas");
    if running {
        state::log(
            &app,
            Level::Warn,
            "Reinicie o servidor para aplicar as novas configurações",
        );
    }

    Ok(())
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
pub fn open_recording(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}
