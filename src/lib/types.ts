export type BookStatus =
  | "collecting"
  | "synthesizing"
  | "ready"
  | "failed"
  | string;

export interface Book {
  id: number;
  title: string;
  author: string | null;
  language: string | null;
  status: BookStatus;
  created_at: string;
  updated_at: string;
  error_message: string | null;
}

export interface Source {
  id: number;
  book_id: number;
  url: string;
  title: string | null;
  site: string | null;
  fetched_at: string;
}

export interface Chapter {
  id: number;
  book_id: number;
  order_idx: number;
  title: string;
  summary_light: string | null;
  summary_med: string | null;
  summary_deep: string | null;
}

export type Stage =
  | "search"
  | "collect"
  | "structure"
  | "cluster"
  | "synthesis"
  | "ready";

export type StageStatus = "started" | "done" | "failed";

export interface ProgressEvent {
  book_id: number;
  stage: Stage;
  status: StageStatus;
  message?: string;
}

export type ProviderKind = "claude" | "openai" | "ollama";

export interface Settings {
  provider: ProviderKind;
  model: string;
  language: string;
  has_anthropic_key: boolean;
  has_openai_key: boolean;
  has_tavily_key: boolean;
  ollama_endpoint: string | null;
}

export interface SettingsUpdate {
  provider?: ProviderKind;
  model?: string;
  language?: string;
  anthropic_api_key?: string;
  openai_api_key?: string;
  tavily_api_key?: string;
  ollama_endpoint?: string;
}

export type Density = "light" | "medium" | "deep";

export interface TestProviderResult {
  ok: boolean;
  message: string;
}
