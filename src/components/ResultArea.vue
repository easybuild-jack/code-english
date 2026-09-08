<script setup lang="ts">
import { computed, h, nextTick, ref, watch } from "vue";
import { NButton, NSkeleton, useMessage } from "naive-ui";
import { Volume2, Square, Star, CircleAlert } from "lucide-vue-next";
import { api, guessKind, type Keyword } from "../api";
import { useTranslate } from "../stores/translate";
import { useTts } from "../composables/useTts";
import WordCard from "./WordCard.vue";

const emit = defineEmits<{ openSettings: [] }>();

const t = useTranslate();
const tts = useTts();
const message = useMessage();
const rootRef = ref<HTMLElement | null>(null);

// Ask 模式有新消息或开始加载时滚到底
watch(
  () => [t.askMessages.length, t.status] as const,
  async () => {
    if (t.mode !== "ask") return;
    await nextTick();
    rootRef.value?.scrollTo({ top: rootRef.value.scrollHeight });
  },
);

const needsSettings = computed(
  () => t.error?.includes("API Key") || t.error?.includes("还没配置") || false,
);

type VChild = ReturnType<typeof h> | string;

/** 每个英文单词包成可点的 <span class="w">，标点和空格原样保留 */
function renderWords(text: string): VChild[] {
  return text.split(/([A-Za-z][A-Za-z0-9'’-]*)/).filter(Boolean).map((tok, i) =>
    i % 2 === 1 ? h("span", { class: "w" }, tok) : tok,
  );
}

/** 关键词段落再包一层 <span class="hl">，大小写不敏感、整词匹配 */
function renderSentence(sentence: string, keywords: Keyword[]): VChild[] {
  const words = keywords.map((k) => k.word.trim()).filter(Boolean);
  if (words.length === 0) return renderWords(sentence);
  const escaped = words.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const re = new RegExp(`(?<![A-Za-z0-9])(${escaped.join("|")})(?![A-Za-z0-9])`, "gi");
  const nodes: VChild[] = [];
  let last = 0;
  for (const m of sentence.matchAll(re)) {
    const i = m.index ?? 0;
    if (i > last) nodes.push(...renderWords(sentence.slice(last, i)));
    nodes.push(h("span", { class: "hl" }, renderWords(m[0])));
    last = i + m[0].length;
  }
  if (last < sentence.length) nodes.push(...renderWords(sentence.slice(last)));
  return nodes;
}

const Sentence = (props: { text: string; keywords: Keyword[] }) =>
  h("span", { class: "txt selectable" }, renderSentence(props.text, props.keywords));

// ---- 点词 / 划词 ----

const card = ref<{ text: string; context: string; x: number; y: number } | null>(null);
const CARD_W = 280;

function openCard(text: string, context: string, anchor: DOMRect) {
  const root = rootRef.value;
  if (!root) return;
  const base = root.getBoundingClientRect();
  const x = Math.max(8, Math.min(anchor.left - base.left + root.scrollLeft, root.clientWidth - CARD_W - 8));
  const y = anchor.bottom - base.top + root.scrollTop + 6;
  card.value = { text, context, x, y };
}

function sentOf(node: Node | null | undefined): HTMLElement | null {
  const el = node instanceof HTMLElement ? node : node?.parentElement ?? null;
  return el?.closest(".sent") ?? null;
}

const WORD_CHAR = /[A-Za-z0-9'’-]/;

/** 划选往往从词中间开始、在词中间结束，把选区两端扩到整词 */
function snapToWords(sentence: string, range: Range, txtEl: HTMLElement): string {
  const probe = document.createRange();
  probe.selectNodeContents(txtEl);
  probe.setEnd(range.startContainer, range.startOffset);
  let start = probe.toString().length;
  probe.setEnd(range.endContainer, range.endOffset);
  let end = probe.toString().length;
  // 先去掉两端选进来的空格，否则会把相邻的下一个词也带上
  while (start < end && /\s/.test(sentence[start] ?? "")) start++;
  while (end > start && /\s/.test(sentence[end - 1] ?? "")) end--;
  while (start > 0 && WORD_CHAR.test(sentence[start - 1] ?? "")) start--;
  while (end < sentence.length && WORD_CHAR.test(sentence[end] ?? "")) end++;
  return sentence.slice(start, end).trim();
}

/** 松开鼠标：有选区就查选中的短语，没有就查点到的那个词 */
function onSentencesMouseUp(e: MouseEvent) {
  const sel = window.getSelection();
  if (sel && !sel.isCollapsed && sel.rangeCount > 0) {
    const range = sel.getRangeAt(0);
    const from = sentOf(range.startContainer);
    const to = sentOf(range.endContainer);
    const txtEl = from?.querySelector<HTMLElement>(".txt");
    if (!from || from !== to || !txtEl) return;
    const sentence = from.dataset.text ?? "";
    const text = snapToWords(sentence, range, txtEl);
    // 太长、没有字母的选区不查
    if (!text || !/[A-Za-z]/.test(text) || text.split(/\s+/).length > 6) return;
    openCard(text, sentence, range.getBoundingClientRect());
    return;
  }
  const w = (e.target as HTMLElement).closest?.(".w") as HTMLElement | null;
  const sent = sentOf(w);
  if (w && sent) openCard(w.textContent ?? "", sent.dataset.text ?? "", w.getBoundingClientRect());
}

watch(() => [t.result, t.mode, t.status], () => (card.value = null));

async function favorite(text: string, meaning = "", example = "") {
  try {
    const r = await api.addFavorite({ kind: guessKind(text), text, meaning, example });
    message.success(r.created ? "已收藏" : "已收藏过，次数 +1");
    await t.refreshFavoriteLookup();
  } catch (e) {
    message.error(String(e));
  }
}

function speak(text: string) {
  tts.speak(text, (msg) => message.error(msg));
}

defineExpose({
  speakAll: () => t.mode === "translate" && t.result && speak(t.result.translation),
});
</script>

<template>
  <section ref="rootRef" class="result">
    <!-- Ask 模式：对话 -->
    <template v-if="t.mode === 'ask'">
      <p v-if="t.askMessages.length === 0 && t.status !== 'error'" class="placeholder">
        问英语学习相关的问题：某个词怎么用、这句话地道吗、两种说法有什么区别……<br />
        与英语无关的问题会被拒绝。
      </p>
      <template v-else>
        <div class="ask-head">
          <NButton text size="tiny" @click="t.clearAsk()">新对话</NButton>
        </div>
        <div v-for="(m, i) in t.askMessages" :key="i" class="msg" :class="m.role">
          <span v-if="m.role === 'user'" class="who">你</span>
          <div class="bubble selectable">{{ m.content }}</div>
        </div>
        <div v-if="t.status === 'loading'" class="msg assistant">
          <div class="skeleton" style="width: 70%">
            <NSkeleton text :repeat="1" style="width: 100%" />
            <NSkeleton text :repeat="1" style="width: 60%" />
          </div>
        </div>
        <div v-if="t.status === 'error'" class="error">
          <CircleAlert :size="16" :stroke-width="1.75" />
          <span>发送失败：{{ t.error }}</span>
          <NButton v-if="needsSettings" size="tiny" tertiary @click="emit('openSettings')">去设置</NButton>
        </div>
      </template>
    </template>

    <!-- 初始 -->
    <p v-else-if="t.status === 'idle'" class="placeholder">
      翻译结果显示在这里。对照着敲进对话框，不要复制。
    </p>

    <!-- 翻译中 -->
    <div v-else-if="t.status === 'loading'" class="skeleton">
      <NSkeleton text :repeat="1" style="width: 100%" />
      <NSkeleton text :repeat="1" style="width: 85%" />
      <NSkeleton text :repeat="1" style="width: 60%" />
    </div>

    <!-- 出错 -->
    <div v-else-if="t.status === 'error'" class="error">
      <CircleAlert :size="16" :stroke-width="1.75" />
      <span>翻译失败：{{ t.error }}</span>
      <NButton v-if="needsSettings" size="tiny" tertiary @click="emit('openSettings')">去设置</NButton>
      <NButton v-else size="tiny" tertiary @click="t.run()">重试</NButton>
    </div>

    <!-- 成功 -->
    <template v-else-if="t.result">
      <div class="sentences" @mouseup="onSentencesMouseUp">
        <div v-for="(s, i) in t.result.sentences" :key="i" class="sent" :data-text="s">
          <span v-if="t.result.sentences.length > 1" class="no">{{ i + 1 }}.</span>
          <Sentence :text="s" :keywords="t.result.keywords" />
        </div>
        <div class="actions">
          <span class="tip">点单词或划选短语看释义</span>
          <button
            class="icon-btn"
            :title="tts.isPlaying(t.result.translation) ? '停止' : '朗读整句'"
            @click="speak(t.result.translation)"
          >
            <Square v-if="tts.isPlaying(t.result.translation)" :size="18" :stroke-width="1.75" />
            <Volume2 v-else :size="18" :stroke-width="1.75" />
          </button>
        </div>
      </div>

      <WordCard
        v-if="card"
        :text="card.text"
        :context="card.context"
        :keywords="t.result.keywords"
        :x="card.x"
        :y="card.y"
        @close="card = null"
      />

      <p v-if="t.result.raw_fallback" class="fallback">
        模型没有按约定格式返回，以上为原始内容。<NButton text size="tiny" @click="t.run()">重试</NButton>
      </p>

      <template v-if="t.result.keywords.length">
        <hr />
        <div v-for="k in t.result.keywords" :key="k.word" class="kw">
          <div class="body">
            <button class="word" :title="'朗读 ' + k.word" @click="speak(k.word)">
              <span class="hl">{{ k.word }}</span>
            </button>
            <div class="desc">{{ k.note }}</div>
            <div v-if="t.isFavorited(k.word)" class="meta">已收藏 · 第 {{ t.seenCount(k.word) }} 次遇到</div>
          </div>
          <button
            class="icon-btn"
            :class="{ active: t.isFavorited(k.word) }"
            title="收藏"
            @click="favorite(k.word, k.note, t.result.translation)"
          >
            <Star :size="16" :stroke-width="1.75" :fill="t.isFavorited(k.word) ? 'currentColor' : 'none'" />
          </button>
        </div>
      </template>

    </template>
  </section>
</template>

<style scoped>
.result {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 16px;
}
.placeholder { margin: 0; color: var(--text-3); font-size: var(--fs-sm); }
.skeleton { display: flex; flex-direction: column; gap: 10px; }
.error {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--danger);
  font-size: var(--fs-sm);
}
.fallback { margin: 8px 0 0; color: var(--text-3); font-size: var(--fs-xs); }

.sent {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-family: var(--font-en);
}
.no { width: 20px; flex: none; text-align: right; font-size: var(--fs-xs); color: var(--text-3); }
.sent :deep(.txt) {
  flex: 1;
  font-size: var(--fs-xl);
  line-height: 1.65;
  font-weight: 500;
  letter-spacing: 0.01em;
}
.sent :deep(.w) {
  cursor: pointer;
  border-radius: var(--r-hl);
  transition: background-color 120ms;
}
.sent :deep(.w:hover) { background: var(--bg-3); }
.sent :deep(.hl .w:hover) { background: transparent; text-decoration: underline; text-underline-offset: 3px; }
.actions { display: flex; align-items: center; justify-content: flex-end; gap: 4px; margin-top: 4px; }
.tip { flex: 1; font-size: var(--fs-xs); color: var(--text-3); }

hr { border: 0; border-top: 1px solid var(--border); margin: 16px 0; }

.kw { display: flex; align-items: flex-start; gap: 8px; }
.kw + .kw { margin-top: 12px; }
.kw .body { flex: 1; min-width: 0; }
.kw .word {
  border: 0;
  background: transparent;
  padding: 0;
  font-family: var(--font-en);
  font-size: var(--fs-lg);
  font-weight: 600;
  color: var(--text);
  cursor: pointer;
  text-align: left;
}
.kw .word .hl { padding: 0 4px; }
.kw .desc { font-size: var(--fs-sm); color: var(--text-2); margin-top: 2px; }
.kw .meta { font-size: var(--fs-xs); color: var(--text-3); margin-top: 2px; }
.kw .icon-btn { flex: none; margin-top: 3px; }

.ask-head { display: flex; justify-content: flex-end; margin: -8px 0 4px; }
.msg { display: flex; gap: 8px; align-items: flex-start; }
.msg + .msg { margin-top: 12px; }
.msg .who {
  flex: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--accent-bg);
  color: var(--accent);
  font-size: var(--fs-xs);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
}
.msg .bubble {
  min-width: 0;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.65;
}
.msg.user .bubble { color: var(--text-2); font-size: var(--fs-sm); }
.msg.assistant .bubble { font-size: var(--fs-md); }
.msg.assistant { padding-left: 28px; }
</style>
