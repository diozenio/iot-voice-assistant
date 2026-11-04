pub mod openrouter;

use std::sync::Arc;

use async_trait::async_trait;

use crate::config::Settings;
use openrouter::OpenRouterTranscriptionAdapter;

#[async_trait]
pub trait TranscriptionAdapter: Send + Sync {
    async fn transcribe(&self, pcm: &[u8]) -> Result<String, String>;
}

pub fn build_adapter(settings: &Settings) -> Result<Arc<dyn TranscriptionAdapter>, String> {
    if settings.openrouter_api_key.is_empty() {
        return Err("Defina a chave OPENROUTER_API_KEY nas configurações".into());
    }

    Ok(Arc::new(OpenRouterTranscriptionAdapter::new(
        settings.openrouter_api_key.clone(),
        settings.transcription_model.clone(),
    )))
}
