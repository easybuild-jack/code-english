<script lang="ts">
import type { LookupResult } from "../api";

/** 同一个词在同一句里只问模型一次；模块级，卡片关掉再开也命中 */
const cache = new Map<string, LookupResult>();
</script>

<script setup lang="ts">
// 点词 / 划词后弹出的小卡片：释义、朗读、收藏。定位由父组件算好传进来（相对结果区）。
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import { Volume2, Square, Star, X } from "lucide-vue-next";
import { api, guessKind, type Keyword } from "../api";
import { useTranslate } from "../stores/translate";
import { useTts } from "../composables/useTts";

const props = defineProps<{
  text: string;
  /** 所在句子，作为查词上下文和收藏例句 */
  context: string;
  keywords: Keyword[];
  x: number;
  y: number;
}>();
const emit = defineEmits<{ close: [] }>();

const t = useTranslate();
const tts = useTts();
const message = useMessage();

const root = ref<HTMLElement | null>(null);
const result = ref<LookupResult | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

async function load() {
  const key = `${props.text.toLowerCase()}\n${props.context}`;
  const hit = cache.get(key);
  if (hit) {
    result.value = hit;
    return;
  }
  // 模型已经在 keywords 里解释过的词直接用，不再发请求
  const kw = props.keywords.find((k) => k.word.trim().toLowerCase() === props.text.toLowerCase());
  if (kw) {
    result.value = { text: props.text, meaning: kw.note, pos: "" };
    cache.set(key, result.value);
    return;
  }
  loading.value = true;
  error.value = null;
  result.value = null;
  try {
    const r = await api.lookup(props.text, props.context, t.scene);
    cache.set(key, r);
    result.value = r;
  } catch (e) {
    error.value = typeof e === "string" ? e : String(e);
  } finally {
    loading.value = false;
  }
}

watch(() => [props.text, props.context], load, { immediate: true });

async function favorite() {
  try {
    const r = await api.addFavorite({
      kind: guessKind(props.text),
      text: props.text,
      meaning: result.value?.meaning ?? "",
      example: props.context,
    });
    message.success(r.created ? "已收藏" : "已收藏过，次数 +1");
    await t.refreshFavoriteLookup();
  } catch (e) {
    message.error(String(e));
  }
}

function speak() {
  tts.speak(props.text, (m) => message.error(m));
}

// 点卡片外面关闭；Esc 关卡片而不是隐藏窗口，所以要在捕获阶段抢先处理
function onPointerDown(e: PointerEvent) {
  if (root.value && !root.value.contains(e.target as Node)) emit("close");
}
function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  }
}
onMounted(() => {
  document.addEventListener("pointerdown", onPointerDown, true);
  window.addEventListener("keydown", onKeydown, true);
});
onUnmounted(() => {
  document.removeEventListener("pointerdown", onPointerDown, true);
  window.removeEventListener("keydown", onKeydown, true);
});
</script>

<template>
  <div ref="root" class="card" :style="{ left: x + 'px', top: y + 'px' }">
    <div class="head">
      <span class="word selectable">{{ text }}<span v-if="result?.pos" class="pos">{{ result.pos }}</span></span>
      <button class="icon-btn" :title="tts.isPlaying(text) ? '停止' : '朗读'" @click="speak">
        <Square v-if="tts.isPlaying(text)" :size="16" :stroke-width="1.75" />
        <Volume2 v-else :size="16" :stroke-width="1.75" />
      </button>
      <button class="icon-btn" :class="{ active: t.isFavorited(text) }" title="收藏" :disabled="loading" @click="favorite">
        <Star :size="16" :stroke-width="1.75" :fill="t.isFavorited(text) ? 'currentColor' : 'none'" />
      </button>
      <button class="icon-btn" title="关闭" @click="emit('close')">
        <X :size="16" :stroke-width="1.75" />
      </button>
    </div>
    <div v-if="loading" class="meaning dim">查询中…</div>
    <div v-else-if="error" class="meaning err">
      {{ error }}
      <button class="link" @click="load">重试</button>
    </div>
    <div v-else-if="result" class="meaning">{{ result.meaning }}</div>
    <div v-if="t.isFavorited(text)" class="meta">已收藏 · 第 {{ t.seenCount(text) }} 次遇到</div>
  </div>
</template>

<style scoped>
.card {
  position: absolute;
  z-index: 20;
  width: 280px;
  padding: 10px 12px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--r-card);
  box-shadow: var(--shadow);
}
.head { display: flex; align-items: flex-start; gap: 2px; }
.word { flex: 1; min-width: 0; font-family: var(--font-en); font-size: var(--fs-lg); font-weight: 600; line-height: 24px; word-break: break-word; }
.pos { margin-left: 6px; font-size: var(--fs-xs); color: var(--text-3); font-style: italic; font-weight: 400; }
.meaning { margin-top: 6px; font-size: var(--fs-sm); color: var(--text); line-height: 1.5; }
.meaning.dim { color: var(--text-3); }
.meaning.err { color: var(--danger); }
.meta { margin-top: 4px; font-size: var(--fs-xs); color: var(--text-3); }
.link {
  border: 0;
  padding: 0 0 0 6px;
  background: transparent;
  color: var(--accent);
  font-size: inherit;
  cursor: pointer;
}
</style>
