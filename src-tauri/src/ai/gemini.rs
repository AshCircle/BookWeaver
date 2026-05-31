use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::{AiProvider, Prompt};
use crate::error::{AppError, Result};

const DEFAULT_MODEL: &str = "gemini-2.5-flash-lite";
const DEFAULT_EMBED_MODEL: &str = "text-embedding-004";
const API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

#[derive(Debug)]
pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
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
struct GenerateResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    #[serde(default)]
    content: Option<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(default)]
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct Part {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Embedding,
}

#[derive(Debug, Deserialize)]
struct Embedding {
    values: Vec<f32>,
}

#[async_trait]
impl AiProvider for GeminiProvider {
    async fn complete(&self, prompt: &Prompt) -> Result<String> {
        let mut body = json!({
            "contents": [
                { "role": "user", "parts": [ { "text": prompt.user } ] }
            ],
            "generationConfig": {
                "maxOutputTokens": prompt.max_tokens,
            }
        });
        if let Some(temp) = prompt.temperature {
            body["generationConfig"]["temperature"] = json!(temp);
        }
        if let Some(system) = &prompt.system {
            body["system_instruction"] = json!({ "parts": [ { "text": system } ] });
        }

        let url = format!("{API_BASE}/models/{}:generateContent", self.model);
        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!("gemini api error {status}: {text}")));
        }

        let parsed: GenerateResponse = resp.json().await?;
        let text = parsed
            .candidates
            .into_iter()
            .filter_map(|c| c.content)
            .flat_map(|c| c.parts)
            .filter_map(|p| p.text)
            .collect::<Vec<_>>()
            .join("");

        if text.trim().is_empty() {
            return Err(AppError::Other("gemini returned empty response".into()));
        }
        Ok(text)
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let url = format!("{API_BASE}/models/{DEFAULT_EMBED_MODEL}:embedContent");
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            let body = json!({
                "model": format!("models/{DEFAULT_EMBED_MODEL}"),
                "content": { "parts": [ { "text": text } ] }
            });
            let resp = self
                .client
                .post(&url)
                .header("x-goog-api-key", &self.api_key)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Other(format!("gemini embed error {status}: {body}")));
            }
            let parsed: EmbedResponse = resp.json().await?;
            out.push(parsed.embedding.values);
        }
        Ok(out)
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn context_window(&self) -> usize {
        1_000_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "live Gemini API; run with `cargo test -- --ignored` and GEMINI_API_KEY set"]
    async fn smoke_complete() {
        let key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let provider = GeminiProvider::new(key, String::new());
        let prompt = Prompt::new("Reply with exactly the word: ok").max_tokens(16);
        let out = provider.complete(&prompt).await.expect("gemini complete failed");
        println!("[gemini] -> {out:?}");
        assert!(!out.trim().is_empty());
    }
}
