<script setup lang="ts">
import { computed, h, nextTick, ref, watch } from "vue";
import { NButton, NSkeleton, useMessage } from "naive-ui";
import { Volume2, Square, CircleAlert, Sparkles } from "lucide-vue-next";
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
  return text
    .split(/([A-Za-z][A-Za-z0-9'’-]*)/)
    .filter(Boolean)
    .map((token) =>
      /^[A-Za-z][A-Za-z0-9'’-]*$/.test(token)
        ? h("span", { class: "w" }, token)
        : token,
    );
}

const Sentence = (props: { text: string }) =>
  h("span", { class: "txt selectable" }, renderWords(props.text));

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
    <div v-else-if="t.status === 'loading'" class="thinking">
      <div class="thinking-head">
        <span class="thinking-icon"><Sparkles :size="16" :stroke-width="1.75" /></span>
        <span>正在理解语境</span>
        <span class="thinking-dots"><i /><i /><i /></span>
      </div>
      <div class="skeleton">
        <NSkeleton text :repeat="1" style="width: 100%" />
        <NSkeleton text :repeat="1" style="width: 85%" />
        <NSkeleton text :repeat="1" style="width: 60%" />
      </div>
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
          <Sentence :text="s" />
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
        :source="t.source"
        :x="card.x"
        :y="card.y"
        @close="card = null"
      />

      <p v-if="t.result.raw_fallback" class="fallback">
        模型没有按约定格式返回，以上为原始内容。<NButton text size="tiny" @click="t.run()">重试</NButton>
      </p>

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
.thinking { padding-top: 2px; }
.thinking-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 14px;
  color: var(--text-2);
  font-size: var(--fs-sm);
}
.thinking-icon {
  display: inline-flex;
  color: var(--accent);
  animation: thinking-pulse 1.4s ease-in-out infinite;
}
.thinking-dots { display: inline-flex; align-items: center; gap: 3px; height: 16px; }
.thinking-dots i {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--accent);
  animation: thinking-dot 1.2s ease-in-out infinite;
}
.thinking-dots i:nth-child(2) { animation-delay: 160ms; }
.thinking-dots i:nth-child(3) { animation-delay: 320ms; }
.skeleton { display: flex; flex-direction: column; gap: 10px; opacity: 0.72; }
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
.actions { display: flex; align-items: center; justify-content: flex-end; gap: 4px; margin-top: 4px; }
.tip { flex: 1; font-size: var(--fs-xs); color: var(--text-3); }

@keyframes thinking-pulse {
  0%, 100% { transform: scale(0.9); opacity: 0.55; }
  50% { transform: scale(1.08); opacity: 1; }
}
@keyframes thinking-dot {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.35; }
  30% { transform: translateY(-3px); opacity: 1; }
}
@media (prefers-reduced-motion: reduce) {
  .thinking-icon,
  .thinking-dots i { animation: none; }
}

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
