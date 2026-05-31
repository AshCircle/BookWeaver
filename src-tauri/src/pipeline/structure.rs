use rusqlite::OptionalExtension;
use tracing::info;

use super::PipelineCtx;
use crate::db::repo;
use crate::error::{AppError, Result};

const CHUNK_CHARS: usize = 1600; // ~800 토큰 가정
const OVERLAP_CHARS: usize = 200;

/// Stage 3 stub: 소스의 clean_text를 단순 문자 단위로 잘라 chunks에 넣는다.
/// 헤딩 추출/LLM 잠정 목차는 다음 세션에서.
pub async fn run(ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    let sources = {
        let db = ctx.db.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::list_sources(c, book_id)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
    };

    let db = ctx.db.clone();
    let mut total_chunks = 0;
    for src in sources {
        let text_opt: Option<String> = {
            let db = db.clone();
            let src_id = src.id;
            tokio::task::spawn_blocking(move || -> Result<Option<String>> {
                db.with(|c| {
                    let r = c
                        .query_row(
                            "SELECT clean_text FROM sources WHERE id = ?1",
                            rusqlite::params![src_id],
                            |row| row.get::<_, Option<String>>(0),
                        )
                        .optional()?
                        .flatten();
                    Ok(r)
                })
            })
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
        };
        let Some(text) = text_opt else { continue };
        if text.trim().is_empty() {
            continue;
        }

        for chunk in chunk_text(&text) {
            let db = db.clone();
            let src_id = src.id;
            let chunk = chunk.to_string();
            tokio::task::spawn_blocking(move || -> Result<()> {
                db.with(|c| {
                    repo::insert_chunk(c, src_id, &chunk, Some("(미정)"), Some(chunk.chars().count() as i64))?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| AppError::Other(e.to_string()))??;
            total_chunks += 1;
        }
    }

    info!(book_id, total_chunks, "structure stub completed");
    Ok(())
}

fn chunk_text(text: &str) -> Vec<&str> {
    let bytes_len = text.len();
    if bytes_len <= CHUNK_CHARS {
        return vec![text];
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < bytes_len {
        let mut end = (start + CHUNK_CHARS).min(bytes_len);
        // utf-8 경계로 보정
        while end < bytes_len && !text.is_char_boundary(end) {
            end += 1;
        }
        out.push(&text[start..end]);
        if end == bytes_len {
            break;
        }
        let next = end.saturating_sub(OVERLAP_CHARS);
        let mut next = next;
        while next < bytes_len && !text.is_char_boundary(next) {
            next += 1;
        }
        start = next;
    }
    out
}
