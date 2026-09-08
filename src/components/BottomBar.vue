<script setup lang="ts">
import type { Page } from "../App.vue";

defineProps<{ page: Page }>();
defineEmits<{ navigate: [page: Page] }>();

const tabs: { key: Page; label: string }[] = [
  { key: "translate", label: "翻译" },
  { key: "history", label: "历史" },
  { key: "favorites", label: "收藏" },
  { key: "settings", label: "设置" },
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
      {{ tab.label }}
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
  border: 0;
  background: transparent;
  color: var(--text-2);
  font: inherit;
  font-size: var(--fs-sm);
  padding: 6px 12px;
  border-radius: var(--r-ctl);
  cursor: pointer;
  transition: background-color 120ms, color 120ms;
}
.tab:hover { background: var(--bg-3); color: var(--text); }
.tab.active { color: var(--accent); }
</style>
