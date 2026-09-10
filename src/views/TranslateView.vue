<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NButton, NInput, useMessage } from "naive-ui";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SCENES } from "../api";
import { useTranslate } from "../stores/translate";
import { useSettings } from "../stores/settings";
import { useHotkeys, matches, pretty } from "../composables/useHotkeys";
import ResultArea from "../components/ResultArea.vue";

const emit = defineEmits<{ openSettings: [] }>();

const t = useTranslate();
const settings = useSettings();
const message = useMessage();
const win = getCurrentWindow();
const resultRef = ref<InstanceType<typeof ResultArea> | null>(null);
const inputRef = ref<InstanceType<typeof NInput> | null>(null);

const MAX = 500;

useHotkeys({
  translate: () => void t.run(),
  speak: () => resultRef.value?.speakAll(),
  nextScene: () => {
    const i = SCENES.findIndex((s) => s.value === t.scene);
    t.scene = SCENES[(i + 1) % SCENES.length]!.value;
  },
  toggleTheme: () => settings.toggleTheme(),
  hide: () => void win.hide(),
});

const placeholder = computed(() =>
  t.mode === "ask"
    ? `问一个英语学习的问题，${pretty(settings.hotkeys.translate)} 发送`
    : `输入中文或英文，${pretty(settings.hotkeys.translate)} 翻译`,
);
const MAX_LEN = computed(() => (t.mode === "ask" ? 1000 : MAX));

async function copyInput() {
  if (!t.source) return;
  try {
    await navigator.clipboard.writeText(t.source);
    message.success("已复制");
  } catch {
    message.error("复制失败");
  }
}

function clearInput() {
  t.source = "";
  inputRef.value?.focus();
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.isComposing || e.key !== "Enter" || !(e.ctrlKey || e.metaKey)) return;
  if (matches(settings.hotkeys.translate, e)) return;
  // 浏览器对 textarea 里的 Ctrl+Enter 默认什么都不做，手动插入换行
  const el = e.target as HTMLTextAreaElement;
  e.preventDefault();
  el.setRangeText("\n", el.selectionStart, el.selectionEnd, "end");
  el.dispatchEvent(new Event("input", { bubbles: true }));
}

onMounted(async () => {
  await t.refreshFavoriteLookup().catch(() => {});
  inputRef.value?.focus();
  // 窗口每次显示都把光标放回输入框（PRD 4.1「唤起」）
  await win.onFocusChanged(({ payload }) => {
    if (payload) inputRef.value?.focus();
  });
});
</script>

<template>
  <div class="translate">
    <div class="input-wrap">
      <NInput
        ref="inputRef"
        v-model:value="t.source"
        type="textarea"
        :autosize="{ minRows: 4, maxRows: 7 }"
        :maxlength="MAX_LEN"
        :disabled="t.status === 'loading'"
        :placeholder="placeholder"
        class="input"
        @keydown="onInputKeydown"
      />
      <span v-if="t.status === 'loading' || t.source.length >= MAX_LEN" class="hint">
        <template v-if="t.status === 'loading'">{{ t.mode === "ask" ? "思考中…" : "翻译中…" }}</template>
        <template v-else>太长了，精简一下</template>
      </span>
      <div v-else class="input-actions">
        <NButton text type="primary" size="tiny" :disabled="!t.canTranslate" @click="t.run">发送</NButton>
        <NButton text size="tiny" :disabled="!t.source" @click="copyInput">复制</NButton>
        <NButton text size="tiny" :disabled="!t.source" @click="clearInput">清除</NButton>
      </div>
    </div>
    <ResultArea ref="resultRef" @open-settings="emit('openSettings')" />
  </div>
</template>

<style scoped>
.translate {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.input-wrap {
  position: relative;
  flex: none;
  padding: 16px 16px 0;
}
.hint {
  position: absolute;
  right: 28px;
  bottom: 8px;
  font-size: var(--fs-xs);
  color: var(--text-3);
  pointer-events: none;
}
.input-actions {
  position: absolute;
  right: 28px;
  bottom: 7px;
  display: flex;
  align-items: center;
  gap: 10px;
}
</style>
