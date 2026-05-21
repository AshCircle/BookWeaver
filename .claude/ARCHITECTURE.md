# BookWeaver — Architecture

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Frontend (React + TS + Tailwind, in Tauri WebView)     │
│   - 검색/진행 UI, 뷰어, 설정                              │
└──────────────┬──────────────────────────────────────────┘
               │ Tauri IPC commands
┌──────────────▼──────────────────────────────────────────┐
│  Tauri Core (Rust)                                       │
│   - command handlers, event emitter, state              │
└──────────────┬──────────────────────────────────────────┘
               │ async pipeline orchestrator
   ┌───────────┼────────────┬─────────────┬──────────────┐
   ▼           ▼            ▼             ▼              ▼
 search    collect      structure     cluster        synthesis
 (API)    (scrape)     (rules+LLM)   (embed+vec)    (LLM)
   │           │            │             │              │
   └───────────┴────────────┴─────────────┴──────────────┘
                            │
                       ┌────▼────┐
                       │ SQLite  │  (+ sqlite-vec)
                       │  store  │
                       └─────────┘
```

모든 LLM/임베딩 호출은 단일 `ai_provider` trait 뒤에 추상화되어 Claude / OpenAI / Ollama 어느 것이든 동일 인터페이스로 호출.

파이프라인의 각 단계별 상세 동작은 [PIPELINE.md](PIPELINE.md) 참조.

---

## Repository Layout

```
BookWeaver/
├── src-tauri/                  # Rust 백엔드 (Tauri)
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands.rs         # Tauri IPC commands
│   │   ├── pipeline/           # 6단계 파이프라인
│   │   │   ├── search.rs
│   │   │   ├── collect.rs      # 검색 API + 스크래핑
│   │   │   ├── structure.rs    # 목차/챕터 추출
│   │   │   ├── cluster.rs      # 의미 클러스터링
│   │   │   ├── synthesis.rs    # 최종 합성
│   │   │   └── mod.rs
│   │   ├── ai/                 # AI 프로바이더 추상화
│   │   │   ├── mod.rs          # trait AiProvider
│   │   │   ├── claude.rs
│   │   │   ├── openai.rs
│   │   │   └── ollama.rs
│   │   ├── db/                 # SQLite + sqlite-vec
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── migrations/
│   │   ├── scrape/             # readability/HTML→text
│   │   └── config.rs           # 사용자 설정 (API 키, 모델 선택)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                        # React 프런트엔드
│   ├── main.tsx
│   ├── App.tsx
│   ├── pages/
│   │   ├── Search.tsx          # 책 제목 입력 + 진행 표시
│   │   ├── Library.tsx         # 저장된 책 목록
│   │   ├── Viewer.tsx          # 목차/본문/출처 뷰어
│   │   └── Settings.tsx        # AI 프로바이더/API 키
│   ├── components/
│   │   ├── ProgressTimeline.tsx
│   │   ├── ChapterTree.tsx
│   │   ├── SourceBadges.tsx
│   │   └── DensitySwitch.tsx
│   ├── lib/
│   │   ├── tauri.ts            # IPC 래퍼
│   │   └── i18n.ts             # ko/en
│   └── styles/
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── vite.config.ts
```

---

## Data Model

SQLite 기반. `sqlite-vec` 확장으로 벡터까지 동일 DB에 보관한다.

```sql
-- 책 1권 = 1 row
CREATE TABLE books (
  id            INTEGER PRIMARY KEY,
  title         TEXT NOT NULL,
  author        TEXT,
  language      TEXT,             -- 'ko' | 'en'
  status        TEXT,             -- 'collecting' | 'synthesizing' | 'ready' | 'failed'
  created_at    DATETIME,
  updated_at    DATETIME
);

-- 수집된 원본 문서
CREATE TABLE sources (
  id            INTEGER PRIMARY KEY,
  book_id       INTEGER REFERENCES books(id) ON DELETE CASCADE,
  url           TEXT UNIQUE,
  title         TEXT,
  site          TEXT,             -- velog, brunch, medium ...
  fetched_at    DATETIME,
  raw_html      TEXT,             -- 보존용
  clean_text    TEXT              -- readability 결과
);

-- 원본을 잘게 쪼갠 청크 (클러스터링/임베딩 단위)
CREATE TABLE chunks (
  id            INTEGER PRIMARY KEY,
  source_id     INTEGER REFERENCES sources(id) ON DELETE CASCADE,
  text          TEXT,
  heading_path  TEXT,             -- '1장 > ACID' 같은 경로
  token_count   INTEGER
);

-- sqlite-vec 가상 테이블 (임베딩)
CREATE VIRTUAL TABLE chunk_vectors USING vec0(
  chunk_id INTEGER PRIMARY KEY,
  embedding FLOAT[1024]
);

-- 합성 결과 챕터
CREATE TABLE chapters (
  id            INTEGER PRIMARY KEY,
  book_id       INTEGER REFERENCES books(id) ON DELETE CASCADE,
  order_idx     INTEGER,
  title         TEXT,
  summary_light TEXT,
  summary_med   TEXT,
  summary_deep  TEXT
);

-- 챕터 ↔ 소스 출처 매핑 (Source Traceability)
CREATE TABLE chapter_sources (
  chapter_id    INTEGER REFERENCES chapters(id) ON DELETE CASCADE,
  source_id     INTEGER REFERENCES sources(id) ON DELETE CASCADE,
  PRIMARY KEY (chapter_id, source_id)
);
```

`chapter_sources`는 [UC-06 Inspect Source Trace](USECASES.md#uc-06-inspect-source-trace) 의 근거 표시를 위한 매핑이다.

---

## AI Provider Abstraction

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, prompt: &Prompt) -> Result<String>;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn name(&self) -> &str;
    fn context_window(&self) -> usize;
}
```

설정에서 사용자가 프로바이더 + 모델 + (필요 시) API 키를 선택. 키는 OS keyring(`keyring` crate)에 저장. 자세한 UX는 [UC-08 Configure AI Provider](USECASES.md#uc-08-configure-ai-provider) 참조.

기본값:
- Claude: `claude-opus-4-7` (긴 합성용), `claude-haiku-4-5-20251001` (값싼 사전 작업)
- OpenAI: `gpt-4o`, `text-embedding-3-small`
- Ollama: `qwen2.5:14b`, `nomic-embed-text`

임베딩은 Ollama가 가장 저렴하므로 "프로바이더와 무관하게 임베딩만은 로컬"이라는 추가 옵션도 고려.
