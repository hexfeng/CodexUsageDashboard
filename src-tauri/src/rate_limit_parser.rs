use crate::models::LimitSnapshot;
use serde_json::Value;
use std::path::Path;

pub fn parse_limit_snapshot(
    line: &str,
    source_file: &Path,
    previous: Option<&LimitSnapshot>,
) -> Option<LimitSnapshot> {
    let value: Value = serde_json::from_str(line).ok()?;
    let payload = value.get("payload")?;
    let limits = payload.get("rate_limits")?;
    if limits.is_null() {
        return None;
    }

    let timestamp = value.get("timestamp")?.as_str()?.to_string();
    let primary = limits.get("primary");
    let secondary = limits.get("secondary");
    let five_hour_used_percent = parse_window_used(primary, 300);
    let five_hour_reset_at = parse_window_reset(primary, 300);
    let weekly_used_percent = parse_window_used(secondary, 10_080);
    let weekly_reset_at = parse_window_reset(secondary, 10_080);
    let plan_type = limits
        .get("plan_type")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let unusual = is_unusual_weekly_snapshot(previous, weekly_used_percent, weekly_reset_at);

    Some(LimitSnapshot {
        captured_at: timestamp,
        source_file: source_file.to_string_lossy().to_string(),
        five_hour_used_percent,
        five_hour_reset_at,
        weekly_used_percent,
        weekly_reset_at,
        plan_type,
        unusual,
    })
}

fn parse_window_used(window: Option<&Value>, expected_minutes: i64) -> Option<f64> {
    let window = window?;
    let minutes = window.get("window_minutes")?.as_i64()?;
    if minutes != expected_minutes {
        return None;
    }
    window.get("used_percent")?.as_f64()
}

fn parse_window_reset(window: Option<&Value>, expected_minutes: i64) -> Option<i64> {
    let window = window?;
    let minutes = window.get("window_minutes")?.as_i64()?;
    if minutes != expected_minutes {
        return None;
    }
    window.get("resets_at").and_then(Value::as_i64)
}

fn is_unusual_weekly_snapshot(
    previous: Option<&LimitSnapshot>,
    weekly_used_percent: Option<f64>,
    weekly_reset_at: Option<i64>,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    let jump = match (previous.weekly_used_percent, weekly_used_percent) {
        (Some(before), Some(after)) => (after - before).abs() > 50.0,
        _ => false,
    };

    let reset_moved_back = match (previous.weekly_reset_at, weekly_reset_at) {
        (Some(before), Some(after)) => after < before - 3_600,
        _ => false,
    };

    jump || reset_moved_back
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_primary_secondary_and_plan_type() {
        let line = r#"{"timestamp":"2026-06-17T17:10:00.000Z","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":42.0,"window_minutes":300,"resets_at":1781720400},"secondary":{"used_percent":68.0,"window_minutes":10080,"resets_at":1782079200},"plan_type":"plus"}}}"#;

        let snapshot = parse_limit_snapshot(line, &PathBuf::from("rollout.jsonl"), None).unwrap();

        assert_eq!(snapshot.five_hour_used_percent, Some(42.0));
        assert_eq!(snapshot.five_hour_reset_at, Some(1_781_720_400));
        assert_eq!(snapshot.weekly_used_percent, Some(68.0));
        assert_eq!(snapshot.weekly_reset_at, Some(1_782_079_200));
        assert_eq!(snapshot.plan_type.as_deref(), Some("plus"));
    }

    #[test]
    fn returns_none_when_rate_limits_is_null() {
        let line = r#"{"timestamp":"2026-06-17T17:10:00.000Z","payload":{"type":"token_count","rate_limits":null}}"#;

        assert!(parse_limit_snapshot(line, &PathBuf::from("rollout.jsonl"), None).is_none());
    }

    #[test]
    fn flags_unusual_weekly_jump() {
        let previous = LimitSnapshot {
            captured_at: "2026-06-17T17:00:00.000Z".to_string(),
            source_file: "old.jsonl".to_string(),
            five_hour_used_percent: Some(20.0),
            five_hour_reset_at: Some(1_781_720_400),
            weekly_used_percent: Some(20.0),
            weekly_reset_at: Some(1_782_079_200),
            plan_type: Some("plus".to_string()),
            unusual: false,
        };
        let line = r#"{"timestamp":"2026-06-17T17:10:00.000Z","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":42.0,"window_minutes":300,"resets_at":1781720400},"secondary":{"used_percent":82.0,"window_minutes":10080,"resets_at":1782079200},"plan_type":"plus"}}}"#;

        let snapshot = parse_limit_snapshot(line, &PathBuf::from("rollout.jsonl"), Some(&previous)).unwrap();

        assert!(snapshot.unusual);
    }
}
