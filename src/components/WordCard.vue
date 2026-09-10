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
import { api, guessKind } from "../api";
import { useSettings } from "../stores/settings";
import { useTranslate } from "../stores/translate";
import { useTts } from "../composables/useTts";

const props = defineProps<{
  text: string;
  /** 所在句子，只作为查词上下文，不写入收藏 */
  context: string;
  /** 生成当前英文句子的中文原文 */
  source: string;
  x: number;
  y: number;
}>();
const emit = defineEmits<{ close: [] }>();

const t = useTranslate();
const settings = useSettings();
const tts = useTts();
const message = useMessage();

const root = ref<HTMLElement | null>(null);
const result = ref<LookupResult | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

async function load() {
  const key = `v3\n${settings.ttsAccent}\n${props.text.toLowerCase()}\n${props.context}\n${props.source}`;
  const hit = cache.get(key);
  if (hit) {
    result.value = hit;
    return;
  }
  loading.value = true;
  error.value = null;
  result.value = null;
  try {
    const r = await api.lookup(props.text, props.context, props.source, t.scene, settings.ttsAccent);
    cache.set(key, r);
    result.value = r;
  } catch (e) {
    error.value = typeof e === "string" ? e : String(e);
  } finally {
    loading.value = false;
  }
}

watch(() => [props.text, props.context, settings.ttsAccent], load, { immediate: true });

async function favorite() {
  try {
    const meanings = result.value
      ? [result.value.meaning, ...result.value.other_meanings].join("；")
      : "";
    const r = await api.addFavorite({
      kind: guessKind(props.text),
      text: props.text,
      meaning: meanings,
      ipa: result.value?.ipa ?? "",
      accent: settings.ttsAccent,
    });
    message.success(r.created ? "已收藏" : "已收藏过，次数 +1");
    if (settings.syncBaseUrl) {
      void api.syncFavorites(settings.syncBaseUrl).catch(() => {});
    }
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
      <span class="word selectable">
        {{ text }}<span v-if="result?.pos" class="pos">{{ result.pos }}</span>
        <span v-if="result?.ipa" class="ipa">
          {{ settings.ttsAccent === "us" ? "美" : "英" }} /{{ result.ipa.replace(/^\/|\/$/g, "") }}/
        </span>
      </span>
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
    <template v-else-if="result">
      <div class="context-meaning">
        <span class="context-label">当前语境</span>
        <strong>{{ result.meaning }}</strong>
      </div>
      <div v-if="result.other_meanings.length" class="other-meanings">
        <span>其他常见义</span>
        {{ result.other_meanings.join("、") }}
      </div>
    </template>
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
.ipa { display: block; color: var(--text-2); font-size: var(--fs-sm); font-weight: 400; line-height: 18px; }
.meaning { margin-top: 6px; font-size: var(--fs-sm); color: var(--text); line-height: 1.5; }
.meaning.dim { color: var(--text-3); }
.meaning.err { color: var(--danger); }
.context-meaning { display: flex; align-items: baseline; gap: 8px; margin-top: 6px; }
.context-label {
  flex: none;
  padding: 1px 5px;
  border-radius: var(--r-hl);
  background: var(--accent-bg);
  color: var(--accent);
  font-size: var(--fs-xs);
}
.context-meaning strong { font-size: var(--fs-md); }
.other-meanings { margin-top: 6px; color: var(--text-2); font-size: var(--fs-sm); line-height: 1.5; }
.other-meanings span { margin-right: 8px; color: var(--text-3); font-size: var(--fs-xs); }
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
