import { createContext, useContext } from "react";

type Lang = "ko" | "en";

const dict = {
  ko: {
    "nav.search": "검색",
    "nav.library": "라이브러리",
    "nav.settings": "설정",
    "search.placeholder": "책 제목을 입력하세요 (예: Clean Code)",
    "search.start": "합성 시작",
    "search.hint":
      "Tavily로 리뷰·요약 글을 수집하고 6단계 파이프라인을 거쳐 통합 요약을 생성합니다.",
    "library.empty": "아직 합성한 책이 없습니다. 첫 책을 추가해 보세요.",
    "library.open": "열기",
    "library.failed_retry": "재시도",
    "stage.search": "검색",
    "stage.collect": "수집",
    "stage.structure": "구조화",
    "stage.cluster": "클러스터링",
    "stage.synthesis": "합성",
    "stage.ready": "완료",
    "status.collecting": "진행 중",
    "status.synthesizing": "합성 중",
    "status.ready": "완료",
    "status.failed": "실패",
    "viewer.no_chapter": "선택된 챕터가 없습니다.",
    "viewer.sources": "출처",
    "viewer.open_source": "원본 보기",
    "viewer.todo_synthesis":
      "TODO: 합성(Synthesis) 단계가 아직 구현되지 않았습니다. 수집된 첫 소스의 미리보기를 표시합니다.",
    "density.light": "Light",
    "density.medium": "Medium",
    "density.deep": "Deep",
    "settings.title": "설정",
    "settings.provider": "AI Provider",
    "settings.model": "모델",
    "settings.anthropic_key": "Anthropic API Key",
    "settings.openai_key": "OpenAI API Key",
    "settings.tavily_key": "Tavily API Key",
    "settings.ollama_endpoint": "Ollama Endpoint",
    "settings.language": "UI 언어",
    "settings.save": "저장",
    "settings.test": "테스트 호출",
    "settings.test_ok": "성공",
    "settings.test_fail": "실패",
    "settings.key_set": "설정됨",
    "settings.key_unset": "미설정",
    "common.loading": "불러오는 중...",
    "common.cancel": "취소",
  },
  en: {
    "nav.search": "Search",
    "nav.library": "Library",
    "nav.settings": "Settings",
    "search.placeholder": "Enter book title (e.g., Clean Code)",
    "search.start": "Start weaving",
    "search.hint":
      "Collects reviews/summaries via Tavily and runs the 6-stage pipeline.",
    "library.empty": "No books yet. Add your first one.",
    "library.open": "Open",
    "library.failed_retry": "Retry",
    "stage.search": "Search",
    "stage.collect": "Collect",
    "stage.structure": "Structure",
    "stage.cluster": "Cluster",
    "stage.synthesis": "Synthesis",
    "stage.ready": "Ready",
    "status.collecting": "In progress",
    "status.synthesizing": "Synthesizing",
    "status.ready": "Ready",
    "status.failed": "Failed",
    "viewer.no_chapter": "No chapter selected.",
    "viewer.sources": "Sources",
    "viewer.open_source": "Open source",
    "viewer.todo_synthesis":
      "TODO: Synthesis stage not implemented yet. Showing a preview of the first collected source.",
    "density.light": "Light",
    "density.medium": "Medium",
    "density.deep": "Deep",
    "settings.title": "Settings",
    "settings.provider": "AI Provider",
    "settings.model": "Model",
    "settings.anthropic_key": "Anthropic API Key",
    "settings.openai_key": "OpenAI API Key",
    "settings.tavily_key": "Tavily API Key",
    "settings.ollama_endpoint": "Ollama Endpoint",
    "settings.language": "UI Language",
    "settings.save": "Save",
    "settings.test": "Test call",
    "settings.test_ok": "OK",
    "settings.test_fail": "Failed",
    "settings.key_set": "set",
    "settings.key_unset": "not set",
    "common.loading": "Loading...",
    "common.cancel": "Cancel",
  },
} as const;

export type DictKey = keyof (typeof dict)["ko"];

export interface I18nValue {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: DictKey) => string;
}

export const I18nContext = createContext<I18nValue>({
  lang: "ko",
  setLang: () => {},
  t: (k) => k,
});

export function useT() {
  return useContext(I18nContext);
}

export function translate(lang: Lang, key: DictKey): string {
  return (dict[lang] as Record<string, string>)[key] ?? key;
}

export const STORAGE_LANG_KEY = "bookweaver.lang";
