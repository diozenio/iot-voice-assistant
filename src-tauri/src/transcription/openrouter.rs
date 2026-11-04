use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use super::TranscriptionAdapter;
use crate::audio::pcm_to_wav;

const TRANSCRIPTIONS: &str = "https://openrouter.ai/api/v1/audio/transcriptions";
const LANGUAGE: &str = "pt";

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: Option<String>,
}

pub struct OpenRouterTranscriptionAdapter {
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl OpenRouterTranscriptionAdapter {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TranscriptionAdapter for OpenRouterTranscriptionAdapter {
    async fn transcribe(&self, pcm: &[u8]) -> Result<String, String> {
        let file = Part::bytes(pcm_to_wav(pcm))
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| e.to_string())?;

        let form = Form::new()
            .part("file", file)
            .text("model", self.model.clone())
            .text("language", LANGUAGE);

        let response = self
            .http
            .post(TRANSCRIPTIONS)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("OpenRouter {}: {body}", status.as_u16()));
        }

        let data: TranscriptionResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(data.text.map(|text| text.trim().to_string()).unwrap_or_default())
    }
}
