use std::net::IpAddr;

use scraper::{Html, Selector};
use url::{Host, Url};

use crate::error::{AppError, Result};

#[derive(Debug, Clone)]
pub struct ScrapedDoc {
    pub url: String,
    pub title: Option<String>,
    pub site: Option<String>,
    pub raw_html: String,
    pub clean_text: String,
}

pub async fn fetch_and_clean(http: &reqwest::Client, url: &str) -> Result<ScrapedDoc> {
    validate_url(url)?;

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

/// 외부 검색 결과 URL을 그대로 요청하기 전, 스킴과 호스트를 검증해 내부 자원(SSRF) 접근을 막는다.
/// 도메인이 내부 IP로 DNS 해석되는 경우는 별도 리졸브가 필요하므로 여기서는 다루지 않는다.
fn validate_url(url: &str) -> Result<()> {
    let parsed =
        Url::parse(url).map_err(|e| AppError::Other(format!("invalid url: {e}")))?;

    match parsed.scheme() {
        "http" | "https" => {}
        other => return Err(AppError::Other(format!("unsupported url scheme: {other}"))),
    }

    match parsed.host() {
        Some(Host::Domain(d)) if d.eq_ignore_ascii_case("localhost") => {
            Err(AppError::Other("localhost is not allowed".into()))
        }
        Some(Host::Domain(_)) => Ok(()),
        Some(Host::Ipv4(ip)) => validate_ip(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => validate_ip(IpAddr::V6(ip)),
        None => Err(AppError::Other("url has no host".into())),
    }
}

fn validate_ip(ip: IpAddr) -> Result<()> {
    let blocked = match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_unspecified()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_multicast(),
    };
    if blocked {
        return Err(AppError::Other(format!(
            "address {ip} is not allowed (internal/reserved)"
        )));
    }
    Ok(())
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
