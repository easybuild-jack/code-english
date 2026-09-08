// 边缘吸附：未置顶的窗口失焦后推到屏幕左/右边缘，只露一条边；
// 鼠标碰到、全局快捷键或托盘唤起时弹回原位。类似 QQ 的贴边隐藏。

import { defineStore } from "pinia";
import { ref } from "vue";
import { currentMonitor, cursorPosition, getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { useSettings } from "./settings";

/** 吸附后露在屏幕上的宽度（逻辑像素） */
export const TAB_WIDTH = 8;
/** 把手热区的半高（逻辑像素），与 App.vue 里把手的 64px 对应，上下各多留一点 */
const HANDLE_HALF = 48;
/** 吸附期间轮询鼠标位置的间隔 */
const POLL_MS = 120;
/** 鼠标要在把手上停留几次轮询才弹回，避免拖滚动条时擦过屏幕边缘误触 */
const DWELL_POLLS = 3;

export const useDock = defineStore("dock", () => {
  const settings = useSettings();
  const win = getCurrentWindow();
  const docked = ref(false);
  let restorePos: PhysicalPosition | null = null;
  let busy = false;
  let pollTimer: number | undefined;
  /** 露出来那条边在屏幕上的矩形（物理像素） */
  let tabRect: { x1: number; x2: number; y1: number; y2: number } | null = null;

  /** 失焦时调用。置顶（锁定）状态下不吸附 */
  async function dock() {
    if (docked.value || busy || settings.alwaysOnTop) return;
    if (!(await win.isVisible()) || (await win.isMinimized())) return;
    busy = true;
    try {
      const monitor = await currentMonitor();
      if (!monitor) return;
      const pos = await win.outerPosition();
      const size = await win.outerSize();
      const inner = await win.innerSize();
      // Windows 会给可拉伸窗口留一圈看不见的边框（outer 比 inner 宽），露边时要越过它
      const frame = Math.max(0, Math.round((size.width - inner.width) / 2));
      const tab = Math.round(TAB_WIDTH * monitor.scaleFactor);
      const area = monitor.workArea;
      const areaRight = area.position.x + area.size.width;
      const right = settings.dockEdge === "right";
      const x = right ? areaRight - tab - frame : area.position.x - size.width + tab + frame;
      restorePos = pos;
      const midY = pos.y + size.height / 2;
      const half = Math.round(HANDLE_HALF * monitor.scaleFactor);
      tabRect = {
        x1: right ? areaRight - tab : area.position.x,
        x2: right ? areaRight : area.position.x + tab,
        y1: midY - half,
        y2: midY + half,
      };
      // 吸附期间临时置顶，否则露出的那条边会被别的窗口盖住
      await win.setAlwaysOnTop(true);
      await win.setPosition(new PhysicalPosition(x, pos.y));
      docked.value = true;
      startPolling();
    } finally {
      busy = false;
    }
  }

  /** 弹回原位。focus=false 用于窗口已经拿到焦点的场合 */
  async function undock(focus = true) {
    if (!docked.value) return;
    docked.value = false;
    stopPolling();
    const pos = restorePos;
    restorePos = null;
    if (pos) await win.setPosition(pos);
    await win.setAlwaysOnTop(settings.alwaysOnTop);
    if (focus) await win.setFocus();
  }

  // 露出的 8px 正好落在无边框窗口的拉伸边框上，网页收不到 mouseenter，只能轮询鼠标位置
  function startPolling() {
    stopPolling();
    let hits = 0;
    pollTimer = window.setInterval(async () => {
      if (!docked.value || !tabRect) return;
      const c = await cursorPosition().catch(() => null);
      if (!c) return;
      const inside = c.x >= tabRect.x1 && c.x <= tabRect.x2 && c.y >= tabRect.y1 && c.y <= tabRect.y2;
      hits = inside ? hits + 1 : 0;
      if (hits >= DWELL_POLLS) void undock();
    }, POLL_MS);
  }

  function stopPolling() {
    window.clearInterval(pollTimer);
    pollTimer = undefined;
  }

  return { docked, dock, undock };
});
