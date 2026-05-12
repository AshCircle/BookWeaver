# BookWeaver

> 인터넷에 흩어진 책 요약 조각들을 수집·정제·재구성하여,
> 하나의 구조화된 책 요약으로 복원하는 로컬 AI 애플리케이션.

---

# Overview

BookWeaver는 특정 책에 대한 블로그 리뷰, 독후감, 챕터 요약, 노트 등을 수집하고,
이를 AI를 통해 구조적으로 재조합하여 하나의 통합된 책 요약본을 생성하는 설치형 애플리케이션이다.

핵심 목표:

* 책의 목차 구조 복원
* 챕터 단위 내용 정리
* 중복 제거
* 여러 요약 글 통합
* 일관된 흐름 생성

---

# Main Workflow

## 1. Book Search

사용자가 책 제목 입력.

예:

* Clean Code
* Atomic Habits
* Designing Data-Intensive Applications

---

## 2. Web Collection

웹에서 관련 데이터를 수집.

수집 대상:

* 블로그 요약 글
* 독후감
* 챕터 정리
* 공개 노트
* 기술 블로그

---

## 3. Structure Extraction

수집한 문서에서:

* 목차 추출
* 챕터 후보 추출
* 섹션 제목 정규화
* 유사 챕터 병합

수행.

---

## 4. Semantic Clustering

유사한 내용을 다루는 문단끼리 그룹화.

예:

* ACID
* 트랜잭션
* consistency

관련 설명 통합.

---

## 5. Synthesis

AI가:

* 핵심 정보 추출
* 중복 제거
* 흐름 재정렬
* 설명 통합

을 수행하여 최종 요약 생성.

---

## 6. Viewer Rendering

생성된 내용을 읽기 좋은 뷰어 형태로 제공.

---

# Viewer Features

## 목차 기반 탐색

```text
Book
 ├── Chapter 1
 ├── Chapter 2
 └── Chapter 3
```

---

## Reading Mode

* dark mode
* typography 설정
* 집중 모드

---

## Summary Density

* Light
* Medium
* Deep

요약 강도 조절.

---

## Source Traceability

각 문단의 출처 표시.

예:

```text
Sources:
- blog A
- review B
```

---

# Core Challenges

## Token Limit

책 전체를 직접 처리할 경우:

* 비용 증가
* context overflow
* latency 증가

문제 발생 가능.

---

## Information Quality

블로그 요약 자체가 부정확할 수 있음.

---

## Hallucination

존재하지 않는 내용을 생성할 가능성 존재.

---

# Vision

> BookWeaver는 인터넷에 흩어진 독서 지식을 엮어,
> 하나의 구조화된 책 경험으로 복원하는 로컬 AI 리딩 엔진이다.

