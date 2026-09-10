//! 无边框窗口的系统级外观。
//!
//! Windows 上无边框窗口仍可能带 DWM 描边，与前端 CSS 圆角叠加后在四角露线。
//! tauri.conf 已关闭原生阴影；这里继续让 DWM 裁成圆角并隐藏系统描边。
//! 仅 Windows 11 生效，Windows 10 调用会被忽略，不报错。

use tauri::WebviewWindow;

#[cfg(windows)]
pub fn apply(window: &WebviewWindow) {
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    let Ok(hwnd) = window.hwnd() else { return };
    // DWMWA_COLOR_NONE：不画系统描边，边框交给 CSS
    const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;

    unsafe {
        let corner = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const _,
            std::mem::size_of_val(&corner) as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &DWMWA_COLOR_NONE as *const _ as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg(not(windows))]
pub fn apply(_window: &WebviewWindow) {}
