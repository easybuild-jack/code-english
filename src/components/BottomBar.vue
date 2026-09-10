<script setup lang="ts">
import type { Component } from "vue";
import { Languages, Star, History, Settings } from "lucide-vue-next";
import type { Page } from "../App.vue";

defineProps<{ page: Page }>();
defineEmits<{ navigate: [page: Page] }>();

const tabs: { key: Page; label: string; icon: Component }[] = [
  { key: "translate", label: "翻译", icon: Languages },
  { key: "favorites", label: "收藏", icon: Star },
  { key: "history", label: "历史", icon: History },
  { key: "settings", label: "设置", icon: Settings },
];
</script>

<template>
  <nav class="bottombar">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      class="tab"
      :class="{ active: page === tab.key }"
      @click="$emit('navigate', tab.key)"
    >
      <component :is="tab.icon" :size="15" :stroke-width="1.75" />
      <span>{{ tab.label }}</span>
    </button>
  </nav>
</template>

<style scoped>
.bottombar {
  height: var(--bar-h);
  flex: none;
  display: flex;
  justify-content: space-around;
  align-items: center;
  border-top: 1px solid var(--border);
}
.tab {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  border: 0;
  background: transparent;
  color: var(--text-2);
  font: inherit;
  font-size: var(--fs-sm);
  padding: 6px 4px;
  border-radius: var(--r-ctl);
  cursor: pointer;
  transition: background-color 120ms, color 120ms;
}
.tab:hover { background: var(--bg-3); color: var(--text); }
.tab.active { color: var(--accent); }
</style>
