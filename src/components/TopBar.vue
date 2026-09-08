<script setup lang="ts">
import { computed } from "vue";
import { NSelect, NDropdown } from "naive-ui";
import { Pin, PinOff, Ellipsis } from "lucide-vue-next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MODES, SCENES } from "../api";
import { useSettings } from "../stores/settings";
import { useTranslate } from "../stores/translate";
import Segmented from "./Segmented.vue";

const settings = useSettings();
const t = useTranslate();
const win = getCurrentWindow();

const sceneOptions = SCENES.map((s) => ({ label: s.label, value: s.value }));

async function togglePin() {
  settings.alwaysOnTop = !settings.alwaysOnTop;
  await win.setAlwaysOnTop(settings.alwaysOnTop);
}

const menu = computed(() => [
  { label: settings.theme === "dark" ? "切到浅色" : "切到深色", key: "theme" },
  {
    label: "字号",
    key: "size",
    children: [
      { label: "小", key: "size:sm" },
      { label: "中", key: "size:md" },
      { label: "大", key: "size:lg" },
    ],
  },
  { label: "隐藏到托盘", key: "hide" },
]);

async function onMenu(key: string) {
  if (key === "theme") settings.toggleTheme();
  else if (key.startsWith("size:")) settings.fontSize = key.slice(5) as "sm" | "md" | "lg";
  else if (key === "hide") await win.hide();
}
</script>

<template>
  <header class="topbar" data-tauri-drag-region>
    <Segmented :model-value="t.mode" :options="MODES" class="mode" @update:model-value="t.setMode" />
    <span class="scene-label" data-tauri-drag-region>场景</span>
    <NSelect
      v-model:value="t.scene"
      :options="sceneOptions"
      size="small"
      class="scene"
      :consistent-menu-width="false"
    />
    <span class="spacer" data-tauri-drag-region />
    <button class="icon-btn" :title="settings.alwaysOnTop ? '取消置顶' : '置顶'" @click="togglePin">
      <Pin v-if="settings.alwaysOnTop" :size="18" :stroke-width="1.75" />
      <PinOff v-else :size="18" :stroke-width="1.75" />
    </button>
    <NDropdown :options="menu" trigger="click" @select="onMenu">
      <button class="icon-btn" title="菜单">
        <Ellipsis :size="18" :stroke-width="1.75" />
      </button>
    </NDropdown>
  </header>
</template>

<style scoped>
.topbar {
  height: var(--bar-h);
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px 0 16px;
  border-bottom: 1px solid var(--border);
}
.mode { flex: none; margin-right: 4px; }
.mode :deep(.seg-item) { padding: 2px 8px; font-size: var(--fs-xs); }
.scene-label {
  font-size: var(--fs-sm);
  color: var(--text-2);
  margin-right: -2px;
}
.scene { width: 88px; }
.spacer { flex: 1; height: 100%; }
</style>
