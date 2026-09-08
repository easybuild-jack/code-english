// 本机设置（主题、字号、快捷键、朗读等），存 tauri-plugin-store 的 settings.json。
// 历史 / 收藏 / 档案 / 模型走 SQLite，不在这里。

import { defineStore } from "pinia";
import { LazyStore } from "@tauri-apps/plugin-store";
import { ref, watch } from "vue";

export type Theme = "light" | "dark" | "system";
export type FontSize = "sm" | "md" | "lg";
export type EnFont = "sans" | "mono";
export type Accent = "us" | "uk";
export type DockEdge = "left" | "right";

export interface Hotkeys {
  toggleWindow: string;
  translate: string;
  speak: string;
  favoriteSentence: string;
  nextScene: string;
  toggleTheme: string;
}

/** 设置结构版本，用于迁移旧默认值 */
const SETTINGS_VERSION = 2;

export const DEFAULT_HOTKEYS: Hotkeys = {
  toggleWindow: "CmdOrCtrl+Shift+E",
  // 输入框里 Enter 翻译、Ctrl+Enter 换行（TranslateView 处理换行）
  translate: "Enter",
  speak: "CmdOrCtrl+P",
  favoriteSentence: "CmdOrCtrl+D",
  nextScene: "CmdOrCtrl+Tab",
  toggleTheme: "CmdOrCtrl+Shift+T",
};

interface Persisted {
  version?: number;
  theme: Theme;
  fontSize: FontSize;
  enFont: EnFont;
  inactiveOpacity: number;
  hotkeys: Hotkeys;
  ttsAccent: Accent;
  autostart: boolean;
  alwaysOnTop: boolean;
  /** 未置顶时失焦吸附到哪一侧 */
  dockEdge: DockEdge;
}

const DEFAULTS: Persisted = {
  theme: "system",
  fontSize: "md",
  enFont: "sans",
  inactiveOpacity: 100,
  hotkeys: DEFAULT_HOTKEYS,
  ttsAccent: "us",
  autostart: true,
  alwaysOnTop: true,
  dockEdge: "right",
};

const store = new LazyStore("settings.json");

export const useSettings = defineStore("settings", () => {
  const theme = ref<Theme>(DEFAULTS.theme);
  const fontSize = ref<FontSize>(DEFAULTS.fontSize);
  const enFont = ref<EnFont>(DEFAULTS.enFont);
  const inactiveOpacity = ref(DEFAULTS.inactiveOpacity);
  const hotkeys = ref<Hotkeys>({ ...DEFAULT_HOTKEYS });
  const ttsAccent = ref<Accent>(DEFAULTS.ttsAccent);
  const autostart = ref(DEFAULTS.autostart);
  const alwaysOnTop = ref(DEFAULTS.alwaysOnTop);
  const dockEdge = ref<DockEdge>(DEFAULTS.dockEdge);
  const loaded = ref(false);

  async function load() {
    const saved = (await store.get<Partial<Persisted>>("settings")) ?? {};
    theme.value = saved.theme ?? DEFAULTS.theme;
    fontSize.value = saved.fontSize ?? DEFAULTS.fontSize;
    enFont.value = saved.enFont ?? DEFAULTS.enFont;
    inactiveOpacity.value = saved.inactiveOpacity ?? DEFAULTS.inactiveOpacity;
    hotkeys.value = { ...DEFAULT_HOTKEYS, ...(saved.hotkeys ?? {}) };
    // v1 → v2：翻译默认键从 Ctrl+Enter 改为 Enter，只迁移仍是旧默认值的
    if ((saved.version ?? 1) < 2 && hotkeys.value.translate === "CmdOrCtrl+Enter") {
      hotkeys.value.translate = DEFAULT_HOTKEYS.translate;
    }
    ttsAccent.value = saved.ttsAccent ?? DEFAULTS.ttsAccent;
    autostart.value = saved.autostart ?? DEFAULTS.autostart;
    alwaysOnTop.value = saved.alwaysOnTop ?? DEFAULTS.alwaysOnTop;
    dockEdge.value = saved.dockEdge ?? DEFAULTS.dockEdge;
    loaded.value = true;
  }

  async function persist() {
    const data: Persisted = {
      version: SETTINGS_VERSION,
      theme: theme.value,
      fontSize: fontSize.value,
      enFont: enFont.value,
      inactiveOpacity: inactiveOpacity.value,
      hotkeys: hotkeys.value,
      ttsAccent: ttsAccent.value,
      autostart: autostart.value,
      alwaysOnTop: alwaysOnTop.value,
      dockEdge: dockEdge.value,
    };
    await store.set("settings", data);
    await store.save();
  }

  watch(
    [theme, fontSize, enFont, inactiveOpacity, hotkeys, ttsAccent, autostart, alwaysOnTop, dockEdge],
    () => {
      if (loaded.value) void persist();
    },
    { deep: true },
  );

  function toggleTheme() {
    const resolved = resolveTheme(theme.value);
    theme.value = resolved === "dark" ? "light" : "dark";
  }

  function resetHotkeys() {
    hotkeys.value = { ...DEFAULT_HOTKEYS };
  }

  return {
    theme,
    fontSize,
    enFont,
    inactiveOpacity,
    hotkeys,
    ttsAccent,
    autostart,
    alwaysOnTop,
    dockEdge,
    loaded,
    load,
    toggleTheme,
    resetHotkeys,
  };
});

export function resolveTheme(t: Theme): "light" | "dark" {
  if (t !== "system") return t;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
