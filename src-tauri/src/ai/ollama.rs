// 다음 세션에서 구현 예정. ROADMAP 8단계 (provider 전환).

use async_trait::async_trait;

use super::{AiProvider, Prompt};
use crate::error::{AppError, Result};

#[allow(dead_code)]
pub struct OllamaProvider;

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn complete(&self, _prompt: &Prompt) -> Result<String> {
        Err(AppError::ProviderMissing("ollama not implemented yet".into()))
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Err(AppError::ProviderMissing("ollama not implemented yet".into()))
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn context_window(&self) -> usize {
        32_000
    }
}
