// 与 src-tauri/src/db/models.rs、commands.rs 中的结构一一对应

export type Scene = "work" | "workplace" | "daily" | "shopping" | "travel";

/** 工作是核心场景（写代码、对 AI 编程助手下指令），其余为生活场景 */
export const SCENES: { value: Scene; label: string }[] = [
  { value: "work", label: "工作" },
  { value: "workplace", label: "职场" },
  { value: "daily", label: "日常" },
  { value: "shopping", label: "购物" },
  { value: "travel", label: "旅行" },
];

export interface Profile {
  occupation: string;
  tech_stack: string;
  industry: string;
  default_scene: Scene;
}

export interface ProviderConfig {
  id: string;
  display_name: string;
  kind: "openai_compatible" | "anthropic" | "gemini";
  base_url: string;
  model: string;
  enabled: boolean;
  is_default: boolean;
}

export interface ProviderView extends ProviderConfig {
  has_api_key: boolean;
}

export interface Keyword {
  word: string;
  note: string;
}

export interface TranslationResult {
  translation: string;
  sentences: string[];
  keywords: Keyword[];
  raw_fallback: boolean;
}

export type Mode = "translate" | "ask";

export const MODES: { value: Mode; label: string }[] = [
  { value: "translate", label: "翻译" },
  { value: "ask", label: "Ask" },
];

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

export interface AskResponse {
  answer: string;
  model: string;
  refused: boolean;
}

export interface TranslateResponse {
  history_id: number;
  result: TranslationResult;
  provider_id: string;
  model: string;
  seen_favorites: string[];
}

export interface HistoryItem {
  id: number;
  source_text: string;
  translation: string;
  keywords_json: string;
  scene: Scene;
  provider_id: string;
  model: string;
  created_at: string;
}

export type FavoriteKind = "word" | "phrase" | "sentence";

export interface Favorite {
  id: number;
  kind: FavoriteKind;
  text: string;
  meaning: string;
  domain: string;
  example: string;
  seen_count: number;
  mastered: boolean;
  created_at: string;
}

export interface NewFavorite {
  kind: FavoriteKind;
  text: string;
  meaning?: string;
  domain?: string;
  example?: string;
}

export interface FavoriteFilter {
  kind?: FavoriteKind;
  mastered?: boolean;
  query?: string;
  order_by_seen?: boolean;
}

/** PRD 6：按词数自动判定收藏类型 */
export function guessKind(text: string): FavoriteKind {
  const t = text.trim();
  const words = t.split(/\s+/).filter(Boolean).length;
  if (words <= 1) return "word";
  if (words <= 5 && !/[.!?]$/.test(t)) return "phrase";
  return "sentence";
}
