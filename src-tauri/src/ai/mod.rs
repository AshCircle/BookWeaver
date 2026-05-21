use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::{ProviderKind, UserConfig};
use crate::error::Result;

pub mod claude;
pub mod ollama;
pub mod openai;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub system: Option<String>,
    pub user: String,
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: Option<f32>,
}

impl Prompt {
    pub fn new(user: impl Into<String>) -> Self {
        Self {
            system: None,
            user: user.into(),
            max_tokens: 1024,
            temperature: None,
        }
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = n;
        self
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, prompt: &Prompt) -> Result<String>;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn name(&self) -> &str;
    fn context_window(&self) -> usize;
}

/// 사용자 config로부터 provider를 만든다. 키 누락 시 None.
pub fn build_provider(cfg: &UserConfig) -> Option<Box<dyn AiProvider>> {
    match cfg.provider {
        ProviderKind::Claude => {
            let key = cfg.anthropic_key()?;
            Some(Box::new(claude::ClaudeProvider::new(key, cfg.model.clone())))
        }
        ProviderKind::OpenAI => None, // TODO: 다음 세션
        ProviderKind::Ollama => None, // TODO: 다음 세션
    }
}
