use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Gemini,
    OpenAI,
    Ollama,
}

impl Default for ProviderKind {
    fn default() -> Self {
        ProviderKind::Gemini
    }
}

/// 사용자 설정. 디스크에 평문 JSON으로 저장. API 키는 TODO: keyring으로 이전 예정.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub provider: ProviderKind,
    pub model: String,
    pub language: String, // "ko" | "en"
    #[serde(default)]
    pub gemini_api_key: Option<String>,
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub tavily_api_key: Option<String>,
    #[serde(default)]
    pub ollama_endpoint: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Gemini,
            model: "gemini-2.0-flash".to_string(),
            language: "ko".to_string(),
            gemini_api_key: None,
            openai_api_key: None,
            tavily_api_key: None,
            ollama_endpoint: Some("http://localhost:11434".to_string()),
        }
    }
}

impl UserConfig {
    pub fn gemini_key(&self) -> Option<String> {
        self.gemini_api_key
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("GEMINI_API_KEY").ok())
    }

    pub fn tavily_key(&self) -> Option<String> {
        self.tavily_api_key
            .clone()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("TAVILY_API_KEY").ok())
    }
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "bookweaver", "BookWeaver")
        .ok_or_else(|| AppError::Config("could not resolve project dirs".into()))?;
    let dir = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn data_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "bookweaver", "BookWeaver")
        .ok_or_else(|| AppError::Config("could not resolve project dirs".into()))?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load() -> Result<UserConfig> {
    let path = config_path()?;
    if !path.exists() {
        let cfg = UserConfig::default();
        save(&cfg)?;
        return Ok(cfg);
    }
    let bytes = fs::read(&path)?;
    let cfg: UserConfig = serde_json::from_slice(&bytes).unwrap_or_default();
    Ok(cfg)
}

pub fn save(cfg: &UserConfig) -> Result<()> {
    let path = config_path()?;
    let json = serde_json::to_vec_pretty(cfg)?;
    fs::write(&path, json)?;
    Ok(())
}
