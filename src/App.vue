<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NConfigProvider, NMessageProvider, NDialogProvider, zhCN, dateZhCN } from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "./stores/settings";
import { useTranslate } from "./stores/translate";
import { useDock } from "./stores/dock";
import { useTheme } from "./composables/useTheme";
import TopBar from "./components/TopBar.vue";
import BottomBar from "./components/BottomBar.vue";
import TranslateView from "./views/TranslateView.vue";
import HistoryView from "./views/HistoryView.vue";
import FavoritesView from "./views/FavoritesView.vue";
import SettingsView from "./views/SettingsView.vue";

export type Page = "translate" | "history" | "favorites" | "settings";

const settings = useSettings();
const translate = useTranslate();
const dock = useDock();
const { naiveTheme, naiveOverrides } = useTheme();
const page = ref<Page>("translate");
const ready = ref(false);
const focused = ref(true);

onMounted(async () => {
  await settings.load();
  // 只在启动时读一次档案里的默认场景，之后由用户在顶栏切换
  await translate.loadDefaultScene().catch(() => {});
  ready.value = true;

  const win = getCurrentWindow();
  await win.setAlwaysOnTop(settings.alwaysOnTop);
  await win.onFocusChanged(({ payload }) => {
    focused.value = payload;
    // 未置顶：失焦吸附到边缘；拿到焦点（快捷键 / 托盘 / 点击露出的边）就弹回
    if (payload) void dock.undock(false);
    else void dock.dock();
  });
  await listen("open-settings", () => (page.value = "settings"));
});
</script>

<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="naiveOverrides"
    :locale="zhCN"
    :date-locale="dateZhCN"
    abstract
  >
    <NMessageProvider placement="top" :max="2">
      <NDialogProvider>
        <div
          v-if="ready"
          class="win"
          :class="dock.docked && `docked-${settings.dockEdge}`"
          :style="{ opacity: focused || dock.docked ? 1 : settings.inactiveOpacity / 100 }"
        >
          <TopBar v-if="page === 'translate'" />
          <main class="page">
            <TranslateView v-if="page === 'translate'" @open-settings="page = 'settings'" />
            <HistoryView v-else-if="page === 'history'" @back="page = 'translate'" />
            <FavoritesView v-else-if="page === 'favorites'" @back="page = 'translate'" />
            <SettingsView v-else @back="page = 'translate'" />
          </main>
          <BottomBar :page="page" @navigate="page = $event" />
        </div>
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
.win {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--r-win);
  overflow: hidden;
  transition: background-color 120ms, opacity 120ms;
  position: relative;
}
/* 吸附时窗口本体隐形，只在露出的 8px 里画一枚小把手。吸附在右侧时露的是窗口左边，反之亦然 */
.win.docked-right,
.win.docked-left {
  background: transparent;
  border-color: transparent;
}
.win.docked-right > *,
.win.docked-left > * {
  visibility: hidden;
}
.win.docked-right::before,
.win.docked-left::before {
  content: "";
  position: absolute;
  top: 50%;
  width: 5px;
  height: 64px;
  transform: translateY(-50%);
  border-radius: 3px;
  background: var(--accent);
  opacity: 0.85;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.25), 0 2px 6px rgba(0, 0, 0, 0.35);
  z-index: 10;
}
.win.docked-right::before { left: 2px; }
.win.docked-left::before { right: 2px; }
.page {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
