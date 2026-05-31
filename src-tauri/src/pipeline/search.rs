use serde::Deserialize;
use tracing::warn;

use super::PipelineCtx;
use crate::ai::Prompt;
use crate::db::repo;
use crate::error::Result;

/// Stage 1: 책 row는 commands::start_book에서 이미 만들어졌으므로,
/// 여기서는 메타데이터(저자/언어)를 LLM에게 1회 추정시킨다.
pub async fn run(ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    let title = {
        let db = ctx.db.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::get_book(c, book_id)))
            .await
            .map_err(|e| crate::error::AppError::Other(e.to_string()))??
            .map(|b| b.title)
    };

    let Some(title) = title else {
        return Ok(());
    };

    // ai가 없거나 키가 없으면 메타데이터 추정만 건너뛰고 계속 진행.
    let Some(ai) = ctx.ai.clone() else {
        warn!("no ai provider configured; skipping metadata estimation");
        return Ok(());
    };

    let system = "당신은 책 메타데이터 추정기입니다. 답변은 반드시 JSON {\"author\": string?, \"language\": \"ko\"|\"en\"|null} 한 줄로만 합니다.";
    let user = format!("책 제목: {title}\n저자와 주요 언어를 추정하세요. 모르면 null.");
    let prompt = Prompt::new(user).with_system(system).max_tokens(200);

    let response = match ai.complete(&prompt).await {
        Ok(s) => s,
        Err(e) => {
            warn!(?e, "metadata estimation failed; continuing without it");
            return Ok(());
        }
    };

    if let Some(meta) = parse_metadata(&response) {
        let db = ctx.db.clone();
        let author = meta.author.clone();
        let language = meta.language.clone();
        tokio::task::spawn_blocking(move || {
            db.with(|c| repo::update_metadata(c, book_id, author.as_deref(), language.as_deref()))
        })
        .await
        .map_err(|e| crate::error::AppError::Other(e.to_string()))??;
    } else {
        warn!(response, "could not parse metadata json");
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct MetaJson {
    author: Option<String>,
    language: Option<String>,
}

fn parse_metadata(text: &str) -> Option<MetaJson> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    let slice = &text[start..=end];
    serde_json::from_str(slice).ok()
}
