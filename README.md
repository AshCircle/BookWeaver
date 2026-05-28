# BookWeaver

인터넷에 흩어진 책 요약 조각들을 수집·정제·재구성하여 하나의 구조화된 책 요약으로 복원하는 로컬 설치형 AI 데스크톱 앱.

설계 문서는 [.claude/PLAN.md](.claude/PLAN.md) 참고.

> 현재 상태: **Skeleton + Stage 1~2 동작 + Stage 3~5 stub**. ROADMAP의 1~4단계까지 구현되어 있고,
> 검색→수집→DB→Viewer까지의 종단간 흐름이 끊김 없이 동작한다. 진짜 구조화/클러스터/합성은 다음 세션의 작업.

## Stack

- **Frontend**: React 18 + TypeScript + TailwindCSS + Vite (in Tauri WebView)
- **Backend**: Tauri 2.x (Rust) + Tokio + reqwest
- **Storage**: SQLite (`rusqlite` + `sqlite-vec`)
- **AI**: Gemini (Google Generative Language API) — OpenAI/Ollama는 trait stub만
- **Search**: Tavily

## 사전 요구사항

- Node.js 20+
- Rust toolchain (1.77+)
- Tauri 2.x system deps:
  - **Windows**: Microsoft WebView2 Runtime + MSVC Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: webkit2gtk-4.1 등 — 자세한 사항은 https://tauri.app/start/prerequisites/

## 셋업

```sh
# 1) 의존성 설치
npm install

# 2) (선택) 환경 변수 — .env.example 참고
#    Settings 페이지에서도 입력 가능. env가 우선 fallback.
copy .env.example .env       # Windows
# cp .env.example .env       # macOS/Linux

# 3) 데스크톱 앱 실행 (frontend dev 서버 + Tauri WebView)
npm run tauri dev
```

처음 실행 시 SQLite 파일이 OS의 데이터 디렉터리에 자동 생성된다.

- Windows: `%APPDATA%/dev/bookweaver/BookWeaver/data/bookweaver.sqlite`
- macOS: `~/Library/Application Support/dev.bookweaver.BookWeaver/`
- Linux: `~/.local/share/bookweaver/`

설정 파일(config.json)도 같은 형태로 `config_dir`에 저장.

## 사용 흐름

1. 앱이 뜨면 `/settings`로 가서 **Gemini API Key**와 **Tavily API Key**를 입력 후 **저장**.
2. **테스트 호출** 버튼으로 키 연결 확인.
3. `/`에서 책 제목 입력 → 자동으로 `/library`로 이동.
4. Library 카드의 ProgressTimeline이 Search → Collect → Structure(stub) → Cluster(stub) → Synthesis(stub) → Ready 순으로 흐른다.
5. Ready 상태가 되면 카드 클릭 → Viewer에서 stub chapter와 수집된 sources를 확인.

## 디렉터리 구조

```
BookWeaver/
├── .claude/                # 설계 문서 (OVERVIEW, USECASES, ARCHITECTURE, PIPELINE, FRONTEND, ROADMAP)
├── src-tauri/              # Rust 백엔드 (Tauri 2)
│   ├── src/
│   │   ├── ai/             # AiProvider trait + Gemini/OpenAI/Ollama
│   │   ├── db/             # SQLite + 스키마 + repo
│   │   ├── pipeline/       # 6단계 orchestrator
│   │   ├── scrape/         # 본문 추출
│   │   ├── search/         # SearchProvider + Tavily
│   │   ├── commands.rs     # Tauri IPC commands
│   │   ├── config.rs       # UserConfig 영속화
│   │   ├── error.rs        # AppError
│   │   ├── state.rs        # AppState
│   │   ├── lib.rs          # Tauri entry
│   │   └── main.rs
│   ├── capabilities/
│   └── Cargo.toml
├── src/                    # React
│   ├── components/
│   ├── lib/                # tauri.ts (IPC 래퍼), types, i18n
│   ├── pages/              # Search, Library, Viewer, Settings
│   └── App.tsx, main.tsx, index.css
└── package.json, vite.config.ts, tailwind.config.ts, tsconfig.json
```

## 다음 작업

- Stage 3 진짜 구현: heading 추출 + LLM 잠정 목차
- Stage 4: 임베딩 + sqlite-vec KNN
- Stage 5: map-reduce 합성 + density 3종 + chapter_sources 파싱
- Stage 9: keyring 마이그레이션 + OpenAI/Ollama provider
- 재합성 (UC-09), 오프라인 모드 (UC-10)
- Reading mode (다크/포커스), 회고 테스트

## 알려진 제약 (이번 세션 범위 밖)

- API 키는 **평문 config.json**에 저장됨. ROADMAP 8단계에서 OS keyring으로 이전.
- `chunk_vectors` 가상 테이블은 sqlite-vec 로드 실패 시 자동 비활성화. 이번 범위에서는 사용 안 함.
- 아이콘 파일이 비어 있어 `tauri build`(릴리스)는 실패한다. `src-tauri/icons/README.md` 참고.
