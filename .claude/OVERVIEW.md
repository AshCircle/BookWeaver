# BookWeaver — Overview

## Vision

> 인터넷에 흩어진 책 요약 조각들을 수집·정제·재구성하여, 하나의 구조화된 책 요약으로 복원하는 **로컬 설치형 AI 애플리케이션**.

BookWeaver는 특정 책에 대한 블로그 리뷰, 독후감, 챕터 요약, 노트 등을 수집하고, 이를 AI를 통해 구조적으로 재조합하여 하나의 통합된 책 요약본을 생성한다.

핵심 목표:
- 책의 목차 구조 복원
- 챕터 단위 내용 정리
- 중복 제거
- 여러 요약 글 통합
- 일관된 흐름 생성

## Context

현재 저장소는 사실상 비어있다 (`.claude/draft.md` 외에는 코드가 없음). 따라서 이 설계 문서들은 "어떻게 구현할지"가 아닌 **"어떤 스택과 모듈로 처음부터 구성할지"** 를 정의한다.

## 핵심 결정

- **Tauri 데스크톱 앱** (Rust 백엔드 + React/TS/Tailwind 프런트엔드)
- **하이브리드 AI**: 사용자가 Claude / OpenAI / Ollama 중 설정에서 선택
- **수집 방식**: 검색 API (Tavily/Brave) + 직접 스크래핑
- **저장소**: SQLite + sqlite-vec (메타데이터·본문·벡터 모두 단일 DB)
- **언어**: 한국어 우선, 영어 지원
- **MVP 범위**: Search → Collection → Structure → Cluster → Synthesis → Viewer **종단간**

## 관련 문서

- 사용자 시점 기능 정의 → [USECASES.md](USECASES.md)
- 모듈/DB/AI 추상화 → [ARCHITECTURE.md](ARCHITECTURE.md)
- 6단계 합성 파이프라인 → [PIPELINE.md](PIPELINE.md)
- 화면·라우트·UX → [FRONTEND.md](FRONTEND.md)
- 구현 순서 및 검증 → [ROADMAP.md](ROADMAP.md)
- 원본 비전 문서 → [draft.md](draft.md)
