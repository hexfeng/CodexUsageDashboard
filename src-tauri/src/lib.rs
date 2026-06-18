mod codex_scanner;
mod commands;
mod db;
mod models;
mod rate_limit_parser;
mod token_parser;
mod watcher;

use codex_scanner::CodexScanner;
use commands::{hide_main_window, show_main_window, AppState};
use db::AppDb;
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, Wry};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db = AppDb::open(&AppDb::default_path())?;
            let scanner = Arc::new(Mutex::new(CodexScanner::new(db)));
            let cache = Arc::new(Mutex::new(None));

            if let Ok(scanner_guard) = scanner.try_lock() {
                if let Ok(state) = scanner_guard.dashboard_state() {
                    if let Ok(mut cached) = cache.lock() {
                        *cached = Some(state);
                    }
                }
            }

            app.manage(AppState {
                scanner: Arc::clone(&scanner),
                cache: Arc::clone(&cache),
            });

            create_tray(app.handle())?;
            watcher::start_polling(app.handle().clone(), scanner, cache);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard_state,
            commands::refresh_now,
            commands::set_always_on_top,
            commands::get_settings,
            commands::update_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Codex Usage widget");
}

fn create_tray(app: &tauri::AppHandle<Wry>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &refresh, &exit])?;

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
                        let Ok(scanner) = scanner.try_lock() else {
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
            "exit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
