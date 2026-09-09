<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import {
  NInput,
  NButton,
  NSwitch,
  NSlider,
  NTag,
  useMessage,
} from "naive-ui";
import { enable as enableAutostart, disable as disableAutostart } from "@tauri-apps/plugin-autostart";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { api, SCENES, type Profile, type ProviderView } from "../api";
import { useSettings, DEFAULT_HOTKEYS, type Hotkeys } from "../stores/settings";
import { useTts } from "../composables/useTts";
import { pretty } from "../composables/useHotkeys";
import PageHeader from "../components/PageHeader.vue";
import Segmented from "../components/Segmented.vue";

const THEME_OPTS = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
] as const;
const SIZE_OPTS = [
  { value: "sm", label: "小" },
  { value: "md", label: "中" },
  { value: "lg", label: "大" },
] as const;
const FONT_OPTS = [
  { value: "sans", label: "无衬线" },
  { value: "mono", label: "等宽" },
] as const;
const ACCENT_OPTS = [
  { value: "us", label: "美音" },
  { value: "uk", label: "英音" },
] as const;
const SCENE_OPTS = SCENES.map((s) => ({ value: s.value, label: s.label }));

const emit = defineEmits<{ back: [] }>();

const settings = useSettings();
const message = useMessage();
const tts = useTts();

// ---- 档案 ----
const profile = reactive<Profile>({ occupation: "", tech_stack: "", industry: "", default_scene: "work" });
let profileTimer: number | undefined;
async function loadProfile() {
  Object.assign(profile, await api.getProfile());
}
function saveProfileDebounced() {
  window.clearTimeout(profileTimer);
  profileTimer = window.setTimeout(() => void api.saveProfile({ ...profile }), 400);
}

// ---- 模型 ----
const providers = ref<ProviderView[]>([]);
const editing = ref<ProviderView | null>(null);
const apiKeyInput = ref("");
const testing = ref(false);
const testResult = ref<string | null>(null);

/** 常用模型速填。翻译只需便宜的，第一个为推荐 */
const MODEL_PRESETS: Record<string, { model: string; hint: string }[]> = {
  "deepseek.com": [
    { model: "deepseek-v4-flash", hint: "便宜，翻译够用" },
    { model: "deepseek-v4-pro", hint: "更强，约 3 倍价格" },
  ],
};
function presetsFor(baseUrl: string) {
  const host = Object.keys(MODEL_PRESETS).find((k) => baseUrl.includes(k));
  return host ? MODEL_PRESETS[host]! : [];
}

async function loadProviders() {
  providers.value = await api.listProviders();
}
function edit(p: ProviderView) {
  editing.value = { ...p };
  apiKeyInput.value = "";
  testResult.value = null;
}
async function saveProvider() {
  if (!editing.value) return;
  const { has_api_key: _omit, ...config } = editing.value;
  await api.saveProvider(config, apiKeyInput.value || undefined);
  message.success("已保存");
  editing.value = null;
  await loadProviders();
}
async function test() {
  if (!editing.value) return;
  // 先保存再测，保证测的是当前填的 Key
  const { has_api_key: _omit, ...config } = editing.value;
  await api.saveProvider(config, apiKeyInput.value || undefined);
  testing.value = true;
  testResult.value = null;
  try {
    const ms = await api.testProvider(editing.value.id);
    testResult.value = `已连接 · ${ms}ms`;
  } catch (e) {
    testResult.value = `失败：${e}`;
  } finally {
    testing.value = false;
    await loadProviders();
  }
}

// ---- 快捷键 ----
const HOTKEY_LABELS: { key: keyof Hotkeys; label: string; scope: string }[] = [
  { key: "toggleWindow", label: "显示 / 隐藏窗口", scope: "全局" },
  { key: "translate", label: "翻译", scope: "窗口内" },
  { key: "speak", label: "朗读整句", scope: "窗口内" },
  { key: "nextScene", label: "切换场景", scope: "窗口内" },
  { key: "toggleTheme", label: "切换深浅色", scope: "窗口内" },
];
const recording = ref<keyof Hotkeys | null>(null);

function onRecord(e: KeyboardEvent) {
  if (!recording.value) return;
  e.preventDefault();
  if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CmdOrCtrl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  parts.push(key);
  settings.hotkeys[recording.value] = parts.join("+");
  recording.value = null;
}

// ---- 通用 ----
const dataDir = ref("");
async function onAutostart(v: boolean) {
  settings.autostart = v;
  try {
    if (v) await enableAutostart();
    else await disableAutostart();
  } catch (e) {
    message.error(String(e));
  }
}
async function exportData() {
  try {
    const file = await api.exportData();
    message.success(`已导出到 ${file}`);
  } catch (e) {
    message.error(String(e));
  }
}

onMounted(async () => {
  await Promise.all([loadProfile(), loadProviders()]);
  dataDir.value = await api.dataDir();
  window.addEventListener("keydown", onRecord);
});
</script>

<template>
  <div class="settings">
    <PageHeader title="设置" @back="emit('back')" />
    <div class="body">
      <!-- 7.1 我的档案 -->
      <section>
        <h3>我的档案</h3>
        <label>职业<NInput v-model:value="profile.occupation" size="small" @update:value="saveProfileDebounced" /></label>
        <label>技术栈<NInput v-model:value="profile.tech_stack" size="small" placeholder="逗号分隔" @update:value="saveProfileDebounced" /></label>
        <label>行业<NInput v-model:value="profile.industry" size="small" placeholder="可留空，换项目时改" @update:value="saveProfileDebounced" /></label>
        <label>
          默认场景
          <Segmented v-model="profile.default_scene" :options="SCENE_OPTS" @update:model-value="saveProfileDebounced" />
        </label>
        <p class="note">这些信息会拼进翻译提示词，决定用词是否贴合你的圈子。</p>
      </section>

      <!-- 7.2 模型 -->
      <section>
        <h3>模型</h3>
        <div v-for="p in providers" :key="p.id" class="provider">
          <div class="p-main">
            <div class="p-name">
              {{ p.display_name }}
              <NTag v-if="p.is_default" size="tiny" :bordered="false" type="primary">默认</NTag>
            </div>
            <div class="p-sub">
              {{ p.model }} ·
              <span :class="p.has_api_key ? 'ok' : 'warn'">{{ p.has_api_key ? "已配置 Key" : "未配置 Key" }}</span>
            </div>
          </div>
          <NButton size="tiny" tertiary @click="edit(p)">编辑</NButton>
        </div>
        <p class="note">涉及公司敏感内容请注意，一期尚未接入本地模型。</p>

        <div v-if="editing" class="editor">
          <label>名称<NInput v-model:value="editing.display_name" size="small" /></label>
          <label>Base URL<NInput v-model:value="editing.base_url" size="small" /></label>
          <label>模型名<NInput v-model:value="editing.model" size="small" /></label>
          <div v-if="presetsFor(editing.base_url).length" class="presets">
            <NTag
              v-for="m in presetsFor(editing.base_url)"
              :key="m.model"
              size="small"
              :bordered="false"
              :type="editing.model === m.model ? 'primary' : 'default'"
              class="preset"
              :title="m.hint"
              @click="editing.model = m.model"
            >
              {{ m.model }} <span class="preset-hint">{{ m.hint }}</span>
            </NTag>
          </div>
          <label>
            API Key
            <NInput
              v-model:value="apiKeyInput"
              type="password"
              show-password-on="click"
              size="small"
              :placeholder="editing.has_api_key ? '已保存，留空则不修改' : '粘贴 API Key'"
            />
          </label>
          <div class="editor-ops">
            <NButton size="tiny" :loading="testing" @click="test">测试连接</NButton>
            <span class="test-result" :class="{ ok: testResult?.startsWith('已连接'), warn: testResult?.startsWith('失败') }">{{ testResult }}</span>
            <span class="spacer" />
            <NButton size="tiny" @click="editing = null">取消</NButton>
            <NButton size="tiny" type="primary" @click="saveProvider">保存</NButton>
          </div>
        </div>
      </section>

      <!-- 7.3 朗读 -->
      <section>
        <h3>朗读</h3>
        <label>
          口音
          <Segmented v-model="settings.ttsAccent" :options="ACCENT_OPTS" />
        </label>
        <NButton size="tiny" tertiary @click="tts.speak('This method must be idempotent.', (m) => message.error(m))">试听</NButton>
        <p class="note">使用在线词典发音接口，需联网。</p>
      </section>

      <!-- 7.4 快捷键 -->
      <section>
        <h3>快捷键</h3>
        <div v-for="h in HOTKEY_LABELS" :key="h.key" class="hotkey">
          <span class="hk-label">{{ h.label }} <span class="hk-scope">{{ h.scope }}</span></span>
          <button class="hk-combo" :class="{ recording: recording === h.key }" @click="recording = recording === h.key ? null : h.key">
            {{ recording === h.key ? "按下组合键…" : pretty(settings.hotkeys[h.key]) }}
          </button>
        </div>
        <div class="hotkey">
          <span class="hk-label">隐藏窗口 <span class="hk-scope">窗口内</span></span>
          <span class="hk-combo fixed">Esc</span>
        </div>
        <NButton size="tiny" tertiary @click="settings.hotkeys = { ...DEFAULT_HOTKEYS }">恢复默认</NButton>
      </section>

      <!-- 7.5 外观 -->
      <section>
        <h3>外观</h3>
        <label>
          主题
          <Segmented v-model="settings.theme" :options="THEME_OPTS" />
        </label>
        <label>
          字号
          <Segmented v-model="settings.fontSize" :options="SIZE_OPTS" />
        </label>
        <label>
          翻译字体
          <Segmented v-model="settings.enFont" :options="FONT_OPTS" />
        </label>
        <label>
          失焦时透明度 {{ settings.inactiveOpacity }}%
          <NSlider v-model:value="settings.inactiveOpacity" :min="60" :max="100" :step="5" />
        </label>
      </section>

      <!-- 7.6 通用 -->
      <section>
        <h3>通用</h3>
        <div class="row">
          <span>开机自启</span>
          <NSwitch :value="settings.autostart" size="small" @update:value="onAutostart" />
        </div>
        <div class="row">
          <span class="path" :title="dataDir">数据位置：{{ dataDir }}</span>
          <NButton size="tiny" tertiary @click="revealItemInDir(dataDir)">打开</NButton>
        </div>
        <NButton size="tiny" tertiary @click="exportData">导出数据（JSON）</NButton>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.body { flex: 1; min-height: 0; overflow: auto; padding: 8px 16px 16px; }
section { padding: 12px 0 16px; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 10px; align-items: flex-start; }
section:last-child { border-bottom: 0; }
h3 { margin: 0 0 2px; font-size: var(--fs-md); font-weight: 600; }
label { display: flex; flex-direction: column; align-items: flex-start; gap: 4px; width: 100%; font-size: var(--fs-sm); color: var(--text-2); }
label :deep(.n-input), label :deep(.n-slider) { width: 100%; }
.note { margin: 0; font-size: var(--fs-xs); color: var(--text-3); }
.row { display: flex; align-items: center; justify-content: space-between; width: 100%; gap: 8px; font-size: var(--fs-sm); }
.path { color: var(--text-2); font-size: var(--fs-xs); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.spacer { flex: 1; }

.provider { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 8px 12px; background: var(--bg-2); border-radius: var(--r-card); }
.p-name { display: flex; align-items: center; gap: 6px; font-weight: 600; }
.p-sub { font-size: var(--fs-xs); color: var(--text-3); margin-top: 2px; }
.ok { color: var(--ok); }
.warn { color: var(--danger); }
.editor { width: 100%; display: flex; flex-direction: column; gap: 8px; padding: 12px; background: var(--bg-2); border-radius: var(--r-card); }
.editor-ops { display: flex; align-items: center; gap: 8px; }
.presets { display: flex; flex-wrap: wrap; gap: 6px; margin-top: -2px; }
.preset { cursor: pointer; font-family: var(--font-en-mono); }
.preset-hint { margin-left: 4px; font-family: var(--font-ui); color: var(--text-3); }
.test-result { font-size: var(--fs-xs); }

.hotkey { display: flex; align-items: center; justify-content: space-between; width: 100%; font-size: var(--fs-sm); }
.hk-scope { font-size: var(--fs-xs); color: var(--text-3); margin-left: 4px; }
.hk-combo {
  font: inherit;
  font-size: var(--fs-xs);
  font-family: var(--font-en-mono);
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: var(--r-ctl);
  background: var(--bg-2);
  color: var(--text);
  cursor: pointer;
  min-width: 110px;
  text-align: center;
}
.hk-combo.recording { border-color: var(--accent); color: var(--accent); }
.hk-combo.fixed { cursor: default; color: var(--text-3); }
</style>
