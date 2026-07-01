use crate::codex_scanner::CodexScanner;
use crate::models::{DashboardState, DiagnosticsState, Settings};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    pub scanner: Arc<Mutex<CodexScanner>>,
    pub cache: Arc<Mutex<Option<DashboardState>>>,
    pub diagnostics: Arc<Mutex<RuntimeDiagnostics>>,
}

#[derive(Clone, Debug)]
pub struct RuntimeDiagnostics {
    pub last_scan_started_at: Option<String>,
    pub last_scan_completed_at: Option<String>,
    pub files_scanned: usize,
    pub token_events_accepted: usize,
    pub limit_snapshots_accepted: usize,
    pub malformed_lines: usize,
    pub io_failures: usize,
    pub last_scan_result: String,
    pub last_error: Option<String>,
    pub watcher_status: String,
}

impl Default for RuntimeDiagnostics {
    fn default() -> Self {
        Self {
            last_scan_started_at: None,
            last_scan_completed_at: None,
            files_scanned: 0,
            token_events_accepted: 0,
            limit_snapshots_accepted: 0,
            malformed_lines: 0,
            io_failures: 0,
            last_scan_result: "not run".to_string(),
            last_error: None,
            watcher_status: "starting".to_string(),
        }
    }
}

#[tauri::command]
pub fn get_dashboard_state(state: State<'_, AppState>) -> Result<DashboardState, String> {
    if let Some(cached) = state
        .cache
        .lock()
        .map_err(|error| error.to_string())?
        .clone()
    {
        return Ok(cached);
    }

    let scanner = state
        .scanner
        .try_lock()
        .map_err(|_| "Usage scan is warming up".to_string())?;
    let dashboard = scanner.dashboard_state()?;
    *state.cache.lock().map_err(|error| error.to_string())? = Some(dashboard.clone());
    Ok(dashboard)
}

#[tauri::command]
pub async fn refresh_now(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DashboardState, String> {
    let scanner = Arc::clone(&state.scanner);
    let cache = Arc::clone(&state.cache);
    let diagnostics = Arc::clone(&state.diagnostics);
    record_scan_started(&diagnostics);

    let next_state = tauri::async_runtime::spawn_blocking(move || {
        let scanner = scanner.lock().map_err(|error| error.to_string())?;
        let report = scanner.scan_recent()?;
        record_scan_completed(&diagnostics, &report, None);
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
    let scanner = state
        .scanner
        .try_lock()
        .map_err(|_| "Usage scan is running".to_string())?;
    scanner.get_settings()
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings, String> {
    let scanner = state
        .scanner
        .try_lock()
        .map_err(|_| "Usage scan is running".to_string())?;
    scanner.update_settings(&settings)
}

#[tauri::command]
pub fn get_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DiagnosticsState, String> {
    let scanner = state
        .scanner
        .try_lock()
        .map_err(|_| "Usage scan is running".to_string())?;
    let settings = scanner.get_settings()?;
    let dashboard = scanner.dashboard_state()?;
    let runtime = state
        .diagnostics
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let sessions_path = std::path::PathBuf::from(&settings.sessions_path);
    let log_directory = app
        .path()
        .app_log_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unavailable".to_string());

    Ok(DiagnosticsState {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        sessions_path: settings.sessions_path,
        sessions_exists: sessions_path.exists(),
        sessions_readable: std::fs::read_dir(&sessions_path).is_ok(),
        database_path: crate::db::AppDb::default_path()
            .to_string_lossy()
            .to_string(),
        log_directory,
        last_scan_started_at: runtime.last_scan_started_at,
        last_scan_completed_at: runtime.last_scan_completed_at,
        last_successful_data_update: dashboard.updated_at,
        watcher_status: runtime.watcher_status,
        files_scanned: runtime.files_scanned,
        token_events_accepted: runtime.token_events_accepted,
        limit_snapshots_accepted: runtime.limit_snapshots_accepted,
        malformed_lines: runtime.malformed_lines,
        io_failures: runtime.io_failures,
        last_scan_result: runtime.last_scan_result,
        last_error: runtime
            .last_error
            .or_else(|| dashboard.warnings.first().cloned()),
    })
}

#[tauri::command]
pub fn open_logs_folder(app: AppHandle) -> Result<(), String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(log_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_autostart_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|error| error.to_string())?;
    } else {
        autostart.disable().map_err(|error| error.to_string())?;
    }
    autostart.is_enabled().map_err(|error| error.to_string())
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

pub fn record_scan_started(diagnostics: &Arc<Mutex<RuntimeDiagnostics>>) {
    if let Ok(mut diagnostics) = diagnostics.lock() {
        diagnostics.last_scan_started_at = Some(Utc::now().to_rfc3339());
        diagnostics.last_scan_result = "running".to_string();
        diagnostics.last_error = None;
    }
}

pub fn record_scan_completed(
    diagnostics: &Arc<Mutex<RuntimeDiagnostics>>,
    report: &crate::codex_scanner::ScanReport,
    error: Option<String>,
) {
    if let Ok(mut diagnostics) = diagnostics.lock() {
        diagnostics.last_scan_completed_at = Some(Utc::now().to_rfc3339());
        diagnostics.files_scanned = report.files_scanned;
        diagnostics.token_events_accepted = report.token_events_added;
        diagnostics.limit_snapshots_accepted = report.limit_snapshots_added;
        diagnostics.malformed_lines = report.malformed_lines;
        diagnostics.io_failures = report.io_failures;
        diagnostics.last_scan_result = if error.is_some() {
            "failed".to_string()
        } else if report.io_failures > 0 || report.malformed_lines > 0 {
            "completed with warnings".to_string()
        } else {
            "success".to_string()
        };
        diagnostics.last_error = error;
        diagnostics.watcher_status = "polling".to_string();
    }
}
