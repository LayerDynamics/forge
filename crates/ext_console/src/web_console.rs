//! Ingestion of console messages forwarded from the WebView renderer.
//!
//! The renderer patches its own `console.*` in preload and forwards each call
//! to the Deno side over the `window.host` IPC bridge as a JSON payload. The
//! handler turns that payload into a [`ConsoleRecord`] tagged
//! [`ConsoleSource::Renderer`] via [`parse_renderer_message`], then pushes it
//! into the shared [`crate::console_log::ConsoleState`] like any other record.

use serde_json::Value;

use crate::console_listener::build_record;
use crate::console_log::{ConsoleRecord, ConsoleSource};

/// Parse a renderer-forwarded console payload into a [`ConsoleRecord`].
///
/// Expected shape (extra fields are ignored):
/// ```json
/// { "level": "warn", "args": ["msg", 42], "timestamp": 1700000000000 }
/// ```
///
/// `args` may be a single value or an array; a missing `args` yields an empty
/// argument list. Returns `None` only when `payload` is not a JSON object, so a
/// malformed message is dropped rather than panicking.
pub fn parse_renderer_message(payload: &Value) -> Option<ConsoleRecord> {
    let obj = payload.as_object()?;

    let level = obj
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("log")
        .to_string();

    let args = match obj.get("args") {
        Some(Value::Array(items)) => items.clone(),
        Some(other) => vec![other.clone()],
        None => Vec::new(),
    };

    // Accept either "timestamp" or "timestamp_ms"; renderers vary. Read as f64
    // to match the record's millisecond representation (JS time is a number).
    let timestamp_ms = obj
        .get("timestamp")
        .or_else(|| obj.get("timestamp_ms"))
        .and_then(Value::as_f64);

    Some(build_record(
        level,
        args,
        ConsoleSource::Renderer,
        timestamp_ms,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_full_payload() {
        let payload = json!({
            "level": "error",
            "args": ["boom", { "code": 500 }],
            "timestamp": 1_700_000_000_000u64,
        });
        let rec = parse_renderer_message(&payload).expect("should parse");
        assert_eq!(rec.level, "error");
        assert_eq!(rec.args.len(), 2);
        assert_eq!(rec.args[0], json!("boom"));
        assert_eq!(rec.timestamp_ms, 1_700_000_000_000.0);
        assert_eq!(rec.source, ConsoleSource::Renderer);
    }

    #[test]
    fn single_arg_is_wrapped() {
        let rec = parse_renderer_message(&json!({ "level": "log", "args": "hello" })).unwrap();
        assert_eq!(rec.args, vec![json!("hello")]);
    }

    #[test]
    fn missing_fields_default() {
        let rec = parse_renderer_message(&json!({})).unwrap();
        assert_eq!(rec.level, "log");
        assert!(rec.args.is_empty());
        assert_eq!(rec.source, ConsoleSource::Renderer);
    }

    #[test]
    fn non_object_is_rejected() {
        assert!(parse_renderer_message(&json!("not an object")).is_none());
        assert!(parse_renderer_message(&json!([1, 2, 3])).is_none());
    }
}
