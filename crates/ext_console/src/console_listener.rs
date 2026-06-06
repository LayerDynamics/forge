//! Construction of [`ConsoleRecord`]s from raw console-call parts.
//!
//! Both ingestion paths funnel through here so Deno-side captures and
//! renderer-forwarded messages produce identically-shaped records: the Deno
//! `console.*` shim (via `op_console_push`) and the renderer payload parser in
//! [`crate::web_console`].

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::console_log::{ConsoleRecord, ConsoleSource};

/// Current wall-clock time in milliseconds since the UNIX epoch.
///
/// Returned as `f64` to match the [`ConsoleRecord::timestamp_ms`] field and
/// JavaScript's native time representation. Returns 0 if the system clock is set
/// before the epoch (it never is in practice), so capture can't fail on a
/// timestamp.
pub fn now_millis() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
}

/// Build a [`ConsoleRecord`] from its parts, stamping it with the current time
/// unless an explicit `timestamp_ms` is supplied (renderer payloads may carry
/// their own).
pub fn build_record(
    level: impl Into<String>,
    args: Vec<Value>,
    source: ConsoleSource,
    timestamp_ms: Option<f64>,
) -> ConsoleRecord {
    ConsoleRecord {
        level: normalize_level(&level.into()),
        args,
        timestamp_ms: timestamp_ms.unwrap_or_else(now_millis),
        source,
    }
}

/// Normalize a console level to one of the known names, mapping common aliases
/// and falling back to "log" for anything unrecognized.
fn normalize_level(level: &str) -> String {
    match level.trim().to_ascii_lowercase().as_str() {
        "" | "log" => "log",
        "info" => "info",
        "warn" | "warning" => "warn",
        "error" | "err" => "error",
        "debug" => "debug",
        "trace" => "trace",
        other => return other.to_string(),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn build_record_stamps_time_when_absent() {
        let before = now_millis();
        let r = build_record("info", vec![json!("hi")], ConsoleSource::Deno, None);
        assert_eq!(r.level, "info");
        assert!(r.timestamp_ms >= before);
        assert_eq!(r.source, ConsoleSource::Deno);
    }

    #[test]
    fn build_record_honors_explicit_timestamp() {
        let r = build_record("log", vec![], ConsoleSource::Renderer, Some(42.0));
        assert_eq!(r.timestamp_ms, 42.0);
    }

    #[test]
    fn level_aliases_normalize() {
        assert_eq!(
            build_record("WARNING", vec![], ConsoleSource::Deno, Some(0.0)).level,
            "warn"
        );
        assert_eq!(
            build_record("", vec![], ConsoleSource::Deno, Some(0.0)).level,
            "log"
        );
        assert_eq!(
            build_record("err", vec![], ConsoleSource::Deno, Some(0.0)).level,
            "error"
        );
    }
}
