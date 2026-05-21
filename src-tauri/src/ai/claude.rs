use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{AiProvider, Prompt};
use crate::error::{AppError, Result};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-opus-4-7";
const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";

#[derive(Debug)]
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let model = if model.trim().is_empty() {
            DEFAULT_MODEL.to_string()
        } else {
            model
        };
        Self {
            api_key,
            model,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("reqwest client"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<MessageBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum MessageBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize)]
struct UserMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    async fn complete(&self, prompt: &Prompt) -> Result<String> {
        let body = json!({
            "model": self.model,
            "max_tokens": prompt.max_tokens,
            "temperature": prompt.temperature,
            "system": prompt.system,
            "messages": [
                UserMessage { role: "user", content: &prompt.user }
            ],
        });
        let resp = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "anthropic api error {status}: {text}"
            )));
        }

        let parsed: MessagesResponse = resp.json().await?;
        let text = parsed
            .content
            .into_iter()
            .filter_map(|b| match b {
                MessageBlock::Text { text } => Some(text),
                MessageBlock::Other => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(text)
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Anthropic은 자체 임베딩 API가 없음 (Voyage 권장). 다음 세션에서 Voyage/OpenAI/Ollama로 라우팅.
        Err(AppError::ProviderMissing(
            "claude provider has no embedding endpoint; configure ollama or openai for embeddings".into(),
        ))
    }

    fn name(&self) -> &str {
        "claude"
    }

    fn context_window(&self) -> usize {
        200_000
    }
}
