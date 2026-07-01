use crate::models::{TokenEvent, UsageDelta};
use chrono::{DateTime, Local, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn parse_token_event(line: &str, source_file: &Path, line_number: usize) -> Option<TokenEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    let payload = value.get("payload")?;
    let payload_type = payload.get("type")?.as_str()?;
    if payload_type != "token_count" {
        return None;
    }

    let timestamp = value.get("timestamp")?.as_str()?.to_string();
    let usage = payload
        .get("info")?
        .get("last_token_usage")
        .and_then(parse_usage_delta)?;

    let local_date = timestamp_to_local_date(&timestamp)?;
    let source = source_file.to_string_lossy().to_string();
    let event_hash = event_hash(&source, line_number, &timestamp, line);

    Some(TokenEvent {
        event_hash,
        source_file: source,
        timestamp,
        local_date,
        usage,
    })
}

fn parse_usage_delta(value: &Value) -> Option<UsageDelta> {
    Some(UsageDelta {
        input_tokens: value.get("input_tokens")?.as_i64()?,
        cached_input_tokens: value
            .get("cached_input_tokens")
            .and_then(Value::as_i64)
            .unwrap_or(0),
        output_tokens: value.get("output_tokens")?.as_i64()?,
        reasoning_tokens: value
            .get("reasoning_output_tokens")
            .and_then(Value::as_i64)
            .unwrap_or(0),
        total_tokens: value.get("total_tokens")?.as_i64()?,
    })
}

fn timestamp_to_local_date(timestamp: &str) -> Option<String> {
    let utc: DateTime<Utc> = DateTime::parse_from_rfc3339(timestamp)
        .ok()?
        .with_timezone(&Utc);
    Some(utc.with_timezone(&Local).format("%Y-%m-%d").to_string())
}

fn event_hash(source_file: &str, line_number: usize, timestamp: &str, line: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_file.as_bytes());
    hasher.update(line_number.to_string().as_bytes());
    hasher.update(timestamp.as_bytes());
    hasher.update(line.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_last_token_usage_instead_of_cumulative_total() {
        let line = r#"{"timestamp":"2026-06-17T17:10:00.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":100,"output_tokens":80,"reasoning_output_tokens":20,"total_tokens":1080},"last_token_usage":{"input_tokens":250,"cached_input_tokens":50,"output_tokens":30,"reasoning_output_tokens":5,"total_tokens":280}},"rate_limits":null}}"#;

        let event = parse_token_event(line, &PathBuf::from("rollout.jsonl"), 12).unwrap();

        assert_eq!(event.usage.total_tokens, 280);
        assert_eq!(event.usage.input_tokens, 250);
        assert_eq!(event.usage.cached_input_tokens, 50);
        assert_eq!(event.usage.output_tokens, 30);
        assert_eq!(event.usage.reasoning_tokens, 5);
    }

    #[test]
    fn ignores_non_token_count_lines() {
        let line = r#"{"timestamp":"2026-06-17T17:10:00.000Z","type":"event_msg","payload":{"type":"agent_reasoning","text":"skip"}}"#;

        assert!(parse_token_event(line, &PathBuf::from("rollout.jsonl"), 1).is_none());
    }
}
