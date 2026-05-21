# BookWeaver — Pipeline (6 Stages)

각 단계는 비동기 함수, `book_id`를 키로 상태를 갱신하며, Tauri event로 프런트엔드에 진행률을 emit 한다.

| # | 단계 | 입력 | 출력 | 비고 |
|---|------|------|------|------|
| 1 | **Search** | 책 제목 | book row 생성 | 메타데이터(저자/언어) 자동 추정 (LLM 1회 호출) |
| 2 | **Collect** | book_id | sources[] | Tavily/Brave 검색 API → URL 후보 → `reqwest` 스크래핑 → readability로 본문 추출 |
| 3 | **Structure** | sources[] | chunks + 잠정 목차 | (a) 규칙 기반으로 heading 추출 (b) LLM에게 "잠정 목차 후보" 합치게 요청 |
| 4 | **Cluster** | chunks | chunk ↔ chapter 매핑 | 각 chunk 임베딩 → sqlite-vec KNN → 잠정 목차 항목별 그룹화 |
| 5 | **Synthesis** | 그룹화된 chunks | chapters[] (3가지 density) | 챕터마다 LLM 호출, **map-reduce 패턴**으로 토큰 한계 회피 |
| 6 | **Viewer** | chapters[] | (UI에서 렌더) | React 측이 DB 직접 조회 |

---

## Stage 1: Search

- 입력: 사용자가 입력한 책 제목 (UI에서 IPC로 전달).
- 동작: `books` 테이블에 새 row를 만들고 `status='collecting'`으로 표기.
- 메타데이터(저자·언어)는 LLM 1회 호출로 추정. 실패 시 NULL 허용.
- 트리거: [UC-01 Start Book Weaving](USECASES.md#uc-01-start-book-weaving).

## Stage 2: Collect

- 입력: `book_id`.
- 검색: Tavily 또는 Brave Search API로 책 제목 + "review"/"summary"/"독후감"/"챕터 정리" 등의 쿼리 변형.
- 스크래핑: `reqwest`로 후보 URL을 가져오고 `readability` 알고리즘으로 본문 추출. raw HTML과 정제된 text를 모두 `sources`에 저장.
- 사이트 도메인은 `site` 컬럼에 보관 (velog, brunch, medium 등 통계용).
- 오프라인 모드(UC-10)에서는 이 단계가 스킵된다.

## Stage 3: Structure

- 입력: 한 book의 `sources[]`.
- 단계 (a) — 규칙 기반: `<h1>`/`<h2>` heading, "Chapter N", "N장" 같은 패턴을 추출.
- 단계 (b) — LLM 호출: 여러 출처에서 모인 heading 후보를 입력으로 주고 "잠정 목차"를 정리하게 함.
- 산출: `chunks` 테이블 (각 chunk는 `heading_path`로 잠정 목차상의 위치를 가짐).

## Stage 4: Cluster

- 입력: `chunks[]`.
- 동작: 각 chunk를 임베딩 (`AiProvider::embed`) → `chunk_vectors`에 저장.
- 잠정 목차의 각 항목에 대해 sqlite-vec KNN 쿼리로 가장 유사한 chunk들을 모음.
- 결과는 메모리 상의 `chapter_id_temp -> Vec<chunk_id>` 매핑. 다음 단계에서 사용.

## Stage 5: Synthesis

- 입력: 챕터별로 그룹화된 `chunks[]`.
- 동작: 각 챕터마다 LLM `complete()` 호출. **map-reduce 패턴**으로 토큰 한계 회피.
- Density 3종 (`summary_light`, `summary_med`, `summary_deep`)을 각각 호출하여 `chapters` 테이블에 저장.
- 인용된 source_id를 파싱하여 `chapter_sources`에 매핑.
- [UC-09 Re-synthesize](USECASES.md#uc-09-re-synthesize-with-different-provider) 의 진입점.

## Stage 6: Viewer

- 백엔드 작업 없음 — React가 SQLite에서 직접 조회.
- `books.status='ready'`로 전환되면 [UC-04 Read Weaved Book](USECASES.md#uc-04-read-weaved-book) 이 가능해진다.

---

## Token Limit 대응

- chunk 크기 ~ 800 tokens, overlap 100
- 챕터당 후보 chunk가 너무 많으면 → 임베딩 유사도 상위 N개만 1차 합성 → 결과를 모아 2차 합성 (map-reduce)
- Density는 동일 chunk 묶음에 대해 3번 호출 (Light/Medium/Deep)

## Hallucination Mitigation

- Synthesis 프롬프트에 "원문에 없는 내용 생성 금지, 각 문장마다 근거 source_id 인용" 강제
- 결과를 정규식 파싱하여 `chapter_sources` 채움
- 인용이 없는 문장은 UI에서 별도 표식 → [UC-06 Inspect Source Trace](USECASES.md#uc-06-inspect-source-trace) 에서 사용자가 확인

---

## Orchestrator Events

각 단계 시작/완료/실패 시 Tauri event를 emit:

```
book.progress  { book_id, stage: 'search'|'collect'|..., status: 'started'|'done'|'failed' }
book.ready     { book_id }
book.failed    { book_id, stage, error }
```

프런트엔드의 ProgressTimeline ([FRONTEND.md#progresstimeline](FRONTEND.md#progresstimeline)) 이 이 이벤트를 구독하여 [UC-02 Track Progress](USECASES.md#uc-02-track-progress) 를 구현한다.
