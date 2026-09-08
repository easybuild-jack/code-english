<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NInput, NButton, NEmpty, useDialog, useMessage } from "naive-ui";
import { Trash2 } from "lucide-vue-next";
import { api, SCENES, type HistoryItem } from "../api";
import { useTranslate } from "../stores/translate";
import PageHeader from "../components/PageHeader.vue";

const emit = defineEmits<{ back: [] }>();

const t = useTranslate();
const dialog = useDialog();
const message = useMessage();
const items = ref<HistoryItem[]>([]);
const query = ref("");
const loading = ref(false);

async function load() {
  loading.value = true;
  try {
    items.value = await api.listHistory(query.value || undefined);
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
}

onMounted(load);

const groups = computed(() => {
  const map = new Map<string, HistoryItem[]>();
  for (const it of items.value) {
    const day = dayLabel(it.created_at);
    if (!map.has(day)) map.set(day, []);
    map.get(day)!.push(it);
  }
  return [...map.entries()];
});

function dayLabel(ts: string) {
  const d = new Date(ts.replace(" ", "T"));
  const today = new Date();
  const diff = Math.floor((startOfDay(today) - startOfDay(d)) / 86400000);
  if (diff === 0) return "今天";
  if (diff === 1) return "昨天";
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}
function startOfDay(d: Date) {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}
function timeOf(ts: string) {
  return ts.slice(11, 16);
}
function sceneLabel(v: string) {
  return SCENES.find((s) => s.value === v)?.label ?? v;
}

function open(item: HistoryItem) {
  t.restore(item);
  emit("back");
}

async function remove(item: HistoryItem) {
  await api.deleteHistory(item.id);
  items.value = items.value.filter((i) => i.id !== item.id);
}

function clearAll() {
  dialog.warning({
    title: "清空历史",
    content: "历史就是练习记录，清空后不可恢复。确定？",
    positiveText: "清空",
    negativeText: "取消",
    onPositiveClick: async () => {
      await api.clearHistory();
      items.value = [];
    },
  });
}
</script>

<template>
  <div class="history">
    <PageHeader title="历史" @back="emit('back')">
      <template #actions>
        <NButton text size="small" :disabled="!items.length" @click="clearAll">清空</NButton>
      </template>
    </PageHeader>

    <div class="search">
      <NInput v-model:value="query" size="small" clearable placeholder="搜索原文或翻译" @update:value="load" />
    </div>

    <div class="list">
      <NEmpty v-if="!loading && !items.length" description="还没有翻译记录" style="margin-top: 48px" />
      <template v-for="[day, list] in groups" :key="day">
        <div class="day">{{ day }}</div>
        <div v-for="it in list" :key="it.id" class="item" @click="open(it)">
          <div class="src">{{ it.source_text }}</div>
          <div class="dst selectable">{{ it.translation }}</div>
          <div class="meta">
            <span>{{ sceneLabel(it.scene) }} · {{ it.provider_id }} · {{ timeOf(it.created_at) }}</span>
            <button class="icon-btn" title="删除" @click.stop="remove(it)">
              <Trash2 :size="14" :stroke-width="1.75" />
            </button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.history { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.search { padding: 12px 16px 8px; flex: none; }
.list { flex: 1; min-height: 0; overflow: auto; }
.day {
  padding: 8px 16px 4px;
  font-size: var(--fs-xs);
  color: var(--text-3);
}
.item {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background-color 120ms;
}
.item:hover { background: var(--bg-3); }
.src { font-size: var(--fs-sm); color: var(--text-2); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.dst {
  font-family: var(--font-en);
  font-size: var(--fs-md);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 4px;
  font-size: var(--fs-xs);
  color: var(--text-3);
}
.meta .icon-btn { width: 20px; height: 20px; opacity: 0; }
.item:hover .meta .icon-btn { opacity: 1; }
</style>
