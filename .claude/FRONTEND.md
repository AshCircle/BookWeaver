# BookWeaver — Frontend

Tauri WebView 내부에서 동작하는 React + TypeScript + Tailwind 애플리케이션.

## Routes

| Route | 화면 | 대응 유스케이스 |
|-------|------|----------------|
| `/` | 검색 (책 제목 입력) + Library 진입 | [UC-01](USECASES.md#uc-01-start-book-weaving) |
| `/library` | 저장된 책 목록, 진행 중 작업 | [UC-02](USECASES.md#uc-02-track-progress), [UC-03](USECASES.md#uc-03-browse-library) |
| `/book/:id` | Viewer: 좌측 ChapterTree, 중앙 본문, 우측 SourceBadges | [UC-04](USECASES.md#uc-04-read-weaved-book) |
| `/book/:id?density=light\|medium\|deep` | Density 토글 | [UC-05](USECASES.md#uc-05-switch-density) |
| `/settings` | AI 프로바이더/API 키, 테마, 폰트 | [UC-08](USECASES.md#uc-08-configure-ai-provider), [UC-11](USECASES.md#uc-11-switch-language-koen) |

---

## Search Route

`/` — 책 제목 입력 + 최근 진행 작업 카드.

- 입력 → IPC `start_book(title)` → 백엔드에서 [Stage 1 Search](PIPELINE.md#stage-1-search) 트리거.
- 동일 제목 중복 검사 후 신규 row 생성, `/library`로 자동 이동.

## Library Route

`/library` — 저장된 책 목록.

- `list_books` IPC로 모든 book row 조회 (`updated_at desc`).
- 각 카드:
  - `status='ready'` → 클릭 시 `/book/:id`로 이동.
  - 그 외 → ProgressTimeline 노출.
- 빈 상태에서는 "첫 책을 추가하세요" CTA.

## Viewer Route

`/book/:id` — 3-pane 레이아웃.

- **좌측 — ChapterTree**: `chapters.order_idx` 순으로 트리 표시.
  ```text
  Book
   ├── Chapter 1
   ├── Chapter 2
   └── Chapter 3
  ```
- **중앙 — 본문**: 선택된 챕터의 density에 대응되는 summary 컬럼 렌더.
- **우측 — SourceBadges**: `chapter_sources` 조인으로 인용된 출처 나열.
- 상단 우측: DensitySwitch, Reading Mode 토글, "재합성" 버튼 ([UC-09](USECASES.md#uc-09-re-synthesize-with-different-provider)).

### SourceBadges

각 챕터의 인용 출처를 site 도메인별로 그룹화하여 표시. 클릭 시 팝오버에서 URL과 제목 노출, "원본 보기"로 OS 브라우저 호출. 인용이 없는 문장에는 별도 경고 배지 ([UC-06](USECASES.md#uc-06-inspect-source-trace)).

## Settings Route

`/settings` — 사용자 환경 설정.

- AI Provider 드롭다운: Claude / OpenAI / Ollama.
- 모델 입력 (기본값 자동 제안).
- API 키: OS keyring에 저장. "테스트 호출" 버튼으로 1회 응답 확인.
- 언어 (`ko` / `en`): [i18n](#i18n) 참조.
- Reading Mode 기본값: 다크/라이트/시스템 추종.
- Ollama 사용 시 `localhost:11434` 헬스체크.

---

## Components

### ProgressTimeline

6단계 (Search → Collect → Structure → Cluster → Synthesis → Ready)를 시각화. `book.progress` Tauri event를 구독해 단계 전환 시 UI를 즉시 갱신. 실패 시 단계별 에러 메시지 노출 + 재시도 버튼. ([PIPELINE.md#orchestrator-events](PIPELINE.md#orchestrator-events))

### ChapterTree

`chapters` 테이블 기반 트리. 현재 평면 1-depth지만 향후 하위 섹션 확장을 고려해 트리 구조로 구현.

### DensitySwitch

Light / Medium / Deep 세 단계 세그먼티드 컨트롤. 선택 시 URL 쿼리 파라미터를 갱신하고 본문 컬럼을 교체. ([UC-05](USECASES.md#uc-05-switch-density))

### SourceBadges

위 [SourceBadges](#sourcebadges) 참조.

---

## Reading Mode

Viewer 내부 토글. 사용자 환경 설정을 로컬 스토리지에 영속화 ([UC-07](USECASES.md#uc-07-toggle-reading-mode)).

- **Dark mode** — 다크/라이트/시스템 추종
- **Typography** — 폰트 패밀리, 폰트 크기, 줄간격
- **Focus mode** — 사이드바·우측 패널을 숨겨 본문에 집중

---

## i18n

- 위치: [src/lib/i18n.ts](../src/lib/i18n.ts)
- 지원 언어: 한국어(`ko`, 기본) / 영어(`en`).
- UI 라벨만 i18n 적용. **합성된 챕터 본문은 합성 당시 언어로 보존**되며 `books.language` 컬럼이 출처. ([UC-11](USECASES.md#uc-11-switch-language-koen))
- 사용자 선택은 로컬 스토리지에 저장.

---

## IPC 래퍼

`src/lib/tauri.ts` — 백엔드 IPC 명령을 타입 안전하게 감싼 래퍼.

- `start_book(title)` → Stage 1 진입
- `get_book(id)`
- `list_books()`
- `update_settings(provider, model, api_key?)`
- `resynthesize(book_id, opts)` — [UC-09](USECASES.md#uc-09-re-synthesize-with-different-provider)

이벤트 구독:
- `book.progress`, `book.ready`, `book.failed` — [PIPELINE.md#orchestrator-events](PIPELINE.md#orchestrator-events)
