use crate::db::AppDb;
use crate::models::{DashboardState, Settings};
use crate::rate_limit_parser::parse_limit_snapshot;
use crate::token_parser::parse_token_event;
use chrono::Utc;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_FILES_PER_SCAN: usize = 12;
const MAX_FILES_PER_HISTORY_SCAN: usize = 512;

pub struct CodexScanner {
    db: AppDb,
    default_sessions_path: PathBuf,
}

#[derive(Debug, Default)]
pub struct ScanReport {
    pub files_scanned: usize,
    pub token_events_added: usize,
    pub limit_snapshots_added: usize,
}

impl CodexScanner {
    pub fn new(db: AppDb) -> Self {
        Self {
            db,
            default_sessions_path: default_sessions_path(),
        }
    }

    pub fn get_settings(&self) -> Result<Settings, String> {
        self.db
            .get_settings(self.default_sessions_path.to_string_lossy().to_string())
            .map_err(|error| error.to_string())
    }

    pub fn update_settings(&self, settings: &Settings) -> Result<Settings, String> {
        self.db
            .update_settings(settings)
            .map_err(|error| error.to_string())?;
        self.get_settings()
    }

    pub fn scan_recent(&self) -> Result<ScanReport, String> {
        let settings = self.get_settings()?;
        let sessions_path = PathBuf::from(&settings.sessions_path);
        let files = rollout_files(&sessions_path, 30, MAX_FILES_PER_SCAN)?;
        self.scan_files(files)
    }

    pub fn scan_history(&self) -> Result<ScanReport, String> {
        let settings = self.get_settings()?;
        let sessions_path = PathBuf::from(&settings.sessions_path);
        let files = rollout_files(&sessions_path, 366, MAX_FILES_PER_HISTORY_SCAN)?;
        self.scan_files(files)
    }

    fn scan_files(&self, files: Vec<PathBuf>) -> Result<ScanReport, String> {
        let mut previous_snapshot = self.db.latest_limit_snapshot().map_err(|error| error.to_string())?;
        let mut report = ScanReport {
            files_scanned: files.len(),
            ..ScanReport::default()
        };

        for path in files {
            let file = File::open(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            for (index, line) in BufReader::new(file).lines().enumerate() {
                let line = line.map_err(|error| format!("{}: {error}", path.display()))?;
                if let Some(event) = parse_token_event(&line, &path, index + 1) {
                    if self.db.insert_token_event(&event).map_err(|error| error.to_string())? {
                        report.token_events_added += 1;
                    }
                }

                if let Some(snapshot) = parse_limit_snapshot(&line, &path, previous_snapshot.as_ref()) {
                    self.db
                        .insert_limit_snapshot(&snapshot)
                        .map_err(|error| error.to_string())?;
                    previous_snapshot = Some(snapshot);
                    report.limit_snapshots_added += 1;
                }
            }
        }

        self.db
            .record_scan_completed(Utc::now())
            .map_err(|error| error.to_string())?;

        Ok(report)
    }

    pub fn dashboard_state(&self) -> Result<DashboardState, String> {
        let settings = self.get_settings()?;
        self.db
            .dashboard_state(&settings, settings.sessions_path.clone())
            .map_err(|error| error.to_string())
    }
}

fn rollout_files(path: &Path, days_back: i64, max_files: usize) -> Result<Vec<PathBuf>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(days_back as u64 * 86_400))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    let mut files = Vec::new();
    for entry in WalkDir::new(path).follow_links(false).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if !file_name.starts_with("rollout-") || !file_name.ends_with(".jsonl") {
            continue;
        }

        let modified = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if modified >= cutoff {
            files.push(entry.path().to_path_buf());
        }
    }

    files.sort_by(|a, b| {
        let a_modified = file_modified(a);
        let b_modified = file_modified(b);
        b_modified.cmp(&a_modified).then_with(|| b.cmp(a))
    });
    files.truncate(max_files);
    Ok(files)
}

fn file_modified(path: &Path) -> std::time::SystemTime {
    path.metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}

fn default_sessions_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("sessions")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::AppDb;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scans_rollout_files_and_builds_state() {
        let dir = tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        fs::create_dir_all(&sessions).unwrap();
        let five_hour_reset_at = Utc::now().timestamp() + 3_600;
        let weekly_reset_at = Utc::now().timestamp() + 7 * 86_400;
        fs::write(
            sessions.join("rollout-2026-06-17T17-00-00-test.jsonl"),
            format!(
                r#"{{"timestamp":"2026-06-17T17:10:00.000Z","type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":80,"reasoning_output_tokens":20,"total_tokens":1080}},"last_token_usage":{{"input_tokens":250,"cached_input_tokens":50,"output_tokens":30,"reasoning_output_tokens":5,"total_tokens":280}}}},"rate_limits":{{"primary":{{"used_percent":42.0,"window_minutes":300,"resets_at":{five_hour_reset_at}}},"secondary":{{"used_percent":68.0,"window_minutes":10080,"resets_at":{weekly_reset_at}}},"plan_type":"plus"}}}}}}"#
            ),
        )
        .unwrap();

        let db = AppDb::in_memory().unwrap();
        let scanner = CodexScanner::new(db);
        scanner
            .update_settings(&Settings {
                sessions_path: sessions.to_string_lossy().to_string(),
                always_on_top: true,
                stale_after_minutes: 5,
                heatmap_days: 14,
            })
            .unwrap();

        let report = scanner.scan_recent().unwrap();
        let state = scanner.dashboard_state().unwrap();

        assert_eq!(report.token_events_added, 1);
        assert_eq!(report.limit_snapshots_added, 1);
        assert_eq!(state.limits.five_hour.used_percent, Some(42.0));
    }

    #[test]
    fn rollout_files_keeps_latest_files_first_and_caps_work() {
        let dir = tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        fs::create_dir_all(&sessions).unwrap();
        for index in 0..20 {
            let file = sessions.join(format!("rollout-2026-06-17T17-00-{index:02}-test.jsonl"));
            fs::write(&file, "{}").unwrap();
        }

        let files = rollout_files(&sessions, 30, MAX_FILES_PER_SCAN).unwrap();

        assert_eq!(files.len(), 12);
        assert!(files[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("17-00-19"));
    }
}
