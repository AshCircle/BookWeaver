use tracing::info;

use super::PipelineCtx;
use crate::error::Result;

/// Stage 4 stub. 실제 임베딩/KNN은 다음 세션에서.
pub async fn run(_ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    info!(book_id, "cluster stub: skipped (no embedding yet)");
    Ok(())
}
