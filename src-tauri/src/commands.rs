use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ai::{build_provider, Prompt};
use crate::config::{ProviderKind, UserConfig};
use crate::db::repo::{self, Book, Chapter, Source};
use crate::error::{AppError, Result};
use crate::pipeline::{self, PipelineCtx};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SettingsView {
    pub provider: ProviderKind,
    pub model: String,
    pub language: String,
    pub has_gemini_key: bool,
    pub has_openai_key: bool,
    pub has_tavily_key: bool,
    pub ollama_endpoint: Option<String>,
}

impl From<&UserConfig> for SettingsView {
    fn from(c: &UserConfig) -> Self {
        Self {
            provider: c.provider,
            model: c.model.clone(),
            language: c.language.clone(),
            has_gemini_key: c.gemini_key().is_some(),
            has_openai_key: c.openai_api_key.as_deref().map(|s| !s.is_empty()).unwrap_or(false),
            has_tavily_key: c.tavily_key().is_some(),
            ollama_endpoint: c.ollama_endpoint.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SettingsUpdate {
    pub provider: Option<ProviderKind>,
    pub model: Option<String>,
    pub language: Option<String>,
    pub gemini_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub tavily_api_key: Option<String>,
    pub ollama_endpoint: Option<String>,
}

#[tauri::command]
pub async fn start_book(
    title: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Book> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Other("title is empty".into()));
    }

    // 동일 제목 있으면 그것을 반환 (UC-01 alt flow). 재합성은 다음 세션.
    let existing = {
        let db = state.db.clone();
        let t = title.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::find_book_by_title(c, &t)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
    };
    if let Some(book) = existing {
        return Ok(book);
    }

    let book = {
        let db = state.db.clone();
        let t = title.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::insert_book(c, &t)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
    };

    // 파이프라인을 백그라운드에서 실행. start_book은 즉시 반환.
    let cfg = state.config.lock().await.clone();
    let ai = state.ai.lock().await.clone();
    let ctx = PipelineCtx {
        app: app.clone(),
        db: state.db.clone(),
        http: state.http.clone(),
        ai,
        config: cfg,
    };
    let book_id = book.id;
    tauri::async_runtime::spawn(async move {
        pipeline::run(ctx, book_id).await;
    });

    Ok(book)
}

#[tauri::command]
pub async fn list_books(state: State<'_, AppState>) -> Result<Vec<Book>> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.with(|c| repo::list_books(c)))
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn get_book(id: i64, state: State<'_, AppState>) -> Result<Book> {
    let db = state.db.clone();
    let book = tokio::task::spawn_blocking(move || db.with(|c| repo::get_book(c, id)))
        .await
        .map_err(|e| AppError::Other(e.to_string()))??;
    book.ok_or_else(|| AppError::NotFound(format!("book {id}")))
}

#[tauri::command]
pub async fn get_sources(book_id: i64, state: State<'_, AppState>) -> Result<Vec<Source>> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.with(|c| repo::list_sources(c, book_id)))
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn get_chapters(book_id: i64, state: State<'_, AppState>) -> Result<Vec<Chapter>> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.with(|c| repo::list_chapters(c, book_id)))
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn get_chapter_sources(
    chapter_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Source>> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.with(|c| repo::list_chapter_sources(c, chapter_id)))
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<SettingsView> {
    let cfg = state.config.lock().await;
    Ok(SettingsView::from(&*cfg))
}

#[tauri::command]
pub async fn update_settings(
    patch: SettingsUpdate,
    state: State<'_, AppState>,
) -> Result<SettingsView> {
    let mut cfg = state.config.lock().await;
    if let Some(p) = patch.provider {
        cfg.provider = p;
    }
    if let Some(m) = patch.model {
        cfg.model = m;
    }
    if let Some(l) = patch.language {
        cfg.language = l;
    }
    if let Some(k) = patch.gemini_api_key {
        cfg.gemini_api_key = if k.is_empty() { None } else { Some(k) };
    }
    if let Some(k) = patch.openai_api_key {
        cfg.openai_api_key = if k.is_empty() { None } else { Some(k) };
    }
    if let Some(k) = patch.tavily_api_key {
        cfg.tavily_api_key = if k.is_empty() { None } else { Some(k) };
    }
    if let Some(e) = patch.ollama_endpoint {
        cfg.ollama_endpoint = if e.is_empty() { None } else { Some(e) };
    }

    crate::config::save(&cfg)?;

    // ai provider 재구성
    let new_ai: Option<Arc<dyn crate::ai::AiProvider>> = build_provider(&cfg).map(Arc::from);
    *state.ai.lock().await = new_ai;

    Ok(SettingsView::from(&*cfg))
}

#[derive(Debug, Serialize)]
pub struct TestProviderResult {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn test_provider(state: State<'_, AppState>) -> Result<TestProviderResult> {
    let ai = state.ai.lock().await.clone();
    let Some(ai) = ai else {
        return Ok(TestProviderResult {
            ok: false,
            message: "API key가 설정되지 않았습니다.".into(),
        });
    };
    let prompt = Prompt::new("Reply with the single word: ok").max_tokens(8);
    match ai.complete(&prompt).await {
        Ok(text) => Ok(TestProviderResult {
            ok: true,
            message: format!("response: {}", text.trim()),
        }),
        Err(e) => Ok(TestProviderResult {
            ok: false,
            message: e.to_string(),
        }),
    }
}
