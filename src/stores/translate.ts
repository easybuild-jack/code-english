import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { api, type ChatMessage, type HistoryItem, type Mode, type Scene, type TranslateResponse } from "../api";

export type TranslateStatus = "idle" | "loading" | "done" | "error";

export const useTranslate = defineStore("translate", () => {
  const mode = ref<Mode>("translate");
  const source = ref("");
  const scene = ref<Scene>("work");
  const status = ref<TranslateStatus>("idle");
  const error = ref<string | null>(null);
  const response = ref<TranslateResponse | null>(null);
  /** 已收藏词 → 遇到次数（小写键），用于结果区显示 ★ 和「第 N 次遇到」 */
  const favoriteLookup = ref<Map<string, number>>(new Map());
  /** Ask 模式的对话，只在内存里，切模式或点「新对话」清空 */
  const askMessages = ref<ChatMessage[]>([]);

  const result = computed(() => response.value?.result ?? null);
  const canTranslate = computed(() => source.value.trim().length > 0 && status.value !== "loading");

  function setMode(m: Mode) {
    if (m === mode.value) return;
    mode.value = m;
    status.value = "idle";
    error.value = null;
    if (m === "translate") askMessages.value = [];
  }

  function clearAsk() {
    askMessages.value = [];
    status.value = "idle";
    error.value = null;
  }

  async function refreshFavoriteLookup() {
    const pairs = await api.favoriteLookup();
    favoriteLookup.value = new Map(pairs);
  }

  /** 启动时把档案里的默认场景带进来 */
  async function loadDefaultScene() {
    const profile = await api.getProfile();
    scene.value = profile.default_scene;
  }

  /** Enter 触发：按当前模式翻译或提问 */
  async function run() {
    if (!canTranslate.value) return;
    if (mode.value === "ask") return runAsk();
    status.value = "loading";
    error.value = null;
    try {
      response.value = await api.translate(source.value, scene.value);
      status.value = "done";
      await refreshFavoriteLookup();
    } catch (e) {
      error.value = typeof e === "string" ? e : String(e);
      status.value = "error";
    }
  }

  async function runAsk() {
    const question = source.value.trim();
    askMessages.value.push({ role: "user", content: question });
    source.value = "";
    status.value = "loading";
    error.value = null;
    try {
      const r = await api.ask(askMessages.value, scene.value);
      askMessages.value.push({ role: "assistant", content: r.answer });
      status.value = "done";
    } catch (e) {
      // 失败时把问题退回输入框，方便改了重发
      askMessages.value.pop();
      source.value = question;
      error.value = typeof e === "string" ? e : String(e);
      status.value = "error";
    }
  }

  /** 从历史恢复，不重新调模型 */
  function restore(item: HistoryItem) {
    setMode("translate");
    source.value = item.source_text;
    scene.value = item.scene;
    const keywords = safeParse(item.keywords_json, []);
    response.value = {
      history_id: item.id,
      provider_id: item.provider_id,
      model: item.model,
      seen_favorites: [],
      result: {
        translation: item.translation,
        sentences: [item.translation],
        keywords,
        raw_fallback: false,
      },
    };
    status.value = "done";
    error.value = null;
  }

  function isFavorited(text: string) {
    return favoriteLookup.value.has(text.trim().toLowerCase());
  }

  function seenCount(text: string) {
    return favoriteLookup.value.get(text.trim().toLowerCase()) ?? 0;
  }

  return {
    mode,
    source,
    scene,
    status,
    error,
    response,
    result,
    askMessages,
    canTranslate,
    setMode,
    clearAsk,
    run,
    restore,
    refreshFavoriteLookup,
    loadDefaultScene,
    isFavorited,
    seenCount,
  };
});

function safeParse<T>(json: string, fallback: T): T {
  try {
    return JSON.parse(json) as T;
  } catch {
    return fallback;
  }
}
