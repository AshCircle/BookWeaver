# BookWeaver — Application Plan

## Context

`draft.md`는 BookWeaver의 비전을 정의한다: 인터넷에 흩어진 책 요약/리뷰/노트를 수집·정제·재구성하여 하나의 구조화된 책 요약으로 복원하는 **로컬 설치형 AI 애플리케이션**.

현재 저장소는 사실상 비어있다 (draft.md 외에는 코드가 없음). 따라서 이 플랜은 "어떻게 구현할지"가 아닌 **"어떤 스택과 모듈로 처음부터 구성할지"**를 정의한다.

핵심 결정:
- **Tauri 데스크톱 앱** (Rust 백엔드 + React/TS/Tailwind 프런트엔드)
- **하이브리드 AI**: 사용자가 Claude / OpenAI / Ollama 중 설정에서 선택
- **수집 방식**: 검색 API (Tavily/Brave) + 직접 스크래핑
- **저장소**: SQLite + sqlite-vec (메타데이터·본문·벡터 모두 단일 DB)
- **언어**: 한국어 우선, 영어 지원
- **MVP 범위**: Search → Collection → Structure → Cluster → Synthesis → Viewer **종단간**

---

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
├── vite.config.ts
└── draft.md
```

---

## Data Model (SQLite)

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

---

## Pipeline (6 Stages)

각 단계는 비동기 함수, `book_id`를 키로 상태 갱신, Tauri event로 프런트엔드에 진행률 emit.

| # | 단계 | 입력 | 출력 | 비고 |
|---|------|------|------|------|
| 1 | **Search** | 책 제목 | book row 생성 | 메타데이터(저자/언어) 자동 추정 (LLM 1회 호출) |
| 2 | **Collect** | book_id | sources[] | Tavily/Brave 검색 API → URL 후보 → `reqwest` 스크래핑 → readability로 본문 추출 |
| 3 | **Structure** | sources[] | chunks + 잠정 목차 | (a) 규칙 기반으로 heading 추출 (b) LLM에게 "잠정 목차 후보" 합치게 요청 |
| 4 | **Cluster** | chunks | chunk ↔ chapter 매핑 | 각 chunk 임베딩 → sqlite-vec KNN → 잠정 목차 항목별 그룹화 |
| 5 | **Synthesis** | 그룹화된 chunks | chapters[] (3가지 density) | 챕터마다 LLM 호출, **map-reduce 패턴**으로 토큰 한계 회피 |
| 6 | **Viewer** | chapters[] | (UI에서 렌더) | React 측이 DB 직접 조회 |

### Token Limit 대응
- chunk 크기 ~ 800 tokens, overlap 100
- 챕터당 후보 chunk가 너무 많으면 → 임베딩 유사도 상위 N개만 1차 합성 → 결과를 모아 2차 합성 (map-reduce)
- Density는 동일 chunk 묶음에 대해 3번 호출 (Light/Medium/Deep)

### Hallucination 완화
- Synthesis 프롬프트에 "원문에 없는 내용 생성 금지, 각 문장마다 근거 source_id 인용" 강제
- 결과를 정규식 파싱하여 `chapter_sources` 채움
- 인용이 없는 문장은 UI에서 별도 표식

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

설정에서 사용자가 프로바이더 + 모델 + (필요 시) API 키를 선택. 키는 OS keyring(`keyring` crate)에 저장.

기본값:
- Claude: `claude-opus-4-7` (긴 합성용), `claude-haiku-4-5-20251001` (값싼 사전 작업)
- OpenAI: `gpt-4o`, `text-embedding-3-small`
- Ollama: `qwen2.5:14b`, `nomic-embed-text`

임베딩은 Ollama가 가장 저렴하므로 "프로바이더와 무관하게 임베딩만은 로컬"이라는 추가 옵션도 고려.

---

## Frontend Routes

| Route | 화면 |
|-------|------|
| `/` | 검색 (책 제목 입력) + Library 진입 |
| `/library` | 저장된 책 목록, 진행 중 작업 |
| `/book/:id` | Viewer: 좌측 ChapterTree, 중앙 본문, 우측 SourceBadges |
| `/book/:id?density=light\|medium\|deep` | Density 토글 |
| `/settings` | AI 프로바이더/API 키, 테마, 폰트 |

Reading mode (dark/typography/focus)는 Viewer 내부 토글로.

---

## Critical Files to Create

- [src-tauri/Cargo.toml](src-tauri/Cargo.toml) — `tauri`, `tokio`, `reqwest`, `rusqlite`, `sqlite-vec`, `scraper`, `readability`, `async-trait`, `keyring`, `serde`, `tracing`
- [src-tauri/src/main.rs](src-tauri/src/main.rs)
- [src-tauri/src/commands.rs](src-tauri/src/commands.rs) — `start_book`, `get_book`, `list_books`, `update_settings`
- [src-tauri/src/pipeline/mod.rs](src-tauri/src/pipeline/mod.rs) — orchestrator
- [src-tauri/src/ai/mod.rs](src-tauri/src/ai/mod.rs) — `AiProvider` trait
- [src-tauri/src/db/schema.rs](src-tauri/src/db/schema.rs) — 위 스키마
- [package.json](package.json) — `react`, `react-router-dom`, `@tauri-apps/api`, `tailwindcss`, `vite`
- [src/App.tsx](src/App.tsx)
- [src/pages/Viewer.tsx](src/pages/Viewer.tsx)
- [src/lib/tauri.ts](src/lib/tauri.ts)
- [tauri.conf.json](src-tauri/tauri.conf.json) — `withGlobalTauri`, `allowlist` (`fs`, `dialog`, `shell` 없음)
- [tsconfig.json](tsconfig.json), [tailwind.config.ts](tailwind.config.ts), [vite.config.ts](vite.config.ts)

---

## Implementation Order

1. **Skeleton 구성**: `npm create tauri-app` (React + TS + Tailwind), package.json/Cargo.toml 정리, 빌드 확인
2. **DB 레이어**: sqlite-vec 통합, 마이그레이션, 기본 CRUD
3. **AI Provider**: trait + Claude 구현 1개 먼저, 키는 `.env`/keyring
4. **Pipeline 골격**: search/collect만 동작, 결과를 DB에 적재 후 종료
5. **Structure + Cluster**: chunk 분할, 임베딩, sqlite-vec KNN
6. **Synthesis (map-reduce, density 3종)**
7. **Frontend**: 검색 → 진행 timeline → Viewer (ChapterTree, density 토글, 출처 표시)
8. **Settings**: 프로바이더 전환, OpenAI/Ollama 구현 추가
9. **i18n** (ko/en), reading mode, focus mode
10. **다듬기**: 에러/재시도, 캐싱, 로깅(`tracing`)

각 단계는 종단간 흐름이 깨지지 않도록 incremental하게 — 즉 단계 1~4까지만 끝나도 "URL은 모이고 DB에 들어가는 것"이 UI에서 보임.

---

## Verification

빌드/실행:
- `cargo tauri dev` — 데스크톱 앱이 뜸
- 첫 화면에서 "Clean Code" 입력 → Library에 진행 중 카드가 나타남
- 약 1~3분 후 Viewer에서 챕터 트리/본문 확인 가능

검증 시나리오:
1. **종단간**: "Atomic Habits" 검색 → 6단계 완료 → Viewer에서 5개 이상 챕터, 각 챕터에 1개 이상 source 인용 노출
2. **Density 토글**: Light/Medium/Deep 전환 시 문장 길이가 달라짐
3. **프로바이더 스위치**: Settings에서 Claude → Ollama 전환 후 재합성 정상
4. **한국어 책**: "사피엔스" 같은 한국어 검색어로도 한국어 결과 생성
5. **오프라인 모드** (Ollama 단독): 인터넷 없이 (수집은 못 하지만) 기존 DB의 책은 다시 합성 가능

테스트:
- Rust: `cargo test` — DB 마이그레이션, AI provider mock, pipeline 단위
- 프런트엔드: Vitest로 컴포넌트 스냅샷, 라우터 동작

