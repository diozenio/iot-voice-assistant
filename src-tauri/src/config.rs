use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};

const FILE_NAME: &str = "settings.json";

fn default_port() -> u16 {
    option_env!("DEFAULT_PORT")
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000)
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_api_key")]
    pub openrouter_api_key: String,
    #[serde(default = "default_transcription_model")]
    pub transcription_model: String,
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: default_port(),
            openrouter_api_key: default_api_key(),
            transcription_model: default_transcription_model(),
            ai_model: default_ai_model(),
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
