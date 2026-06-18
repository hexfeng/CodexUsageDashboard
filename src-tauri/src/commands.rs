use crate::codex_scanner::CodexScanner;
use crate::models::{DashboardState, Settings};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

pub struct AppState {
    pub scanner: Arc<Mutex<CodexScanner>>,
    pub cache: Arc<Mutex<Option<DashboardState>>>,
}

#[tauri::command]
pub fn get_dashboard_state(state: State<'_, AppState>) -> Result<DashboardState, String> {
    if let Some(cached) = state.cache.lock().map_err(|error| error.to_string())?.clone() {
        return Ok(cached);
    }

    let scanner = state.scanner.try_lock().map_err(|_| "Usage scan is warming up".to_string())?;
    let dashboard = scanner.dashboard_state()?;
    *state.cache.lock().map_err(|error| error.to_string())? = Some(dashboard.clone());
    Ok(dashboard)
}

#[tauri::command]
pub async fn refresh_now(app: AppHandle, state: State<'_, AppState>) -> Result<DashboardState, String> {
    let scanner = Arc::clone(&state.scanner);
    let cache = Arc::clone(&state.cache);

    let next_state = tauri::async_runtime::spawn_blocking(move || {
        let scanner = scanner.lock().map_err(|error| error.to_string())?;
        scanner.scan_recent()?;
        scanner.dashboard_state()
    })
    .await
    .map_err(|error| error.to_string())??;

    *cache.lock().map_err(|error| error.to_string())? = Some(next_state.clone());
    let _ = app.emit("dashboard-state-updated", next_state.clone());

    Ok(next_state)
}

#[tauri::command]
pub fn set_always_on_top(window: WebviewWindow, enabled: bool) -> Result<(), String> {
    window
        .set_always_on_top(enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let scanner = state.scanner.try_lock().map_err(|_| "Usage scan is running".to_string())?;
    scanner.get_settings()
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let scanner = state.scanner.try_lock().map_err(|_| "Usage scan is running".to_string())?;
    scanner.update_settings(&settings)
}

pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
