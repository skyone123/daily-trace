pub mod aggregator;
pub mod capture;
pub mod commands;
pub mod llm;
pub mod report;
pub mod state;
pub mod stats;
pub mod store;
pub mod todo;

use capture::{Collector, WindowsSource};
use state::AppState;
use store::Store;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");
            std::fs::create_dir_all(&app_data).ok();

            let db_path = app_data.join("daily-trace.db");
            let store = Arc::new(Store::new(&db_path).expect("无法初始化数据库"));
            let paused = store
                .get_setting("paused")
                .map(|s| s == "true")
                .unwrap_or(false);

            let collector = Arc::new(Collector::new(
                store.clone(),
                Box::new(WindowsSource),
                paused,
            ));
            let collector_loop = collector.clone();
            tauri::async_runtime::spawn(async move {
                collector_loop.run().await;
            });

            let state = AppState::new(store, collector);

            // 系统托盘：最小化到右下角通知区
            let show_item = MenuItem::with_id(app, "show", "显示/隐藏窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
                    tauri::image::Image::new_owned(vec![22, 163, 74, 255], 1, 1)
                }))
                .menu(&menu)
                .tooltip("Daily Trace · 记录中")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // 关闭窗口按钮：隐藏到托盘而非退出进程
            if let Some(main_window) = app.get_webview_window("main") {
                let w = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::now_ms,
            commands::list_timeline,
            commands::list_segments,
            commands::aggregate_range,
            commands::generate_report,
            commands::list_reports,
            commands::get_settings,
            commands::save_setting,
            commands::set_paused,
            commands::list_categories,
            commands::stats_by_app,
            commands::seed_demo_data,
            commands::list_todos,
            commands::generate_todos,
            commands::update_todo,
            commands::evaluate_todos,
            commands::stats_heatmap,
            commands::stats_focus,
            commands::stats_wordcloud,
            commands::delete_report,
            commands::clear_reports,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
