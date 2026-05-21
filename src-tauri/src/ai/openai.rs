// 다음 세션에서 구현 예정. ROADMAP 8단계 (provider 전환).
// trait 시그니처는 유지하지만 모든 호출이 ProviderMissing을 반환한다.

use async_trait::async_trait;

use super::{AiProvider, Prompt};
use crate::error::{AppError, Result};

#[allow(dead_code)]
pub struct OpenAiProvider;

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn complete(&self, _prompt: &Prompt) -> Result<String> {
        Err(AppError::ProviderMissing("openai not implemented yet".into()))
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Err(AppError::ProviderMissing("openai not implemented yet".into()))
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn context_window(&self) -> usize {
        128_000
    }
}
