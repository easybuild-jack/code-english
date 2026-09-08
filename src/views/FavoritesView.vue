<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NInput, NTag, NEmpty, NButton, NSelect, useMessage } from "naive-ui";
import { Volume2, Square, Check, Trash2 } from "lucide-vue-next";
import { api, type Favorite, type FavoriteKind } from "../api";
import { useTts } from "../composables/useTts";
import { useTranslate } from "../stores/translate";
import PageHeader from "../components/PageHeader.vue";

const emit = defineEmits<{ back: [] }>();

const message = useMessage();
const tts = useTts();
const t = useTranslate();

const all = ref<Favorite[]>([]);
const kind = ref<FavoriteKind | "all">("all");
const mastered = ref<"pending" | "done">("pending");
const query = ref("");
const orderBySeen = ref(false);
const expanded = ref<number | null>(null);

const KINDS: { key: FavoriteKind | "all"; label: string }[] = [
  { key: "all", label: "全部" },
  { key: "word", label: "单词" },
  { key: "phrase", label: "短语" },
  { key: "sentence", label: "句子" },
];
const kindOptions = KINDS.filter((k) => k.key !== "all").map((k) => ({ label: k.label, value: k.key }));

async function load() {
  try {
    all.value = await api.listFavorites({ order_by_seen: orderBySeen.value });
  } catch (e) {
    message.error(String(e));
  }
}
onMounted(load);
watch(orderBySeen, load);

const counts = computed(() => {
  const c: Record<string, number> = { all: 0, word: 0, phrase: 0, sentence: 0 };
  for (const f of all.value) {
    if (f.mastered !== (mastered.value === "done")) continue;
    c.all!++;
    c[f.kind] = (c[f.kind] ?? 0) + 1;
  }
  return c;
});

const list = computed(() => {
  const q = query.value.trim().toLowerCase();
  return all.value.filter(
    (f) =>
      f.mastered === (mastered.value === "done") &&
      (kind.value === "all" || f.kind === kind.value) &&
      (!q || f.text.toLowerCase().includes(q) || f.meaning.toLowerCase().includes(q)),
  );
});

async function toggleMastered(f: Favorite) {
  f.mastered = !f.mastered;
  await api.updateFavorite(f);
}

async function save(f: Favorite) {
  await api.updateFavorite(f);
  message.success("已保存");
}

async function remove(f: Favorite) {
  await api.deleteFavorite(f.id);
  all.value = all.value.filter((x) => x.id !== f.id);
  await t.refreshFavoriteLookup();
}

function dateOf(ts: string) {
  const d = new Date(ts.replace(" ", "T"));
  return `${d.getMonth() + 1}月${d.getDate()}日`;
}
</script>

<template>
  <div class="favorites">
    <PageHeader title="收藏" @back="emit('back')" />

    <div class="filters">
      <div class="row">
        <NTag
          v-for="k in KINDS"
          :key="k.key"
          size="small"
          :bordered="false"
          :type="kind === k.key ? 'primary' : 'default'"
          class="chip"
          @click="kind = k.key"
        >
          {{ k.label }} {{ counts[k.key] ?? 0 }}
        </NTag>
      </div>
      <div class="row">
        <NTag size="small" :bordered="false" :type="mastered === 'pending' ? 'primary' : 'default'" class="chip" @click="mastered = 'pending'">未掌握</NTag>
        <NTag size="small" :bordered="false" :type="mastered === 'done' ? 'primary' : 'default'" class="chip" @click="mastered = 'done'">已掌握</NTag>
        <span class="spacer" />
        <NButton text size="tiny" @click="orderBySeen = !orderBySeen">
          {{ orderBySeen ? "按遇到次数" : "按收藏时间" }}
        </NButton>
      </div>
      <NInput v-model:value="query" size="small" clearable placeholder="搜索" />
    </div>

    <div class="list">
      <NEmpty v-if="!list.length" description="这里还是空的，翻译时点 ☆ 收藏" style="margin-top: 48px" />
      <div v-for="f in list" :key="f.id" class="item" @click="expanded = expanded === f.id ? null : f.id">
        <div class="head">
          <span class="text selectable">{{ f.text }}</span>
          <span class="ops">
            <button class="icon-btn" :title="tts.isPlaying(f.text) ? '停止' : '朗读'" @click.stop="tts.speak(f.text, (m) => message.error(m))">
              <Square v-if="tts.isPlaying(f.text)" :size="16" :stroke-width="1.75" />
              <Volume2 v-else :size="16" :stroke-width="1.75" />
            </button>
            <button class="icon-btn" :class="{ ok: f.mastered }" :title="f.mastered ? '取消已掌握' : '标为已掌握'" @click.stop="toggleMastered(f)">
              <Check :size="16" :stroke-width="1.75" />
            </button>
          </span>
        </div>
        <div v-if="f.meaning" class="meaning">{{ f.meaning }}</div>
        <div v-if="f.example && expanded !== f.id" class="example">{{ f.example }}</div>
        <div class="meta">遇到 {{ f.seen_count }} 次 · 收藏于 {{ dateOf(f.created_at) }}<template v-if="f.domain"> · {{ f.domain }}</template></div>

        <div v-if="expanded === f.id" class="edit" @click.stop>
          <NSelect v-model:value="f.kind" :options="kindOptions" size="small" style="width: 90px" />
          <NInput v-model:value="f.meaning" size="small" placeholder="中文含义" />
          <NInput v-model:value="f.domain" size="small" placeholder="领域，如 API Design" />
          <NInput v-model:value="f.example" type="textarea" size="small" :autosize="{ minRows: 1, maxRows: 3 }" placeholder="例句" />
          <div class="edit-ops">
            <NButton size="tiny" type="error" tertiary @click="remove(f)">
              <template #icon><Trash2 :size="14" /></template>
              删除
            </NButton>
            <NButton size="tiny" type="primary" @click="save(f)">保存</NButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.favorites { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.filters { padding: 12px 16px 8px; display: flex; flex-direction: column; gap: 8px; flex: none; }
.row { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
.chip { cursor: pointer; }
.spacer { flex: 1; }
.list { flex: 1; min-height: 0; overflow: auto; }
.item { padding: 12px 16px; border-bottom: 1px solid var(--border); cursor: pointer; transition: background-color 120ms; }
.item:hover { background: var(--bg-3); }
.head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.text { font-family: var(--font-en); font-size: var(--fs-lg); font-weight: 600; }
.ops { display: inline-flex; gap: 4px; }
.icon-btn.ok { color: var(--ok); }
.meaning { font-size: var(--fs-sm); color: var(--text-2); margin-top: 2px; }
.example { font-family: var(--font-en); font-size: var(--fs-sm); color: var(--text-2); margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.meta { font-size: var(--fs-xs); color: var(--text-3); margin-top: 4px; }
.edit { display: flex; flex-direction: column; gap: 8px; margin-top: 12px; }
.edit-ops { display: flex; justify-content: flex-end; gap: 8px; }
</style>
