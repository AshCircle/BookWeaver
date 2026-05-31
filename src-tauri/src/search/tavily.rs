use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::{SearchHit, SearchProvider};
use crate::error::{AppError, Result};

const ENDPOINT: &str = "https://api.tavily.com/search";

#[derive(Debug, Clone)]
pub struct TavilyClient {
    api_key: String,
    http: reqwest::Client,
}

impl TavilyClient {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    url: String,
    title: Option<String>,
    content: Option<String>,
}

#[async_trait]
impl SearchProvider for TavilyClient {
    async fn search(&self, query: &str, max_results: u32) -> Result<Vec<SearchHit>> {
        let body = json!({
            "api_key": self.api_key,
            "query": query,
            "max_results": max_results,
            "search_depth": "basic",
            "include_answer": false,
            "include_raw_content": false,
        });

        let resp = self
            .http
            .post(ENDPOINT)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(format!(
                "tavily api error {status}: {text}"
            )));
        }

        let parsed: TavilyResponse = resp.json().await?;
        Ok(parsed
            .results
            .into_iter()
            .map(|r| SearchHit {
                url: r.url,
                title: r.title,
                snippet: r.content,
            })
            .collect())
    }

    fn name(&self) -> &'static str {
        "tavily"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "live Tavily API; run with `cargo test -- --ignored` and TAVILY_API_KEY set"]
    async fn smoke_search() {
        let key = std::env::var("TAVILY_API_KEY").expect("TAVILY_API_KEY must be set");
        let client = TavilyClient::new(key, reqwest::Client::new());
        let hits = client.search("Clean Code 리뷰", 3).await.expect("tavily search failed");
        println!("[tavily] {} hits", hits.len());
        for h in &hits {
            println!("  - {}", h.url);
        }
        assert!(!hits.is_empty());
    }
}
