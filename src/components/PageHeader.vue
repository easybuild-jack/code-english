<script setup lang="ts">
import { ArrowLeft } from "lucide-vue-next";

defineProps<{ title: string }>();
defineEmits<{ back: [] }>();
</script>

<template>
  <header class="page-header" data-tauri-drag-region>
    <button class="back" @click="$emit('back')">
      <ArrowLeft :size="16" :stroke-width="1.75" />
      <span>返回翻译</span>
    </button>
    <span class="title">{{ title }}</span>
    <span class="actions"><slot name="actions" /></span>
  </header>
</template>

<style scoped>
.page-header {
  height: var(--bar-h);
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px 0 8px;
  border-bottom: 1px solid var(--border);
}
.back {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 0;
  background: transparent;
  color: var(--text-2);
  font: inherit;
  font-size: var(--fs-sm);
  padding: 4px 8px;
  border-radius: var(--r-ctl);
  cursor: pointer;
}
.back:hover { background: var(--bg-3); color: var(--text); }
.title { flex: 1; text-align: center; font-size: var(--fs-md); font-weight: 600; }
.actions { display: inline-flex; gap: 4px; min-width: 60px; justify-content: flex-end; }
</style>
