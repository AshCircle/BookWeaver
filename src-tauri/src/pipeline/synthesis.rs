use rusqlite::OptionalExtension;
use tracing::info;

use super::PipelineCtx;
use crate::db::repo;
use crate::error::{AppError, Result};

const SUMMARY_PREVIEW_CHARS: usize = 1200;

/// Stage 5 stub: 진짜 합성 대신, 수집된 첫 source의 clean_text를 잘라
/// "TODO: synthesis 미구현" 안내가 붙은 single chapter를 만든다.
/// chapter_sources에는 책의 모든 sources를 연결한다.
pub async fn run(ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    let sources = {
        let db = ctx.db.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::list_sources(c, book_id)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
    };

    if sources.is_empty() {
        info!(book_id, "synthesis stub: no sources, nothing to do");
        return Ok(());
    }

    // 첫 source의 clean_text 미리보기를 summary_med에 박는다.
    let preview = {
        let db = ctx.db.clone();
        let src_id = sources[0].id;
        tokio::task::spawn_blocking(move || -> Result<String> {
            db.with(|c| {
                let txt: Option<String> = c
                    .query_row(
                        "SELECT clean_text FROM sources WHERE id = ?1",
                        rusqlite::params![src_id],
                        |row| row.get(0),
                    )
                    .optional()?
                    .flatten();
                Ok(txt.unwrap_or_default())
            })
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))??
    };
    let preview = if preview.chars().count() > SUMMARY_PREVIEW_CHARS {
        preview.chars().take(SUMMARY_PREVIEW_CHARS).collect::<String>() + "…"
    } else {
        preview
    };

    let med = format!(
        "[TODO: synthesis 미구현 — 수집된 첫 소스의 미리보기]\n\n{preview}"
    );

    let db = ctx.db.clone();
    let sources_for_link = sources.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        db.with(|c| {
            // 기존 챕터 제거 (idempotent)
            repo::delete_chapters_for_book(c, book_id)?;
            let chapter_id = repo::insert_chapter(
                c,
                book_id,
                0,
                "(임시) 챕터 1",
                Some("[TODO: synthesis 미구현]"),
                Some(&med),
                Some("[TODO: synthesis 미구현]"),
            )?;
            for src in sources_for_link {
                repo::link_chapter_source(c, chapter_id, src.id)?;
            }
            Ok(())
        })
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))??;

    Ok(())
}
