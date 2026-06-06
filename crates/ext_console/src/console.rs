//! The `runtime:console` ops: capture, retrieve, and clear console records.

use deno_core::{op2, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use serde_json::Value;

use crate::console_listener::build_record;
use crate::console_log::{ConsoleRecord, ConsoleSource, ConsoleState};

/// Extension metadata.
#[weld_struct]
#[derive(Serialize)]
pub struct ExtensionInfo {
    /// Extension name.
    pub name: &'static str,
    /// Extension version.
    pub version: &'static str,
    /// Readiness status.
    pub status: &'static str,
}

/// Get extension information (name, version, status).
#[weld_op]
#[op2]
#[serde]
pub fn op_console_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_console",
        version: env!("CARGO_PKG_VERSION"),
        status: "ready",
    }
}

/// Capture a console record into the buffer.
///
/// `source` is `"deno"` or `"renderer"` (anything else is treated as renderer);
/// it defaults to `"deno"` when omitted. The record is timestamped on capture.
#[weld_op]
#[op2]
pub fn op_console_push(
    state: &mut OpState,
    #[string] level: String,
    #[serde] args: Vec<Value>,
    #[string] source: Option<String>,
) {
    capture(state, level, args, source.as_deref());
}

/// Return the most recent `n` console records in chronological order.
#[weld_op]
#[op2]
#[serde]
pub fn op_console_tail(state: &mut OpState, #[smi] n: u32) -> Vec<ConsoleRecord> {
    state.borrow::<ConsoleState>().tail(n as usize)
}

/// Clear all buffered console records, returning the number removed.
#[weld_op]
#[op2(fast)]
pub fn op_console_clear(state: &mut OpState) -> u32 {
    state.borrow_mut::<ConsoleState>().clear()
}

/// Core of [`op_console_push`], factored out so the source-defaulting and
/// capture path can be unit-tested without going through the generated op
/// wrapper (the `#[op2]` macro turns the op fns into non-callable `OpDecl`s).
///
/// `source` defaults to [`ConsoleSource::Deno`] when `None`; any other tag is
/// resolved via [`ConsoleSource::from_tag`].
fn capture(state: &mut OpState, level: String, args: Vec<Value>, source: Option<&str>) {
    let source = source
        .map(ConsoleSource::from_tag)
        .unwrap_or(ConsoleSource::Deno);
    let record = build_record(level, args, source, None);
    state.borrow_mut::<ConsoleState>().push(record);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn op_state_with_console() -> OpState {
        let mut op_state = OpState::new(None);
        op_state.put(ConsoleState::default());
        op_state
    }

    #[test]
    fn capture_defaults_source_to_deno_and_resolves_tags() {
        let mut state = op_state_with_console();
        capture(&mut state, "warn".to_string(), vec![json!("careful")], None);
        capture(
            &mut state,
            "error".to_string(),
            vec![json!("boom")],
            Some("renderer"),
        );

        let tail = state.borrow::<ConsoleState>().tail(10);
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0].level, "warn");
        assert_eq!(tail[0].source, ConsoleSource::Deno);
        assert_eq!(tail[1].level, "error");
        assert_eq!(tail[1].source, ConsoleSource::Renderer);
    }

    #[test]
    fn captured_record_lands_in_shared_state() {
        let mut state = op_state_with_console();
        capture(&mut state, "log".to_string(), vec![json!(1)], None);
        assert_eq!(state.borrow::<ConsoleState>().len(), 1);
        assert_eq!(state.borrow_mut::<ConsoleState>().clear(), 1);
        assert!(state.borrow::<ConsoleState>().is_empty());
    }
}
