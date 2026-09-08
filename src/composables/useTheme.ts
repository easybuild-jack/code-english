import { computed, onMounted, onUnmounted, ref, watchEffect } from "vue";
import { darkTheme, type GlobalTheme, type GlobalThemeOverrides } from "naive-ui";
import { resolveTheme, useSettings } from "../stores/settings";

/** 把 CSS 变量的主题同步到 <html data-*>，并给 Naive UI 生成对应主题 */
export function useTheme() {
  const settings = useSettings();
  const systemDark = ref(window.matchMedia("(prefers-color-scheme: dark)").matches);

  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  const onChange = (e: MediaQueryListEvent) => (systemDark.value = e.matches);
  onMounted(() => mq.addEventListener("change", onChange));
  onUnmounted(() => mq.removeEventListener("change", onChange));

  const resolved = computed<"light" | "dark">(() => {
    // 读一下 systemDark 让 computed 在系统切换时重新计算
    void systemDark.value;
    return resolveTheme(settings.theme);
  });

  watchEffect(() => {
    const html = document.documentElement;
    html.dataset.theme = resolved.value;
    html.dataset.size = settings.fontSize;
    html.dataset.font = settings.enFont;
  });

  const naiveTheme = computed<GlobalTheme | null>(() => (resolved.value === "dark" ? darkTheme : null));

  // Naive UI 组件颜色从同一组 token 取值（视觉规范第 7 章）
  const naiveOverrides = computed<GlobalThemeOverrides>(() => {
    const dark = resolved.value === "dark";
    return {
      common: {
        fontFamily: "var(--font-ui)",
        fontSize: "var(--fs-md)",
        borderRadius: "6px",
        primaryColor: dark ? "#6EA0FF" : "#2F6FED",
        primaryColorHover: dark ? "#86B1FF" : "#4A82F0",
        primaryColorPressed: dark ? "#5A8EEB" : "#255DD0",
        bodyColor: dark ? "#1E1F22" : "#FFFFFF",
        cardColor: dark ? "#2B2D31" : "#F6F7F9",
        inputColor: dark ? "#2B2D31" : "#F6F7F9",
        borderColor: dark ? "#3A3D42" : "#E3E6EA",
        textColorBase: dark ? "#E8E8E8" : "#1F2329",
        textColor2: dark ? "#A0A4AB" : "#5F6672",
        textColor3: dark ? "#6F7480" : "#9AA0A8",
        errorColor: dark ? "#F26D72" : "#E5484D",
        successColor: dark ? "#3DBE85" : "#22A06B",
      },
    };
  });

  return { resolved, naiveTheme, naiveOverrides };
}
