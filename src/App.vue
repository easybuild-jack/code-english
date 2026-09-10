<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NConfigProvider, NMessageProvider, NDialogProvider, zhCN, dateZhCN } from "naive-ui";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./api";
import { useSettings } from "./stores/settings";
import { useTranslate } from "./stores/translate";
import { useTheme } from "./composables/useTheme";
import TopBar from "./components/TopBar.vue";
import BottomBar from "./components/BottomBar.vue";
import TranslateView from "./views/TranslateView.vue";
import FavoritesView from "./views/FavoritesView.vue";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";

export type Page = "translate" | "favorites" | "history" | "settings";

const settings = useSettings();
const translate = useTranslate();
const { naiveTheme, naiveOverrides } = useTheme();
const page = ref<Page>("translate");
const ready = ref(false);
const focused = ref(true);

onMounted(async () => {
  await settings.load();
  if (settings.syncBaseUrl) {
    void api.syncFavorites(settings.syncBaseUrl).catch(() => {});
  }
  // 只在启动时读一次档案里的默认场景，之后由用户在顶栏切换
  await translate.loadDefaultScene().catch(() => {});
  ready.value = true;

  const win = getCurrentWindow();
  await win.setAlwaysOnTop(settings.alwaysOnTop);
  await win.onFocusChanged(({ payload }) => {
    focused.value = payload;
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
          :style="{ opacity: focused ? 1 : settings.inactiveOpacity / 100 }"
        >
          <TopBar v-if="page === 'translate'" />
          <main class="page">
            <TranslateView v-if="page === 'translate'" @open-settings="page = 'settings'" />
            <FavoritesView v-else-if="page === 'favorites'" @back="page = 'translate'" />
            <HistoryView v-else-if="page === 'history'" @back="page = 'translate'" />
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
.page {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
