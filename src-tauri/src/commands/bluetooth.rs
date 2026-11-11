use tauri::AppHandle;

use crate::bluetooth;

#[tauri::command]
pub async fn start_scan(app: AppHandle) -> Result<(), String> {
    bluetooth::start_scan(&app).await
}

#[tauri::command]
pub async fn stop_scan(app: AppHandle) -> Result<(), String> {
    bluetooth::stop_scan(&app).await
}

#[tauri::command]
pub async fn connect_device(app: AppHandle, id: String) -> Result<(), String> {
    bluetooth::connect(&app, id).await
}

#[tauri::command]
pub async fn disconnect_device(app: AppHandle) -> Result<(), String> {
    bluetooth::disconnect(&app).await
}
