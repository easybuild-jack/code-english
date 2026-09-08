// 朗读：手动点击才读（PRD 7.3）。
// 有道 dictvoice：type=1 英音，type=2 美音。单词走词典发音（小写）；句子加 le=eng 走长句语音。
// 但有道只对词库里有的词和常见短语返回音频，任意句子会 500 "returned null audio"，
// 所以句子请求失败时退回系统语音（Web Speech API，WebView2 用 Windows 自带语音），不让用户看到失败。
// 长文本按句拆开顺序播放。

import { ref } from "vue";
import { useSettings } from "../stores/settings";

const MAX_CHARS = 200;
/** 句子播放速率，0.88x 沉稳清晰，适合跟读；单词保持原速 */
const SENTENCE_RATE = 0.88;

const playing = ref<string | null>(null);
let current: HTMLAudioElement | null = null;
let queue: string[] = [];

function isSingleWord(text: string) {
  return !/\s/.test(text.trim());
}

function url(text: string, accent: "us" | "uk") {
  const type = accent === "uk" ? 1 : 2;
  if (isSingleWord(text)) {
    return `https://dict.youdao.com/dictvoice?audio=${encodeURIComponent(text.trim().toLowerCase())}&type=${type}`;
  }
  return `https://dict.youdao.com/dictvoice?audio=${encodeURIComponent(text.trim())}&type=${type}&le=eng`;
}

function chunk(text: string): string[] {
  const parts = text.match(/[^.!?]+[.!?]?/g)?.map((s) => s.trim()).filter(Boolean) ?? [text];
  const out: string[] = [];
  for (const p of parts) {
    if (p.length <= MAX_CHARS) {
      out.push(p);
    } else {
      // 极长句按逗号再拆
      out.push(...p.split(/,\s*/).filter(Boolean));
    }
  }
  return out;
}

/** 系统语音兜底。优先挑对应口音的 Natural / Online 音色，其次同语言任意音色 */
function speakLocal(text: string, accent: "us" | "uk", onEnd: () => void, onError: () => void): boolean {
  const synth = window.speechSynthesis;
  if (!synth) return false;
  const lang = accent === "uk" ? "en-GB" : "en-US";
  const voices = synth.getVoices();
  const sameLang = voices.filter((v) => v.lang.replace("_", "-").toLowerCase() === lang.toLowerCase());
  const voice =
    sameLang.find((v) => /natural|online/i.test(v.name)) ??
    sameLang[0] ??
    voices.find((v) => v.lang.toLowerCase().startsWith("en"));
  const u = new SpeechSynthesisUtterance(text);
  u.lang = lang;
  u.rate = SENTENCE_RATE;
  if (voice) u.voice = voice;
  u.onend = onEnd;
  u.onerror = (e) => (e.error === "interrupted" || e.error === "canceled" ? undefined : onError());
  synth.cancel();
  synth.speak(u);
  return true;
}

export function useTts() {
  const settings = useSettings();

  function stop() {
    queue = [];
    if (current) {
      current.pause();
      current.src = "";
      current = null;
    }
    window.speechSynthesis?.cancel();
    playing.value = null;
  }

  function playNext(key: string, onError: (msg: string) => void) {
    const next = queue.shift();
    if (!next) {
      playing.value = null;
      current = null;
      return;
    }
    const fail = () => {
      stop();
      onError("朗读失败");
    };
    // onerror 和 play() 的 reject 可能都触发，只兜底一次
    let handled = false;
    const fallback = () => {
      if (handled || current !== audio) return;
      handled = true;
      current = null;
      // 单词有道基本都有；句子拿不到就用系统语音
      if (isSingleWord(next) || !speakLocal(next, settings.ttsAccent, () => playNext(key, onError), fail)) fail();
    };
    const audio = new Audio(url(next, settings.ttsAccent));
    if (!isSingleWord(next)) audio.playbackRate = SENTENCE_RATE;
    current = audio;
    audio.onended = () => playNext(key, onError);
    audio.onerror = fallback;
    void audio.play().catch(fallback);
  }

  /** 同一段再点一次 = 停止；点另一段 = 停掉上一段再播 */
  function speak(text: string, onError: (msg: string) => void = () => {}) {
    const key = text.trim();
    if (!key) return;
    if (playing.value === key) {
      stop();
      return;
    }
    stop();
    playing.value = key;
    queue = chunk(key);
    playNext(key, onError);
  }

  function isPlaying(text: string) {
    return playing.value === text.trim();
  }

  return { speak, stop, isPlaying, playing };
}
