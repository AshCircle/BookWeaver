-- 책 1권 = 1 row
CREATE TABLE IF NOT EXISTS books (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  title         TEXT NOT NULL,
  author        TEXT,
  language      TEXT,
  status        TEXT NOT NULL DEFAULT 'collecting',
  created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_books_updated_at ON books(updated_at DESC);

CREATE TABLE IF NOT EXISTS sources (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  book_id       INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  url           TEXT NOT NULL,
  title         TEXT,
  site          TEXT,
  fetched_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  raw_html      TEXT,
  clean_text    TEXT,
  UNIQUE(book_id, url)
);

CREATE INDEX IF NOT EXISTS idx_sources_book_id ON sources(book_id);

CREATE TABLE IF NOT EXISTS chunks (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id     INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  text          TEXT NOT NULL,
  heading_path  TEXT,
  token_count   INTEGER
);

CREATE INDEX IF NOT EXISTS idx_chunks_source_id ON chunks(source_id);

CREATE TABLE IF NOT EXISTS chapters (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  book_id       INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  order_idx     INTEGER NOT NULL,
  title         TEXT NOT NULL,
  summary_light TEXT,
  summary_med   TEXT,
  summary_deep  TEXT
);

CREATE INDEX IF NOT EXISTS idx_chapters_book_id ON chapters(book_id, order_idx);

CREATE TABLE IF NOT EXISTS chapter_sources (
  chapter_id    INTEGER NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
  source_id     INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  PRIMARY KEY (chapter_id, source_id)
);
