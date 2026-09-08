// 朗读：在线词典发音接口，手动点击才读（PRD 7.3）。
// 有道 dictvoice：type=1 英音，type=2 美音。长文本按句拆开顺序播放。

import { ref } from "vue";
import { useSettings } from "../stores/settings";

const MAX_CHARS = 200;

const playing = ref<string | null>(null);
let current: HTMLAudioElement | null = null;
let queue: string[] = [];

function url(text: string, accent: "us" | "uk") {
  const type = accent === "uk" ? 1 : 2;
  return `https://dict.youdao.com/dictvoice?audio=${encodeURIComponent(text)}&type=${type}`;
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

export function useTts() {
  const settings = useSettings();

  function stop() {
    queue = [];
    if (current) {
      current.pause();
      current.src = "";
      current = null;
    }
    playing.value = null;
  }

  function playNext(key: string, onError: (msg: string) => void) {
    const next = queue.shift();
    if (!next) {
      playing.value = null;
      current = null;
      return;
    }
    const audio = new Audio(url(next, settings.ttsAccent));
    current = audio;
    audio.onended = () => playNext(key, onError);
    audio.onerror = () => {
      stop();
      onError("朗读失败");
    };
    void audio.play().catch(() => {
      stop();
      onError("朗读失败");
    });
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
