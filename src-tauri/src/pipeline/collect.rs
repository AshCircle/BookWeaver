use std::collections::HashSet;

use tracing::{info, warn};

use super::PipelineCtx;
use crate::db::repo;
use crate::error::{AppError, Result};
use crate::scrape::fetch_and_clean;
use crate::search::{query_variants, tavily::TavilyClient, SearchHit, SearchProvider};

const MAX_SOURCES: usize = 8;
const PER_QUERY: u32 = 6;

pub async fn run(ctx: &PipelineCtx, book_id: i64) -> Result<()> {
    // 책 정보 조회
    let book = {
        let db = ctx.db.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::get_book(c, book_id)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
            .ok_or_else(|| AppError::NotFound(format!("book {book_id}")))?
    };

    // Tavily 키 확인
    let key = ctx
        .config
        .tavily_key()
        .ok_or_else(|| AppError::Config("TAVILY_API_KEY not set".into()))?;
    let tavily = TavilyClient::new(key, ctx.http.clone());

    // 쿼리 변형
    let variants = query_variants(&book.title, book.language.as_deref());
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut hits: Vec<SearchHit> = Vec::new();

    for q in &variants {
        match tavily.search(q, PER_QUERY).await {
            Ok(results) => {
                for hit in results {
                    if seen_urls.insert(hit.url.clone()) {
                        hits.push(hit);
                        if hits.len() >= MAX_SOURCES {
                            break;
                        }
                    }
                }
                if hits.len() >= MAX_SOURCES {
                    break;
                }
            }
            Err(e) => warn!(?e, query = %q, "tavily query failed"),
        }
    }

    if hits.is_empty() {
        return Err(AppError::Other("no search results from tavily".into()));
    }

    info!(book_id, count = hits.len(), "scraping candidates");

    // 스크래핑 + DB 적재
    for hit in hits {
        match fetch_and_clean(&ctx.http, &hit.url).await {
            Ok(doc) => {
                if doc.clean_text.trim().len() < 200 {
                    info!(url = %doc.url, "skipping: clean text too short");
                    continue;
                }
                let db = ctx.db.clone();
                let title = doc.title.clone().or(hit.title.clone());
                let site = doc.site.clone();
                let raw = doc.raw_html.clone();
                let clean = doc.clean_text.clone();
                let url = doc.url.clone();
                tokio::task::spawn_blocking(move || {
                    db.with(|c| {
                        repo::insert_source(
                            c,
                            book_id,
                            &url,
                            title.as_deref(),
                            site.as_deref(),
                            Some(&raw),
                            Some(&clean),
                        )
                        .map(|_| ())
                    })
                })
                .await
                .map_err(|e| AppError::Other(e.to_string()))??;
            }
            Err(e) => warn!(url = %hit.url, ?e, "scrape failed"),
        }
    }

    let count = {
        let db = ctx.db.clone();
        tokio::task::spawn_blocking(move || db.with(|c| repo::count_sources(c, book_id)))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??
    };

    if count == 0 {
        return Err(AppError::Other("collected zero sources".into()));
    }

    Ok(())
}
