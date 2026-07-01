use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Freshness {
    pub state: String,
    pub label: String,
    pub age_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitBucket {
    pub label: String,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub reset_at: Option<i64>,
    pub reset_label: Option<String>,
    pub available: bool,
    pub unusual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitSummary {
    pub five_hour: LimitBucket,
    pub weekly: LimitBucket,
    pub plan_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayUsage {
    pub date: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDay {
    pub date: String,
    pub total_tokens: i64,
    pub sessions: i64,
    pub level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardState {
    pub source_path: String,
    pub updated_at: Option<String>,
    pub freshness: Freshness,
    pub warnings: Vec<String>,
    pub limits: LimitSummary,
    pub today: TodayUsage,
    pub heatmap_days: Vec<HeatmapDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub sessions_path: String,
    pub always_on_top: bool,
    pub stale_after_minutes: i64,
    pub heatmap_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsState {
    pub app_version: String,
    pub platform: String,
    pub arch: String,
    pub sessions_path: String,
    pub sessions_exists: bool,
    pub sessions_readable: bool,
    pub database_path: String,
    pub log_directory: String,
    pub last_scan_started_at: Option<String>,
    pub last_scan_completed_at: Option<String>,
    pub last_successful_data_update: Option<String>,
    pub watcher_status: String,
    pub files_scanned: usize,
    pub token_events_accepted: usize,
    pub limit_snapshots_accepted: usize,
    pub malformed_lines: usize,
    pub io_failures: usize,
    pub last_scan_result: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UsageDelta {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
}

#[derive(Debug, Clone)]
pub struct TokenEvent {
    pub event_hash: String,
    pub source_file: String,
    pub timestamp: String,
    pub local_date: String,
    pub usage: UsageDelta,
}

#[derive(Debug, Clone)]
pub struct LimitSnapshot {
    pub captured_at: String,
    pub source_file: String,
    pub five_hour_used_percent: Option<f64>,
    pub five_hour_reset_at: Option<i64>,
    pub weekly_used_percent: Option<f64>,
    pub weekly_reset_at: Option<i64>,
    pub plan_type: Option<String>,
    pub unusual: bool,
}
