use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

use crate::ai::AiProvider;
use crate::config::UserConfig;
use crate::db::Db;
use crate::error::{AppError, Result};

pub mod cluster;
pub mod collect;
pub mod search;
pub mod structure;
pub mod synthesis;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Search,
    Collect,
    Structure,
    Cluster,
    Synthesis,
    Ready,
}

impl Stage {
    fn as_str(&self) -> &'static str {
        match self {
            Stage::Search => "search",
            Stage::Collect => "collect",
            Stage::Structure => "structure",
            Stage::Cluster => "cluster",
            Stage::Synthesis => "synthesis",
            Stage::Ready => "ready",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Started,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub book_id: i64,
    pub stage: &'static str,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone)]
pub struct PipelineCtx {
    pub app: AppHandle,
    pub db: Arc<Db>,
    pub http: reqwest::Client,
    pub ai: Option<Arc<dyn AiProvider>>,
    pub config: UserConfig,
}

impl PipelineCtx {
    pub fn emit(&self, book_id: i64, stage: Stage, status: StageStatus, message: Option<String>) {
        let evt = ProgressEvent {
            book_id,
            stage: stage.as_str(),
            status: match status {
                StageStatus::Started => "started",
                StageStatus::Done => "done",
                StageStatus::Failed => "failed",
            },
            message,
        };
        if let Err(e) = self.app.emit("book.progress", &evt) {
            error!(?e, "failed to emit book.progress");
        }
    }
}

/// 한 책에 대해 6단계 파이프라인을 순차 실행한다.
pub async fn run(ctx: PipelineCtx, book_id: i64) {
    info!(book_id, "pipeline start");

    let result = run_inner(&ctx, book_id).await;

    match result {
        Ok(()) => {
            info!(book_id, "pipeline complete");
            let db = ctx.db.clone();
            let _ = tokio::task::spawn_blocking(move || {
                db.with(|conn| crate::db::repo::update_status(conn, book_id, "ready", None))
            })
            .await;
            ctx.emit(book_id, Stage::Ready, StageStatus::Done, None);
            let _ = ctx.app.emit("book.ready", &serde_json::json!({ "book_id": book_id }));
        }
        Err(e) => {
            error!(book_id, ?e, "pipeline failed");
            let msg = e.to_string();
            let db = ctx.db.clone();
            let m = msg.clone();
            let _ = tokio::task::spawn_blocking(move || {
                db.with(|conn| crate::db::repo::update_status(conn, book_id, "failed", Some(&m)))
            })
            .await;
            let _ = ctx.app.emit(
                "book.failed",
                &serde_json::json!({ "book_id": book_id, "error": msg }),
            );
        }
    }
}

async fn run_inner(ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    // Stage 1: Search (메타데이터 추정)
    ctx.emit(book_id, Stage::Search, StageStatus::Started, None);
    search::run(ctx, book_id).await.map_err(|e| stage_err("search", e))?;
    ctx.emit(book_id, Stage::Search, StageStatus::Done, None);

    // Stage 2: Collect (Tavily + scrape)
    ctx.emit(book_id, Stage::Collect, StageStatus::Started, None);
    collect::run(ctx, book_id).await.map_err(|e| stage_err("collect", e))?;
    ctx.emit(book_id, Stage::Collect, StageStatus::Done, None);

    // Stage 3: Structure (stub)
    ctx.emit(book_id, Stage::Structure, StageStatus::Started, None);
    structure::run(ctx, book_id).await.map_err(|e| stage_err("structure", e))?;
    ctx.emit(book_id, Stage::Structure, StageStatus::Done, None);

    // Stage 4: Cluster (stub)
    ctx.emit(book_id, Stage::Cluster, StageStatus::Started, None);
    cluster::run(ctx, book_id).await.map_err(|e| stage_err("cluster", e))?;
    ctx.emit(book_id, Stage::Cluster, StageStatus::Done, None);

    // Stage 5: Synthesis (stub)
    ctx.emit(book_id, Stage::Synthesis, StageStatus::Started, None);
    synthesis::run(ctx, book_id).await.map_err(|e| stage_err("synthesis", e))?;
    ctx.emit(book_id, Stage::Synthesis, StageStatus::Done, None);

    Ok(())
}

fn stage_err(stage: &str, e: AppError) -> AppError {
    AppError::Pipeline {
        stage: stage.to_string(),
        message: e.to_string(),
    }
}
