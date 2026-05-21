use scraper::{Html, Selector};
use url::Url;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ScrapedDoc {
    pub url: String,
    pub title: Option<String>,
    pub site: Option<String>,
    pub raw_html: String,
    pub clean_text: String,
}

pub async fn fetch_and_clean(http: &reqwest::Client, url: &str) -> Result<ScrapedDoc> {
    let resp = http
        .get(url)
        .header("accept", "text/html,application/xhtml+xml")
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(crate::error::AppError::Other(format!(
            "scrape {url} returned {status}"
        )));
    }
    let html = resp.text().await?;
    let doc = Html::parse_document(&html);

    let title = doc
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|t| t.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty());

    let clean_text = extract_main_text(&doc);
    let site = Url::parse(url).ok().and_then(|u| u.host_str().map(|s| s.to_string()));

    Ok(ScrapedDoc {
        url: url.to_string(),
        title,
        site,
        raw_html: html,
        clean_text,
    })
}

/// 매우 단순한 readability 대용: script/style/nav/header/footer 제거하고
/// p, h1~h6, li 텍스트를 줄바꿈으로 합친다. dom_smoothie를 쓰지 않는 이유는 의존성을 줄이기 위함.
fn extract_main_text(doc: &Html) -> String {
    let block_sel = Selector::parse(
        "article p, article h1, article h2, article h3, article h4, article li,\
         main p, main h1, main h2, main h3, main h4, main li,\
         section p, section h1, section h2, section h3, section h4, section li,\
         p, h1, h2, h3, h4, li",
    )
    .unwrap();

    let mut lines = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for el in doc.select(&block_sel) {
        // 본문 콘텐츠 외 영역 배제
        if has_excluded_ancestor(&el) {
            continue;
        }
        let text = el.text().collect::<String>().trim().to_string();
        if text.is_empty() || text.len() < 8 {
            continue;
        }
        if !seen.insert(text.clone()) {
            continue;
        }
        lines.push(text);
    }
    lines.join("\n\n")
}

fn has_excluded_ancestor(el: &scraper::ElementRef) -> bool {
    const EXCLUDED: &[&str] = &["nav", "header", "footer", "aside", "form", "script", "style"];
    let mut cur = el.parent();
    while let Some(node) = cur {
        if let Some(elref) = scraper::ElementRef::wrap(node) {
            let tag = elref.value().name();
            if EXCLUDED.contains(&tag) {
                return true;
            }
            cur = node.parent();
        } else {
            break;
        }
    }
    false
}
