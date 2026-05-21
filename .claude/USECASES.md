# BookWeaver — Use Cases

본 문서는 BookWeaver 애플리케이션의 유스케이스를 **UML 스타일 (Actor / Goal / Precondition / Main Flow / Alternative Flow / Postcondition)** 로 정의한다.

각 유스케이스는 마지막 줄에 **구현 참조** 항목을 두어 어느 파이프라인 단계 / 라우트에 대응되는지 명시한다.

## Actors

- **Reader (사용자)**: 책에 대한 통합 요약을 얻고자 하는 최종 사용자. 데스크톱 앱을 단독으로 사용한다.

외부 시스템 (Actor가 아닌 협력자):
- 검색 API (Tavily / Brave)
- 웹 페이지 (블로그, 리뷰, 노트)
- AI Provider (Claude / OpenAI / Ollama)
- 로컬 SQLite DB

---

## UC-01. Start Book Weaving

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 책 제목을 입력해 새 책의 통합 요약 합성을 개시한다. |
| **Precondition** | 앱이 실행 중이며, AI Provider가 최소 1개 설정되어 있다 (UC-08). |
| **Main Flow** | 1. Reader가 `/` (Search) 라우트에서 책 제목을 입력한다.<br>2. 시스템이 `books` row를 생성하고 `status='collecting'`으로 표시한다.<br>3. LLM 1회 호출로 저자·언어 메타데이터를 추정한다.<br>4. Collect → Structure → Cluster → Synthesis 파이프라인이 비동기로 시작된다.<br>5. Reader는 `/library`로 이동되어 진행 카드를 본다 (UC-02). |
| **Alternative Flow** | - 동일 제목의 책이 이미 존재하면 기존 book을 보여주고 "재합성"을 제안한다 (UC-09).<br>- 메타데이터 추정 실패 시 사용자가 직접 저자·언어 입력 가능. |
| **Postcondition** | `books`에 새 row, 파이프라인 비동기 실행 중. |
| **구현 참조** | [PIPELINE.md#stage-1-search](PIPELINE.md#stage-1-search), [FRONTEND.md#search-route](FRONTEND.md#search-route) |

---

## UC-02. Track Progress

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 진행 중인 책의 6단계 파이프라인 진행률을 실시간으로 확인한다. |
| **Precondition** | UC-01이 시작되어 적어도 1권의 책이 합성 중. |
| **Main Flow** | 1. Reader가 `/library`에 진입한다.<br>2. 각 진행 중 책의 카드에 ProgressTimeline 컴포넌트가 6단계 (Search → Collect → Structure → Cluster → Synthesis → Ready)를 표시한다.<br>3. Tauri event가 단계 전환 시점에 emit되어 UI가 자동 갱신된다. |
| **Alternative Flow** | - 단계 실패 시 카드가 `status='failed'`로 전환되고 에러 메시지 노출.<br>- Reader가 "재시도" 버튼으로 실패 단계부터 재실행 가능. |
| **Postcondition** | UI 상태가 DB의 `books.status`와 동기화. |
| **구현 참조** | [PIPELINE.md#orchestrator-events](PIPELINE.md#orchestrator-events), [FRONTEND.md#library-route](FRONTEND.md#library-route) |

---

## UC-03. Browse Library

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 합성 완료/진행 중 책 목록을 탐색하여 읽고 싶은 책을 선택한다. |
| **Precondition** | DB에 1권 이상의 book row 존재. |
| **Main Flow** | 1. Reader가 `/library`에 진입한다.<br>2. 시스템이 `books` 테이블에서 모든 책을 `updated_at desc` 순으로 조회·표시한다.<br>3. Reader가 카드를 클릭한다.<br>4. `status='ready'`이면 `/book/:id`로 이동 (UC-04), 그 외엔 진행 카드를 펼친다 (UC-02). |
| **Alternative Flow** | - DB가 비어있으면 "첫 책을 추가하세요" CTA를 표시 (UC-01 유도). |
| **Postcondition** | 변동 없음 (읽기 전용). |
| **구현 참조** | [ARCHITECTURE.md#data-model](ARCHITECTURE.md#data-model), [FRONTEND.md#library-route](FRONTEND.md#library-route) |

---

## UC-04. Read Weaved Book

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 합성된 책의 챕터 트리, 본문, 출처를 뷰어에서 읽는다. |
| **Precondition** | 대상 책의 `status='ready'`, `chapters` 테이블에 1개 이상 row. |
| **Main Flow** | 1. Reader가 `/book/:id`에 진입한다.<br>2. 좌측 ChapterTree가 `chapters.order_idx` 순으로 목차를 렌더한다.<br>3. 중앙 본문은 선택된 챕터의 `summary_med` (기본 density)를 표시한다.<br>4. 우측 SourceBadges는 `chapter_sources` 조인을 통해 인용된 원본 출처를 나열한다. |
| **Alternative Flow** | - density 변경 (UC-05), 출처 클릭 (UC-06), reading mode 전환 (UC-07)이 동일 화면에서 가능. |
| **Postcondition** | 변동 없음 (읽기 전용). |
| **구현 참조** | [FRONTEND.md#viewer-route](FRONTEND.md#viewer-route), [ARCHITECTURE.md#data-model](ARCHITECTURE.md#data-model) |

---

## UC-05. Switch Density

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 요약 강도(Light / Medium / Deep)를 토글하여 같은 챕터를 다른 깊이로 읽는다. |
| **Precondition** | UC-04 진행 중. 챕터의 `summary_light`, `summary_med`, `summary_deep` 모두 채워져 있음. |
| **Main Flow** | 1. Reader가 DensitySwitch 컴포넌트에서 Light/Medium/Deep 중 하나를 선택한다.<br>2. URL이 `/book/:id?density=light\|medium\|deep`로 갱신된다.<br>3. 본문이 해당 컬럼의 텍스트로 즉시 교체된다. |
| **Alternative Flow** | - 특정 density가 비어있으면 (구버전 book) 회색 표시 + "재합성 필요" 안내 (UC-09 유도). |
| **Postcondition** | URL과 UI 상태 동기화. DB 변동 없음. |
| **구현 참조** | [PIPELINE.md#stage-5-synthesis](PIPELINE.md#stage-5-synthesis), [FRONTEND.md#viewer-route](FRONTEND.md#viewer-route) |

---

## UC-06. Inspect Source Trace

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 본문 문단의 근거가 된 원본 출처(블로그·리뷰 URL)를 확인하여 환각 여부를 판단한다. |
| **Precondition** | UC-04 진행 중. 챕터에 1개 이상의 `chapter_sources` 매핑 존재. |
| **Main Flow** | 1. Reader가 본문의 인용 마크 또는 우측 SourceBadges를 클릭한다.<br>2. 시스템이 해당 source의 `url`, `title`, `site`를 팝오버로 표시한다.<br>3. Reader가 "원본 보기"를 누르면 OS 기본 브라우저로 URL을 연다. |
| **Alternative Flow** | - 인용이 없는 문장은 "출처 미상" 경고 배지가 붙어 있어 신뢰도 낮음을 알린다. |
| **Postcondition** | 변동 없음. |
| **구현 참조** | [PIPELINE.md#hallucination-mitigation](PIPELINE.md#hallucination-mitigation), [FRONTEND.md#sourcebadges](FRONTEND.md#sourcebadges) |

---

## UC-07. Toggle Reading Mode

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 다크 모드, 타이포그래피, 포커스 모드를 변경하여 읽기 환경을 개인화한다. |
| **Precondition** | UC-04 진행 중. |
| **Main Flow** | 1. Reader가 Viewer 상단의 Reading Mode 토글을 연다.<br>2. dark/light, 폰트 크기, focus mode (사이드바 숨김) 중 하나 이상을 변경한다.<br>3. 설정이 로컬 스토리지에 저장되어 다음 진입 시 유지된다. |
| **Alternative Flow** | - 시스템 다크 모드를 자동 추종하는 옵션도 제공. |
| **Postcondition** | 사용자 환경 설정이 영속화. DB 변동 없음. |
| **구현 참조** | [FRONTEND.md#reading-mode](FRONTEND.md#reading-mode) |

---

## UC-08. Configure AI Provider

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 사용할 AI Provider (Claude / OpenAI / Ollama), 모델, API 키를 설정한다. |
| **Precondition** | 앱이 실행 중. |
| **Main Flow** | 1. Reader가 `/settings`에 진입한다.<br>2. Provider 드롭다운에서 Claude / OpenAI / Ollama 중 선택.<br>3. 모델명을 입력 (기본값 자동 채움).<br>4. (Claude/OpenAI인 경우) API 키 입력 — OS keyring에 암호화 저장.<br>5. "테스트 호출" 버튼으로 1회 토큰 응답 확인.<br>6. 저장 시 이후 모든 합성에 적용. |
| **Alternative Flow** | - Ollama 선택 시 `localhost:11434` 헬스체크 자동 수행, 미실행 시 안내.<br>- 임베딩만 Ollama로 강제하는 별도 옵션도 가능. |
| **Postcondition** | `config` 저장소에 provider 선택 영속화, API 키는 keyring에. |
| **구현 참조** | [ARCHITECTURE.md#ai-provider-abstraction](ARCHITECTURE.md#ai-provider-abstraction), [FRONTEND.md#settings-route](FRONTEND.md#settings-route) |

---

## UC-09. Re-synthesize with Different Provider

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 기존 책을 다른 AI Provider 또는 다른 모델로 다시 합성한다. |
| **Precondition** | 대상 책의 `status='ready'`. `sources`, `chunks`, `chunk_vectors`는 보존되어 있음. |
| **Main Flow** | 1. Reader가 `/book/:id`에서 "재합성" 버튼을 누른다.<br>2. 현재 Provider 정보를 표시하고 변경 가능 여부를 안내.<br>3. 확인 시 시스템은 Stage 5 (Synthesis)부터 재실행 — Collect/Structure/Cluster는 재사용.<br>4. 기존 `chapters` row를 갈아끼우고 `updated_at` 갱신. |
| **Alternative Flow** | - "원본부터 재수집" 옵션 선택 시 Stage 2부터 재실행 (오래 걸림 경고). |
| **Postcondition** | `chapters` 테이블이 갱신, 기존 `chunk_vectors`는 임베딩 모델이 동일하면 재사용. |
| **구현 참조** | [PIPELINE.md#stage-5-synthesis](PIPELINE.md#stage-5-synthesis), [ARCHITECTURE.md#ai-provider-abstraction](ARCHITECTURE.md#ai-provider-abstraction) |

---

## UC-10. Operate Offline (Ollama-only)

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | 인터넷 연결 없이 로컬 Ollama만으로 기존 DB의 책을 다시 합성한다. |
| **Precondition** | UC-08에서 Ollama가 설정되어 있고 로컬에서 실행 중. DB에 기존 책의 `sources`/`chunks`가 있음. |
| **Main Flow** | 1. Reader가 오프라인 상태에서 앱을 실행한다.<br>2. `/library`에서 기존 책을 선택해 UC-09를 트리거.<br>3. 시스템은 Stage 2 (Collect)를 건너뛰고 (네트워크 필요) Stage 5부터 진행.<br>4. 합성 결과가 갱신된다. |
| **Alternative Flow** | - UC-01 (신규 책)을 시도하면 "오프라인에서는 수집 불가" 안내. |
| **Postcondition** | 기존 책의 챕터만 갱신. 신규 수집은 불가. |
| **구현 참조** | [PIPELINE.md#stage-2-collect](PIPELINE.md#stage-2-collect), [ARCHITECTURE.md#ai-provider-abstraction](ARCHITECTURE.md#ai-provider-abstraction) |

---

## UC-11. Switch Language (ko/en)

| 항목 | 내용 |
|------|------|
| **Actor** | Reader |
| **Goal** | UI 표시 언어를 한국어/영어 사이에서 전환한다. |
| **Precondition** | 앱 실행 중. |
| **Main Flow** | 1. Reader가 `/settings`에서 Language 드롭다운을 변경한다.<br>2. 시스템이 `i18n` 리소스를 교체하여 모든 라벨을 즉시 갱신한다.<br>3. 선택이 로컬에 영속화된다. |
| **Alternative Flow** | - 책의 합성 언어는 별개 — `books.language` 컬럼이 책별로 보존되며 UI 언어와 독립적. |
| **Postcondition** | UI 언어만 변경. 합성 결과 텍스트는 그대로. |
| **구현 참조** | [FRONTEND.md#i18n](FRONTEND.md#i18n) |
