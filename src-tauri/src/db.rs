use crate::models::{
    DashboardState, Freshness, HeatmapDay, LimitBucket, LimitSnapshot, LimitSummary, Settings,
    TodayUsage, TokenEvent,
};
use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_STALE_MINUTES: i64 = 5;
const DEFAULT_HEATMAP_DAYS: i64 = 366;

pub struct AppDb {
    conn: Connection,
}

impl AppDb {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let db = Self {
            conn: Connection::open_in_memory()?,
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("CodexUsageWidget")
            .join("usage.sqlite")
    }

    fn migrate(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS usage_daily (
              date TEXT PRIMARY KEY,
              total_tokens INTEGER DEFAULT 0,
              input_tokens INTEGER DEFAULT 0,
              cached_input_tokens INTEGER DEFAULT 0,
              output_tokens INTEGER DEFAULT 0,
              reasoning_tokens INTEGER DEFAULT 0,
              session_count INTEGER DEFAULT 0,
              updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS limit_snapshots (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              captured_at TEXT,
              source_file TEXT,
              five_hour_used_percent REAL,
              five_hour_reset_at INTEGER,
              weekly_used_percent REAL,
              weekly_reset_at INTEGER,
              plan_type TEXT,
              unusual INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS processed_events (
              event_hash TEXT PRIMARY KEY,
              source_file TEXT,
              timestamp TEXT
            );

            CREATE TABLE IF NOT EXISTS app_config (
              key TEXT PRIMARY KEY,
              value TEXT NOT NULL
            );
            "#,
        )
    }

    pub fn insert_token_event(&self, event: &TokenEvent) -> Result<bool, rusqlite::Error> {
        let inserted = self.conn.execute(
            "INSERT OR IGNORE INTO processed_events (event_hash, source_file, timestamp) VALUES (?1, ?2, ?3)",
            params![event.event_hash, event.source_file, event.timestamp],
        )?;

        if inserted == 0 {
            return Ok(false);
        }

        self.conn.execute(
            r#"
            INSERT INTO usage_daily (
              date,
              total_tokens,
              input_tokens,
              cached_input_tokens,
              output_tokens,
              reasoning_tokens,
              session_count,
              updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)
            ON CONFLICT(date) DO UPDATE SET
              total_tokens = total_tokens + excluded.total_tokens,
              input_tokens = input_tokens + excluded.input_tokens,
              cached_input_tokens = cached_input_tokens + excluded.cached_input_tokens,
              output_tokens = output_tokens + excluded.output_tokens,
              reasoning_tokens = reasoning_tokens + excluded.reasoning_tokens,
              session_count = session_count + 1,
              updated_at = excluded.updated_at
            "#,
            params![
                event.local_date,
                event.usage.total_tokens,
                event.usage.input_tokens,
                event.usage.cached_input_tokens,
                event.usage.output_tokens,
                event.usage.reasoning_tokens,
                event.timestamp,
            ],
        )?;

        Ok(true)
    }

    pub fn insert_limit_snapshot(&self, snapshot: &LimitSnapshot) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            r#"
            INSERT INTO limit_snapshots (
              captured_at,
              source_file,
              five_hour_used_percent,
              five_hour_reset_at,
              weekly_used_percent,
              weekly_reset_at,
              plan_type,
              unusual
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                snapshot.captured_at,
                snapshot.source_file,
                snapshot.five_hour_used_percent,
                snapshot.five_hour_reset_at,
                snapshot.weekly_used_percent,
                snapshot.weekly_reset_at,
                snapshot.plan_type,
                snapshot.unusual as i64,
            ],
        )?;
        Ok(())
    }

    pub fn latest_limit_snapshot(&self) -> Result<Option<LimitSnapshot>, rusqlite::Error> {
        self.conn
            .query_row(
                r#"
                SELECT
                  captured_at,
                  source_file,
                  five_hour_used_percent,
                  five_hour_reset_at,
                  weekly_used_percent,
                  weekly_reset_at,
                  plan_type,
                  unusual
                FROM limit_snapshots
                ORDER BY captured_at DESC, id DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok(LimitSnapshot {
                        captured_at: row.get(0)?,
                        source_file: row.get(1)?,
                        five_hour_used_percent: row.get(2)?,
                        five_hour_reset_at: row.get(3)?,
                        weekly_used_percent: row.get(4)?,
                        weekly_reset_at: row.get(5)?,
                        plan_type: row.get(6)?,
                        unusual: row.get::<_, i64>(7)? != 0,
                    })
                },
            )
            .optional()
    }

    pub fn today_usage(&self, date: &str) -> Result<TodayUsage, rusqlite::Error> {
        self.conn
            .query_row(
                r#"
                SELECT
                  date,
                  total_tokens,
                  input_tokens,
                  cached_input_tokens,
                  output_tokens,
                  reasoning_tokens,
                  session_count
                FROM usage_daily
                WHERE date = ?1
                "#,
                params![date],
                |row| {
                    Ok(TodayUsage {
                        date: row.get(0)?,
                        total_tokens: row.get(1)?,
                        input_tokens: row.get(2)?,
                        cached_input_tokens: row.get(3)?,
                        output_tokens: row.get(4)?,
                        reasoning_tokens: row.get(5)?,
                        sessions: row.get(6)?,
                    })
                },
            )
            .optional()
            .map(|value| {
                value.unwrap_or_else(|| TodayUsage {
                    date: date.to_string(),
                    ..TodayUsage::default()
                })
            })
    }

    pub fn heatmap_days(&self, count: i64) -> Result<Vec<HeatmapDay>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, total_tokens, session_count
            FROM usage_daily
            ORDER BY date DESC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![count], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
        })?;

        let mut raw = Vec::new();
        for row in rows {
            raw.push(row?);
        }
        raw.reverse();

        let max = raw.iter().map(|(_, total, _)| *total).max().unwrap_or(0);
        Ok(raw
            .into_iter()
            .map(|(date, total_tokens, sessions)| HeatmapDay {
                date,
                total_tokens,
                sessions,
                level: heatmap_level(total_tokens, max),
            })
            .collect())
    }

    pub fn get_settings(&self, sessions_path: String) -> Result<Settings, rusqlite::Error> {
        Ok(Settings {
            sessions_path: self
                .get_config("sessions_path")?
                .unwrap_or(sessions_path),
            always_on_top: self
                .get_config("always_on_top")?
                .map(|value| value == "true")
                .unwrap_or(true),
            stale_after_minutes: self
                .get_config("stale_after_minutes")?
                .and_then(|value| value.parse().ok())
                .unwrap_or(DEFAULT_STALE_MINUTES),
            heatmap_days: self
                .get_config("heatmap_days")?
                .and_then(|value| value.parse().ok())
                .map(|value: i64| value.max(DEFAULT_HEATMAP_DAYS))
                .unwrap_or(DEFAULT_HEATMAP_DAYS),
        })
    }

    pub fn update_settings(&self, settings: &Settings) -> Result<(), rusqlite::Error> {
        self.set_config("sessions_path", &settings.sessions_path)?;
        self.set_config("always_on_top", if settings.always_on_top { "true" } else { "false" })?;
        self.set_config("stale_after_minutes", &settings.stale_after_minutes.to_string())?;
        self.set_config("heatmap_days", &settings.heatmap_days.to_string())?;
        Ok(())
    }

    pub fn record_scan_completed(&self, scanned_at: DateTime<Utc>) -> Result<(), rusqlite::Error> {
        self.set_config("last_scan_at", &scanned_at.to_rfc3339())
    }

    pub fn dashboard_state(
        &self,
        settings: &Settings,
        source_path: String,
    ) -> Result<DashboardState, rusqlite::Error> {
        let today_key = Local::now().format("%Y-%m-%d").to_string();
        let today = self.today_usage(&today_key)?;
        let heatmap_days = self.heatmap_days(settings.heatmap_days)?;
        let latest_limit = self.latest_limit_snapshot()?;
        let updated_at = self
            .get_config("last_scan_at")?
            .or_else(|| latest_limit.as_ref().map(|snapshot| snapshot.captured_at.clone()));
        let freshness = freshness(updated_at.as_deref(), settings.stale_after_minutes);

        let mut warnings = Vec::new();
        if latest_limit.is_none() {
            warnings.push("No rate_limits found in latest Codex session logs.".to_string());
        }
        if latest_limit.as_ref().map(|snapshot| snapshot.unusual).unwrap_or(false) {
            warnings.push("Weekly usage has an unusual reset window.".to_string());
        }

        Ok(DashboardState {
            source_path,
            updated_at,
            freshness,
            warnings,
            limits: limit_summary(latest_limit),
            today,
            heatmap_days,
        })
    }

    fn get_config(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        self.conn
            .query_row("SELECT value FROM app_config WHERE key = ?1", params![key], |row| row.get(0))
            .optional()
    }

    fn set_config(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn limit_summary(snapshot: Option<LimitSnapshot>) -> LimitSummary {
    let Some(snapshot) = snapshot else {
        return LimitSummary {
            five_hour: unavailable("5h"),
            weekly: unavailable("Weekly"),
            plan_type: None,
        };
    };

    LimitSummary {
        five_hour: bucket(
            "5h",
            snapshot.five_hour_used_percent,
            snapshot.five_hour_reset_at,
            false,
        ),
        weekly: bucket(
            "Weekly",
            snapshot.weekly_used_percent,
            snapshot.weekly_reset_at,
            snapshot.unusual,
        ),
        plan_type: snapshot.plan_type,
    }
}

fn bucket(label: &str, used_percent: Option<f64>, reset_at: Option<i64>, unusual: bool) -> LimitBucket {
    let (used_percent, reset_at) = normalize_bucket_after_reset(label, used_percent, reset_at);

    LimitBucket {
        label: label.to_string(),
        used_percent,
        remaining_percent: used_percent.map(|used| (100.0 - used).max(0.0)),
        reset_at,
        reset_label: reset_at.map(reset_label),
        available: used_percent.is_some(),
        unusual,
    }
}

fn normalize_bucket_after_reset(
    label: &str,
    used_percent: Option<f64>,
    reset_at: Option<i64>,
) -> (Option<f64>, Option<i64>) {
    if used_percent.is_none() {
        return (used_percent, reset_at);
    }

    let Some(reset_at_value) = reset_at else {
        return (used_percent, reset_at);
    };

    let now = Utc::now().timestamp();
    if reset_at_value > now {
        return (used_percent, reset_at);
    }

    let Some(window_seconds) = window_seconds(label) else {
        return (used_percent, reset_at);
    };

    let mut next_reset = reset_at_value;
    while next_reset <= now {
        next_reset += window_seconds;
    }

    (Some(1.0), Some(next_reset))
}

fn window_seconds(label: &str) -> Option<i64> {
    match label {
        "5h" => Some(300 * 60),
        "Weekly" => Some(10_080 * 60),
        _ => None,
    }
}

fn unavailable(label: &str) -> LimitBucket {
    LimitBucket {
        label: label.to_string(),
        used_percent: None,
        remaining_percent: None,
        reset_at: None,
        reset_label: None,
        available: false,
        unusual: false,
    }
}

fn freshness(updated_at: Option<&str>, stale_after_minutes: i64) -> Freshness {
    let Some(updated_at) = updated_at else {
        return Freshness {
            state: "missing".to_string(),
            label: "No local data yet".to_string(),
            age_seconds: None,
        };
    };

    let Ok(parsed) = DateTime::parse_from_rfc3339(updated_at) else {
        return Freshness {
            state: "stale".to_string(),
            label: "Updated time unavailable".to_string(),
            age_seconds: None,
        };
    };

    let now = Utc::now();
    let age = now.signed_duration_since(parsed.with_timezone(&Utc)).num_seconds().max(0);
    let state = if age > stale_after_minutes * 60 {
        "stale"
    } else {
        "fresh"
    };

    Freshness {
        state: state.to_string(),
        label: format!("Updated {} ago", age_label(age)),
        age_seconds: Some(age),
    }
}

fn reset_label(reset_at: i64) -> String {
    let now = Utc::now().timestamp();
    let seconds = (reset_at - now).max(0);
    if seconds == 0 {
        return "now".to_string();
    }

    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{}m", minutes.max(1))
    }
}

fn age_label(seconds: i64) -> String {
    if seconds < 60 {
        return format!("{seconds}s");
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }
    format!("{}h {}m", minutes / 60, minutes % 60)
}

fn heatmap_level(total_tokens: i64, max_tokens: i64) -> i64 {
    if total_tokens <= 0 || max_tokens <= 0 {
        return 0;
    }
    let ratio = total_tokens as f64 / max_tokens as f64;
    if ratio <= 0.15 {
        1
    } else if ratio <= 0.35 {
        2
    } else if ratio <= 0.65 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UsageDelta;

    fn event(hash: &str, total: i64, date: &str) -> TokenEvent {
        TokenEvent {
            event_hash: hash.to_string(),
            source_file: "rollout.jsonl".to_string(),
            timestamp: format!("{date}T12:00:00.000Z"),
            local_date: date.to_string(),
            usage: UsageDelta {
                total_tokens: total,
                input_tokens: total - 10,
                cached_input_tokens: 5,
                output_tokens: 10,
                reasoning_tokens: 1,
            },
        }
    }

    #[test]
    fn dedupes_processed_events() {
        let db = AppDb::in_memory().unwrap();
        let first = db.insert_token_event(&event("same", 100, "2026-06-17")).unwrap();
        let second = db.insert_token_event(&event("same", 100, "2026-06-17")).unwrap();

        assert!(first);
        assert!(!second);
        assert_eq!(db.today_usage("2026-06-17").unwrap().total_tokens, 100);
    }

    #[test]
    fn daily_rows_accumulate_multiple_events() {
        let db = AppDb::in_memory().unwrap();
        db.insert_token_event(&event("one", 100, "2026-06-17")).unwrap();
        db.insert_token_event(&event("two", 250, "2026-06-17")).unwrap();

        let usage = db.today_usage("2026-06-17").unwrap();

        assert_eq!(usage.total_tokens, 350);
        assert_eq!(usage.sessions, 2);
    }

    #[test]
    fn latest_limit_snapshot_is_selected_by_timestamp() {
        let db = AppDb::in_memory().unwrap();
        db.insert_limit_snapshot(&LimitSnapshot {
            captured_at: "2026-06-17T17:00:00.000Z".to_string(),
            source_file: "old.jsonl".to_string(),
            five_hour_used_percent: Some(10.0),
            five_hour_reset_at: Some(1_781_720_400),
            weekly_used_percent: Some(20.0),
            weekly_reset_at: Some(1_782_079_200),
            plan_type: Some("plus".to_string()),
            unusual: false,
        })
        .unwrap();
        db.insert_limit_snapshot(&LimitSnapshot {
            captured_at: "2026-06-17T18:00:00.000Z".to_string(),
            source_file: "new.jsonl".to_string(),
            five_hour_used_percent: Some(42.0),
            five_hour_reset_at: Some(1_781_724_000),
            weekly_used_percent: Some(68.0),
            weekly_reset_at: Some(1_782_079_200),
            plan_type: Some("plus".to_string()),
            unusual: false,
        })
        .unwrap();

        assert_eq!(
            db.latest_limit_snapshot().unwrap().unwrap().five_hour_used_percent,
            Some(42.0)
        );
    }

    #[test]
    fn dashboard_freshness_uses_last_successful_scan_time() {
        let db = AppDb::in_memory().unwrap();
        db.insert_limit_snapshot(&LimitSnapshot {
            captured_at: "2026-06-17T18:00:00.000Z".to_string(),
            source_file: "old.jsonl".to_string(),
            five_hour_used_percent: Some(42.0),
            five_hour_reset_at: Some(1_781_724_000),
            weekly_used_percent: Some(68.0),
            weekly_reset_at: Some(1_782_079_200),
            plan_type: Some("plus".to_string()),
            unusual: false,
        })
        .unwrap();
        db.record_scan_completed(Utc::now()).unwrap();

        let settings = db.get_settings("sessions".to_string()).unwrap();
        let state = db.dashboard_state(&settings, "sessions".to_string()).unwrap();

        assert_eq!(state.freshness.state, "fresh");
        assert_eq!(state.updated_at, db.get_config("last_scan_at").unwrap());
    }

    #[test]
    fn expired_limit_bucket_displays_as_reset_remaining() {
        let db = AppDb::in_memory().unwrap();
        db.insert_limit_snapshot(&LimitSnapshot {
            captured_at: Utc::now().to_rfc3339(),
            source_file: "rollout.jsonl".to_string(),
            five_hour_used_percent: Some(25.0),
            five_hour_reset_at: Some(Utc::now().timestamp() - 1),
            weekly_used_percent: Some(16.0),
            weekly_reset_at: Some(Utc::now().timestamp() + 86_400),
            plan_type: Some("plus".to_string()),
            unusual: false,
        })
        .unwrap();

        let settings = db.get_settings("sessions".to_string()).unwrap();
        let state = db.dashboard_state(&settings, "sessions".to_string()).unwrap();

        assert_eq!(state.limits.five_hour.used_percent, Some(1.0));
        assert_eq!(state.limits.five_hour.remaining_percent, Some(99.0));
        assert_ne!(state.limits.five_hour.reset_label.as_deref(), Some("now"));
    }
}
