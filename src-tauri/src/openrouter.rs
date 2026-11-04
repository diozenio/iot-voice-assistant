use serde::{Deserialize, Serialize};

pub const CHAT_COMPLETIONS: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<ResponseMessage>,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

pub async fn first_content(response: reqwest::Response) -> Result<String, String> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenRouter {}: {body}", status.as_u16()));
    }

    let data: ChatResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(data
        .choices
        .and_then(|choices| choices.into_iter().next())
        .and_then(|choice| choice.message)
        .and_then(|message| message.content)
        .map(|text| text.trim().to_string())
        .unwrap_or_default())
}

pub async fn complete(
    http: &reqwest::Client,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let response = http
        .post(CHAT_COMPLETIONS)
        .bearer_auth(api_key)
        .json(&serde_json::json!({ "model": model, "messages": messages }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    first_content(response).await
}
