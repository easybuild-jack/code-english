// 全局快捷键（唤起窗口）走 tauri 插件；窗口内快捷键走 keydown 监听。
// 快捷键字符串统一用 Tauri 的写法，如 "CmdOrCtrl+Shift+E"。

import { onMounted, onUnmounted, watch } from "vue";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "../stores/settings";
import { useDock } from "../stores/dock";

export interface HotkeyActions {
  translate: () => void;
  speak: () => void;
  nextScene: () => void;
  toggleTheme: () => void;
  hide: () => void;
}

const isMac = navigator.platform.toLowerCase().includes("mac");

/** 把 "CmdOrCtrl+Shift+E" 和 KeyboardEvent 做比对 */
export function matches(combo: string, e: KeyboardEvent): boolean {
  const parts = combo.split("+").map((p) => p.trim().toLowerCase());
  const key = parts.pop();
  if (!key) return false;

  const needCtrl = parts.includes("ctrl") || (parts.includes("cmdorctrl") && !isMac);
  const needMeta = parts.includes("cmd") || parts.includes("super") || (parts.includes("cmdorctrl") && isMac);
  const needShift = parts.includes("shift");
  const needAlt = parts.includes("alt") || parts.includes("option");

  if (e.ctrlKey !== needCtrl || e.metaKey !== needMeta || e.shiftKey !== needShift || e.altKey !== needAlt) {
    return false;
  }
  const pressed = e.key.toLowerCase();
  return pressed === key || (key === "enter" && pressed === "enter") || (key === "tab" && pressed === "tab");
}

export function hasModifier(combo: string): boolean {
  return combo.includes("+");
}

/** 展示用：Windows 显示 Ctrl，mac 显示 ⌘ */
export function pretty(combo: string): string {
  return combo
    .replace(/CmdOrCtrl/gi, isMac ? "⌘" : "Ctrl")
    .replace(/Cmd/gi, "⌘")
    .replace(/Shift/gi, isMac ? "⇧" : "Shift")
    .replace(/Alt/gi, isMac ? "⌥" : "Alt");
}

export function useHotkeys(actions: HotkeyActions) {
  const settings = useSettings();
  const dock = useDock();
  const win = getCurrentWindow();

  async function registerGlobal() {
    await unregisterAll().catch(() => {});
    try {
      await register(settings.hotkeys.toggleWindow, async (event) => {
        if (event.state !== "Pressed") return;
        if (dock.docked) {
          await dock.undock();
        } else if (await win.isVisible()) {
          await win.hide();
        } else {
          await win.show();
          await win.setFocus();
        }
      });
      return null;
    } catch (e) {
      return typeof e === "string" ? e : "快捷键被占用，请换一个";
    }
  }

  function onKeydown(e: KeyboardEvent) {
    // 中文输入法选词时的 Enter 不是快捷键
    if (e.isComposing || e.keyCode === 229) return;
    const h = settings.hotkeys;
    if (e.key === "Escape") {
      actions.hide();
      return;
    }
    // 不带修饰键的快捷键（如 Enter）只在输入框里生效，避免抢走按钮、下拉的回车
    const inTextarea = e.target instanceof HTMLTextAreaElement;
    const table: [string, () => void][] = [
      [h.translate, actions.translate],
      [h.speak, actions.speak],
      [h.nextScene, actions.nextScene],
      [h.toggleTheme, actions.toggleTheme],
    ];
    for (const [combo, fn] of table) {
      if (!hasModifier(combo) && !inTextarea) continue;
      if (matches(combo, e)) {
        e.preventDefault();
        fn();
        return;
      }
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", onKeydown);
    void registerGlobal();
  });
  onUnmounted(() => {
    window.removeEventListener("keydown", onKeydown);
    void unregisterAll();
  });

  watch(() => settings.hotkeys.toggleWindow, () => void registerGlobal());

  return { registerGlobal };
}
