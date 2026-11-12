use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};

const FILE_NAME: &str = "settings.json";
const LAST_DEVICE_FILE: &str = "last_device.txt";

fn default_api_key() -> String {
    option_env!("DEFAULT_OPENROUTER_API_KEY")
        .unwrap_or_default()
        .to_string()
}

fn default_transcription_model() -> String {
    option_env!("DEFAULT_MODELO_TRANSCRICAO")
        .unwrap_or("openai/gpt-4o-transcribe")
        .to_string()
}

fn default_ai_model() -> String {
    option_env!("DEFAULT_MODELO_IA")
        .unwrap_or("google/gemini-2.5-flash")
        .to_string()
}

fn default_tts_model() -> String {
    option_env!("DEFAULT_MODELO_VOZ")
        .unwrap_or("elevenlabs/eleven-flash-v2.5")
        .to_string()
}

fn default_tts_instructions() -> String {
    String::new()
}

fn default_headphone_volume() -> u8 {
    80
}

fn default_tts_voice() -> String {
    option_env!("DEFAULT_VOZ").unwrap_or("sarah").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_api_key")]
    pub openrouter_api_key: String,
    #[serde(default = "default_transcription_model")]
    pub transcription_model: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    #[serde(default = "default_tts_model")]
    pub tts_model: String,
    #[serde(default = "default_tts_voice")]
    pub tts_voice: String,
    #[serde(default = "default_tts_instructions")]
    pub tts_instructions: String,
    #[serde(default = "default_headphone_volume")]
    pub headphone_volume: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            openrouter_api_key: default_api_key(),
            transcription_model: default_transcription_model(),
            ai_model: default_ai_model(),
            tts_model: default_tts_model(),
            tts_voice: default_tts_voice(),
            tts_instructions: default_tts_instructions(),
            headphone_volume: default_headphone_volume(),
        }
    }
}

fn settings_path<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(FILE_NAME))
}

pub fn load<R: Runtime>(app: &tauri::AppHandle<R>) -> Settings {
    let Ok(path) = settings_path(app) else {
        return Settings::default();
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

pub fn save<R: Runtime>(app: &tauri::AppHandle<R>, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let contents = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, contents).map_err(|e| e.to_string())
}

pub fn recordings_dir<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("gravacoes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn last_device_path<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(LAST_DEVICE_FILE))
}

pub fn load_last_device<R: Runtime>(app: &tauri::AppHandle<R>) -> Option<String> {
    let contents = fs::read_to_string(last_device_path(app).ok()?).ok()?;
    let id = contents.trim();
    (!id.is_empty()).then(|| id.to_string())
}

pub fn save_last_device<R: Runtime>(app: &tauri::AppHandle<R>, id: &str) -> Result<(), String> {
    let path = last_device_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, id).map_err(|e| e.to_string())
}
