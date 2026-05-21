# BookWeaver — Roadmap

## Critical Files to Create

Rust 백엔드:
- [src-tauri/Cargo.toml](../src-tauri/Cargo.toml) — `tauri`, `tokio`, `reqwest`, `rusqlite`, `sqlite-vec`, `scraper`, `readability`, `async-trait`, `keyring`, `serde`, `tracing`
- [src-tauri/src/main.rs](../src-tauri/src/main.rs)
- [src-tauri/src/commands.rs](../src-tauri/src/commands.rs) — `start_book`, `get_book`, `list_books`, `update_settings`
- [src-tauri/src/pipeline/mod.rs](../src-tauri/src/pipeline/mod.rs) — orchestrator
- [src-tauri/src/ai/mod.rs](../src-tauri/src/ai/mod.rs) — `AiProvider` trait
- [src-tauri/src/db/schema.rs](../src-tauri/src/db/schema.rs) — [ARCHITECTURE.md#data-model](ARCHITECTURE.md#data-model) 의 스키마
- [src-tauri/tauri.conf.json](../src-tauri/tauri.conf.json) — `withGlobalTauri`, `allowlist` (`fs`, `dialog`, `shell` 없음)

프런트엔드:
- [package.json](../package.json) — `react`, `react-router-dom`, `@tauri-apps/api`, `tailwindcss`, `vite`
- [src/App.tsx](../src/App.tsx)
- [src/pages/Viewer.tsx](../src/pages/Viewer.tsx)
- [src/lib/tauri.ts](../src/lib/tauri.ts)
- [tsconfig.json](../tsconfig.json), [tailwind.config.ts](../tailwind.config.ts), [vite.config.ts](../vite.config.ts)

---

## Implementation Order

1. **Skeleton 구성**: `npm create tauri-app` (React + TS + Tailwind), package.json/Cargo.toml 정리, 빌드 확인
2. **DB 레이어**: sqlite-vec 통합, 마이그레이션, 기본 CRUD ([ARCHITECTURE.md#data-model](ARCHITECTURE.md#data-model))
3. **AI Provider**: trait + Claude 구현 1개 먼저, 키는 `.env`/keyring ([ARCHITECTURE.md#ai-provider-abstraction](ARCHITECTURE.md#ai-provider-abstraction))
4. **Pipeline 골격**: search/collect만 동작, 결과를 DB에 적재 후 종료 ([PIPELINE.md#stage-1-search](PIPELINE.md#stage-1-search), [PIPELINE.md#stage-2-collect](PIPELINE.md#stage-2-collect))
5. **Structure + Cluster**: chunk 분할, 임베딩, sqlite-vec KNN ([PIPELINE.md#stage-3-structure](PIPELINE.md#stage-3-structure), [PIPELINE.md#stage-4-cluster](PIPELINE.md#stage-4-cluster))
6. **Synthesis** (map-reduce, density 3종) ([PIPELINE.md#stage-5-synthesis](PIPELINE.md#stage-5-synthesis))
7. **Frontend**: 검색 → 진행 timeline → Viewer (ChapterTree, density 토글, 출처 표시) ([FRONTEND.md](FRONTEND.md))
8. **Settings**: 프로바이더 전환, OpenAI/Ollama 구현 추가 ([USECASES.md#uc-08-configure-ai-provider](USECASES.md#uc-08-configure-ai-provider))
9. **i18n** (ko/en), reading mode, focus mode ([USECASES.md#uc-07-toggle-reading-mode](USECASES.md#uc-07-toggle-reading-mode), [USECASES.md#uc-11-switch-language-koen](USECASES.md#uc-11-switch-language-koen))
10. **다듬기**: 에러/재시도, 캐싱, 로깅(`tracing`)

각 단계는 종단간 흐름이 깨지지 않도록 incremental하게 — 즉 단계 1~4까지만 끝나도 "URL은 모이고 DB에 들어가는 것"이 UI에서 보임.

---

## Verification

### 빌드/실행

- `cargo tauri dev` — 데스크톱 앱이 뜸
- 첫 화면에서 "Clean Code" 입력 → Library에 진행 중 카드가 나타남
- 약 1~3분 후 Viewer에서 챕터 트리/본문 확인 가능

### 검증 시나리오

1. **종단간**: "Atomic Habits" 검색 → 6단계 완료 → Viewer에서 5개 이상 챕터, 각 챕터에 1개 이상 source 인용 노출 ([UC-01](USECASES.md#uc-01-start-book-weaving) → [UC-04](USECASES.md#uc-04-read-weaved-book))
2. **Density 토글**: Light/Medium/Deep 전환 시 문장 길이가 달라짐 ([UC-05](USECASES.md#uc-05-switch-density))
3. **프로바이더 스위치**: Settings에서 Claude → Ollama 전환 후 재합성 정상 ([UC-08](USECASES.md#uc-08-configure-ai-provider), [UC-09](USECASES.md#uc-09-re-synthesize-with-different-provider))
4. **한국어 책**: "사피엔스" 같은 한국어 검색어로도 한국어 결과 생성
5. **오프라인 모드** (Ollama 단독): 인터넷 없이 (수집은 못 하지만) 기존 DB의 책은 다시 합성 가능 ([UC-10](USECASES.md#uc-10-operate-offline-ollama-only))

### 테스트

- Rust: `cargo test` — DB 마이그레이션, AI provider mock, pipeline 단위
- 프런트엔드: Vitest로 컴포넌트 스냅샷, 라우터 동작
