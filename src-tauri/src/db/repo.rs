use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: i64,
    pub book_id: i64,
    pub url: String,
    pub title: Option<String>,
    pub site: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i64,
    pub book_id: i64,
    pub order_idx: i64,
    pub title: String,
    pub summary_light: Option<String>,
    pub summary_med: Option<String>,
    pub summary_deep: Option<String>,
}

fn map_book(row: &rusqlite::Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get("id")?,
        title: row.get("title")?,
        author: row.get("author")?,
        language: row.get("language")?,
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        error_message: row.get("error_message")?,
    })
}

fn map_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    Ok(Source {
        id: row.get("id")?,
        book_id: row.get("book_id")?,
        url: row.get("url")?,
        title: row.get("title")?,
        site: row.get("site")?,
        fetched_at: row.get("fetched_at")?,
    })
}

fn map_chapter(row: &rusqlite::Row<'_>) -> rusqlite::Result<Chapter> {
    Ok(Chapter {
        id: row.get("id")?,
        book_id: row.get("book_id")?,
        order_idx: row.get("order_idx")?,
        title: row.get("title")?,
        summary_light: row.get("summary_light")?,
        summary_med: row.get("summary_med")?,
        summary_deep: row.get("summary_deep")?,
    })
}

pub fn insert_book(conn: &Connection, title: &str) -> Result<Book> {
    conn.execute(
        "INSERT INTO books (title, status) VALUES (?1, 'collecting')",
        params![title],
    )?;
    let id = conn.last_insert_rowid();
    get_book(conn, id)?
        .ok_or_else(|| AppError::NotFound(format!("book {id} after insert")))
}

pub fn find_book_by_title(conn: &Connection, title: &str) -> Result<Option<Book>> {
    Ok(conn
        .query_row(
            "SELECT * FROM books WHERE title = ?1 LIMIT 1",
            params![title],
            map_book,
        )
        .optional()?)
}

pub fn list_books(conn: &Connection) -> Result<Vec<Book>> {
    let mut stmt = conn.prepare("SELECT * FROM books ORDER BY updated_at DESC")?;
    let rows = stmt
        .query_map([], map_book)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_book(conn: &Connection, id: i64) -> Result<Option<Book>> {
    Ok(conn
        .query_row("SELECT * FROM books WHERE id = ?1", params![id], map_book)
        .optional()?)
}

pub fn update_status(
    conn: &Connection,
    id: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<()> {
    let changed = conn.execute(
        "UPDATE books SET status = ?1, error_message = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![status, error_message, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("book {id}")));
    }
    Ok(())
}

pub fn update_metadata(
    conn: &Connection,
    id: i64,
    author: Option<&str>,
    language: Option<&str>,
) -> Result<()> {
    let changed = conn.execute(
        "UPDATE books SET author = COALESCE(?1, author), language = COALESCE(?2, language), updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![author, language, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("book {id}")));
    }
    Ok(())
}

pub fn insert_source(
    conn: &Connection,
    book_id: i64,
    url: &str,
    title: Option<&str>,
    site: Option<&str>,
    raw_html: Option<&str>,
    clean_text: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO sources (book_id, url, title, site, raw_html, clean_text) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(book_id, url) DO UPDATE SET \
           title = COALESCE(excluded.title, sources.title), \
           site = COALESCE(excluded.site, sources.site), \
           raw_html = COALESCE(excluded.raw_html, sources.raw_html), \
           clean_text = COALESCE(excluded.clean_text, sources.clean_text), \
           fetched_at = CURRENT_TIMESTAMP",
        params![book_id, url, title, site, raw_html, clean_text],
    )?;
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM sources WHERE book_id = ?1 AND url = ?2",
            params![book_id, url],
            |row| row.get(0),
        )
        .optional()?;
    id.ok_or_else(|| AppError::NotFound(format!("source after insert ({url})")))
}

pub fn list_sources(conn: &Connection, book_id: i64) -> Result<Vec<Source>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, url, title, site, fetched_at FROM sources \
         WHERE book_id = ?1 ORDER BY fetched_at ASC",
    )?;
    let rows = stmt
        .query_map(params![book_id], map_source)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn count_sources(conn: &Connection, book_id: i64) -> Result<i64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sources WHERE book_id = ?1",
        params![book_id],
        |row| row.get(0),
    )?;
    Ok(n)
}

pub fn insert_chunk(
    conn: &Connection,
    source_id: i64,
    text: &str,
    heading_path: Option<&str>,
    token_count: Option<i64>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO chunks (source_id, text, heading_path, token_count) VALUES (?1, ?2, ?3, ?4)",
        params![source_id, text, heading_path, token_count],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn insert_chapter(
    conn: &Connection,
    book_id: i64,
    order_idx: i64,
    title: &str,
    summary_light: Option<&str>,
    summary_med: Option<&str>,
    summary_deep: Option<&str>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO chapters (book_id, order_idx, title, summary_light, summary_med, summary_deep) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![book_id, order_idx, title, summary_light, summary_med, summary_deep],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_chapters(conn: &Connection, book_id: i64) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, book_id, order_idx, title, summary_light, summary_med, summary_deep \
         FROM chapters WHERE book_id = ?1 ORDER BY order_idx ASC",
    )?;
    let rows = stmt
        .query_map(params![book_id], map_chapter)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn link_chapter_source(conn: &Connection, chapter_id: i64, source_id: i64) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO chapter_sources (chapter_id, source_id) VALUES (?1, ?2)",
        params![chapter_id, source_id],
    )?;
    Ok(())
}

pub fn list_chapter_sources(conn: &Connection, chapter_id: i64) -> Result<Vec<Source>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.book_id, s.url, s.title, s.site, s.fetched_at \
         FROM sources s \
         INNER JOIN chapter_sources cs ON cs.source_id = s.id \
         WHERE cs.chapter_id = ?1 \
         ORDER BY s.fetched_at ASC",
    )?;
    let rows = stmt
        .query_map(params![chapter_id], map_source)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn delete_chapters_for_book(conn: &Connection, book_id: i64) -> Result<()> {
    conn.execute("DELETE FROM chapters WHERE book_id = ?1", params![book_id])?;
    Ok(())
}
