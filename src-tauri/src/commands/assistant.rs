use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::{
    audio::pcm_to_wav,
    bluetooth,
    config::{self, Settings},
    presets::{self, VoicePreset},
    speech,
    state::{self, AppState, Level, Snapshot},
};

const PREVIEW_TEXT: &str = "Olá! Eu sou o seu assistente de voz. Como posso ajudar você hoje?";

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
pub fn list_voice_presets() -> &'static [VoicePreset] {
    presets::all()
}

#[tauri::command]
pub async fn preview_voice(
    app: AppHandle,
    model: String,
    voice: String,
    instructions: String,
) -> Result<tauri::ipc::Response, String> {
    let api_key = app
        .state::<AppState>()
        .settings
        .lock()
        .unwrap()
        .openrouter_api_key
        .clone();
    if api_key.is_empty() {
        return Err("Defina a chave da OpenRouter nas configurações".into());
    }
    let samples = speech::synthesize(
        &reqwest::Client::new(),
        &api_key,
        &model,
        &voice,
        &instructions,
        PREVIEW_TEXT,
    )
    .await?;
    let pcm: Vec<u8> = samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    Ok(tauri::ipc::Response::new(pcm_to_wav(&pcm)))
}

#[tauri::command]
pub async fn set_volume(app: AppHandle, volume: u8) -> Result<(), String> {
    let settings = {
        let state = app.state::<AppState>();
        let mut settings = state.settings.lock().unwrap();
        settings.headphone_volume = volume.min(100);
        settings.clone()
    };
    config::save(&app, &settings)?;
    bluetooth::send_volume(&app, settings.headphone_volume).await
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
