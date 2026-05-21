use serde::{Serialize, Serializer};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("db: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("http: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("ai provider missing: {0}")]
    ProviderMissing(String),

    #[error("config: {0}")]
    Config(String),

    #[error("pipeline {stage}: {message}")]
    Pipeline { stage: String, message: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind = match self {
            AppError::Db(_) => "db",
            AppError::Http(_) => "http",
            AppError::Io(_) => "io",
            AppError::Json(_) => "json",
            AppError::Tauri(_) => "tauri",
            AppError::ProviderMissing(_) => "provider_missing",
            AppError::Config(_) => "config",
            AppError::Pipeline { .. } => "pipeline",
            AppError::NotFound(_) => "not_found",
            AppError::Other(_) => "other",
        };
        let mut map = std::collections::BTreeMap::new();
        map.insert("kind", kind.to_string());
        map.insert("message", self.to_string());
        if let AppError::Pipeline { stage, .. } = self {
            map.insert("stage", stage.clone());
        }
        map.serialize(s)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}
