use crate::codex_scanner::CodexScanner;
use crate::commands::{record_scan_completed, record_scan_started, RuntimeDiagnostics};
use crate::models::DashboardState;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn start_polling(
    app: AppHandle,
    scanner: Arc<Mutex<CodexScanner>>,
    cache: Arc<Mutex<Option<DashboardState>>>,
    diagnostics: Arc<Mutex<RuntimeDiagnostics>>,
    needs_history_scan: bool,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut needs_history_scan = needs_history_scan;
        loop {
            interval.tick().await;
            let state = {
                let Ok(scanner) = scanner.try_lock() else {
                    continue;
                };
                record_scan_started(&diagnostics);
                let report = if needs_history_scan {
                    scanner.scan_history()
                } else {
                    scanner.scan_recent()
                };
                needs_history_scan = false;
                match report {
                    Ok(report) => record_scan_completed(&diagnostics, &report, None),
                    Err(error) => {
                        record_scan_completed(&diagnostics, &Default::default(), Some(error))
                    }
                };
                scanner.dashboard_state().ok()
            };

            if let Some(state) = state {
                if let Ok(mut cached) = cache.lock() {
                    *cached = Some(state.clone());
                }
                let _ = app.emit("dashboard-state-updated", state);
            }
        }
    });
}
