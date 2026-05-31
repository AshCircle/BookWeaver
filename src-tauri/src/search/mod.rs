use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

pub mod tavily;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, max_results: u32) -> Result<Vec<SearchHit>>;
    fn name(&self) -> &'static str;
}

pub fn query_variants(title: &str, language: Option<&str>) -> Vec<String> {
    let mut v = Vec::new();
    let lang = language.unwrap_or("ko");
    if lang == "ko" {
        v.push(format!("{title} 리뷰"));
        v.push(format!("{title} 챕터 요약"));
        v.push(format!("{title} 독후감"));
        v.push(format!("{title} 책 정리"));
    } else {
        v.push(format!("{title} review"));
        v.push(format!("{title} chapter summary"));
        v.push(format!("{title} book notes"));
        v.push(format!("{title} key takeaways"));
    }
    v
}
