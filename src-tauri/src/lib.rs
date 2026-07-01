mod codex_scanner;
mod commands;
mod db;
mod models;
mod rate_limit_parser;
mod token_parser;
mod watcher;

use codex_scanner::CodexScanner;
use commands::{
    hide_main_window, record_scan_completed, record_scan_started, show_main_window, AppState,
    RuntimeDiagnostics,
};
use db::AppDb;
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, RunEvent, WindowEvent, Wry};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_window_state::StateFlags;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(Target::new(TargetKind::LogDir {
                    file_name: Some("codex-usage-widget".to_string()),
                }))
                .build(),
        )
        .setup(|app| {
            let db = AppDb::open(&AppDb::default_path())?;
            let scanner = Arc::new(Mutex::new(CodexScanner::new(db)));
            let cache = Arc::new(Mutex::new(None));
            let diagnostics = Arc::new(Mutex::new(RuntimeDiagnostics::default()));
            let mut needs_history_scan = true;

            if let Ok(scanner_guard) = scanner.try_lock() {
                record_scan_started(&diagnostics);
                let scan_result = scanner_guard.scan_history();
                match &scan_result {
                    Ok(report) => {
                        needs_history_scan = false;
                        record_scan_completed(&diagnostics, report, None);
                    }
                    Err(error) => {
                        record_scan_completed(
                            &diagnostics,
                            &Default::default(),
                            Some(error.clone()),
                        );
                    }
                }

                if let Ok(state) = scanner_guard.dashboard_state() {
                    if let Ok(mut cached) = cache.lock() {
                        *cached = Some(state);
                    }
                }
            }

            app.manage(AppState {
                scanner: Arc::clone(&scanner),
                cache: Arc::clone(&cache),
                diagnostics: Arc::clone(&diagnostics),
            });

            create_tray(app.handle())?;
            watcher::start_polling(
                app.handle().clone(),
                scanner,
                cache,
                diagnostics,
                needs_history_scan,
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard_state,
            commands::refresh_now,
            commands::set_always_on_top,
            commands::get_settings,
            commands::update_settings,
            commands::get_diagnostics,
            commands::open_logs_folder,
            commands::get_autostart_enabled,
            commands::set_autostart_enabled
        ])
        .build(tauri::generate_context!())
        .expect("error while building Codex Usage widget")
        .run(|app, event| {
            if let RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                if label == "main" {
                    api.prevent_close();
                    hide_main_window(app);
                }
            }
        });
}

fn create_tray(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let diagnostics = MenuItem::with_id(app, "diagnostics", "Diagnostics", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &refresh, &diagnostics, &exit])?;

    TrayIconBuilder::new()
        .tooltip("Codex Usage")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "hide" => hide_main_window(app),
            "refresh" => {
                let state = app.state::<AppState>();
                let scanner = Arc::clone(&state.scanner);
                let cache = Arc::clone(&state.cache);
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let next_state = {
                        let Ok(scanner) = scanner.lock() else {
                            return;
                        };
                        let _ = scanner.scan_recent();
                        scanner.dashboard_state().ok()
                    };

                    if let Some(next_state) = next_state {
                        if let Ok(mut cached) = cache.lock() {
                            *cached = Some(next_state.clone());
                        }
                        let _ = app_handle.emit("dashboard-state-updated", next_state);
                    }
                });
                show_main_window(app);
            }
            "diagnostics" => {
                let _ = app.emit("show-diagnostics", ());
                show_main_window(app);
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
