use crate::codex_scanner::CodexScanner;
use crate::models::DashboardState;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn start_polling(
    app: AppHandle,
    scanner: Arc<Mutex<CodexScanner>>,
    cache: Arc<Mutex<Option<DashboardState>>>,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let mut needs_history_scan = true;
        loop {
            interval.tick().await;
            let state = {
                let Ok(scanner) = scanner.try_lock() else {
                    continue;
                };
                if needs_history_scan {
                    let _ = scanner.scan_history();
                    needs_history_scan = false;
                } else {
                    let _ = scanner.scan_recent();
                }
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
