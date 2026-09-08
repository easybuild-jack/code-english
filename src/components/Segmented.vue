<script setup lang="ts" generic="T extends string">
defineProps<{ options: readonly { value: T; label: string }[] }>();
const model = defineModel<T>({ required: true });
</script>

<template>
  <div class="seg">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      class="seg-item"
      :class="{ active: model === o.value }"
      @click="model = o.value"
    >
      {{ o.label }}
    </button>
  </div>
</template>

<style scoped>
.seg {
  display: inline-flex;
  padding: 2px;
  gap: 2px;
  background: var(--bg-2);
  border: 1px solid var(--border);
  border-radius: var(--r-ctl);
}
.seg-item {
  border: 0;
  background: transparent;
  color: var(--text-2);
  font: inherit;
  font-size: var(--fs-sm);
  padding: 3px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 120ms, color 120ms;
}
.seg-item:hover { color: var(--text); }
.seg-item.active { background: var(--bg); color: var(--accent); box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08); }
.seg-item:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
