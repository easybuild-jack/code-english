<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NEmpty, useMessage } from "naive-ui";
import { Trash2 } from "lucide-vue-next";
import { api, type Favorite } from "../api";
import { useTranslate } from "../stores/translate";
import PageHeader from "../components/PageHeader.vue";

const emit = defineEmits<{ back: [] }>();

const message = useMessage();
const t = useTranslate();

const all = ref<Favorite[]>([]);

async function load() {
  try {
    all.value = await api.listFavorites();
  } catch (e) {
    message.error(String(e));
  }
}
onMounted(load);

async function remove(f: Favorite) {
  try {
    await api.deleteFavorite(f.id);
    all.value = all.value.filter((x) => x.id !== f.id);
    await t.refreshFavoriteLookup();
  } catch (e) {
    message.error(String(e));
  }
}

function dateOf(ts: string) {
  const d = new Date(ts.replace(" ", "T"));
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}
</script>

<template>
  <div class="favorites">
    <PageHeader title="收藏" @back="emit('back')" />

    <div class="list">
      <NEmpty v-if="!all.length" description="这里还是空的，翻译结果里点单词就能收藏" style="margin-top: 48px" />
      <div v-for="f in all" :key="f.id" class="item">
        <div class="head">
          <span class="text selectable">{{ f.text }}</span>
          <button class="icon-btn danger" title="删除" @click="remove(f)">
            <Trash2 :size="16" :stroke-width="1.75" />
          </button>
        </div>
        <div v-if="f.ipa" class="ipa">{{ f.accent === "uk" ? "英" : "美" }} /{{ f.ipa.replace(/^\/|\/$/g, "") }}/</div>
        <div v-if="f.meaning" class="meaning">{{ f.meaning }}</div>
        <div class="meta">遇到 {{ f.seen_count }} 次 · 收藏于 {{ dateOf(f.created_at) }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.favorites { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.list { flex: 1; min-height: 0; overflow: auto; }
.item { padding: 12px 16px; border-bottom: 1px solid var(--border); transition: background-color 120ms; }
.item:hover { background: var(--bg-3); }
.head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.text { font-family: var(--font-en); font-size: var(--fs-lg); font-weight: 600; }
.icon-btn.danger:hover { color: var(--danger); }
.ipa { font-family: var(--font-en); font-size: var(--fs-sm); color: var(--text-2); margin-top: 1px; }
.meaning { font-size: var(--fs-sm); color: var(--text-2); margin-top: 2px; }
.meta { font-size: var(--fs-xs); color: var(--text-3); margin-top: 4px; }
</style>
