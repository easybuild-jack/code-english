mod ask;
mod commands;
mod db;
mod error;
mod llm;
mod lookup;
mod secrets;
mod translate;
mod tray;
mod window_style;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let db = db::Db::open(&data_dir)?;
            app.manage(db);

            tray::setup(app.handle())?;

            if let Some(win) = app.get_webview_window("main") {
                window_style::apply(&win);
            }

            // 非开机自启时直接显示窗口；开机自启时留在托盘（PRD 7.6「启动后」默认值）
            let autostarted = std::env::args().any(|a| a == "--autostart");
            if !autostarted {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关闭按钮 = 隐藏到托盘，不退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::translate,
            commands::ask,
            commands::lookup_word,
            commands::get_profile,
            commands::save_profile,
            commands::list_providers,
            commands::save_provider,
            commands::test_provider,
            commands::list_history,
            commands::delete_history,
            commands::clear_history,
            commands::add_favorite,
            commands::list_favorites,
            commands::update_favorite,
            commands::delete_favorite,
            commands::favorite_lookup,
            commands::data_dir,
            commands::export_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
