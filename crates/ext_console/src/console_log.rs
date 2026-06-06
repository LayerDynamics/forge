//! Console record data model and the in-memory ring buffer.
//!
//! [`ConsoleState`] holds a bounded, FIFO buffer of the most recent
//! [`ConsoleRecord`]s captured from either the Deno side or the WebView
//! renderer. It is stored in `OpState` and read back through the
//! `op_console_*` ops.

use std::collections::VecDeque;

use forge_weld_macro::{weld_enum, weld_struct};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Default number of records retained before the oldest are evicted.
pub const DEFAULT_CAPACITY: usize = 1000;

/// Where a captured console record originated.
#[weld_enum]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleSource {
    /// Emitted by `console.*` on the Deno (main) side.
    Deno,
    /// Forwarded from the WebView renderer over the IPC bridge.
    Renderer,
}

impl ConsoleSource {
    /// Parse a source tag (e.g. from a renderer IPC payload), defaulting to
    /// [`ConsoleSource::Renderer`] for unknown values.
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_ascii_lowercase().as_str() {
            "deno" | "main" => ConsoleSource::Deno,
            _ => ConsoleSource::Renderer,
        }
    }
}

/// A single captured console call.
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleRecord {
    /// Console level: "log", "info", "warn", "error", "debug".
    pub level: String,
    /// The arguments passed to the console call, preserved as JSON values.
    pub args: Vec<Value>,
    /// Wall-clock capture time in milliseconds since the UNIX epoch.
    ///
    /// Stored as `f64` to match JavaScript's native time representation
    /// (`Date.now()` is a `number`): millisecond timestamps are exact in `f64`
    /// well beyond any realistic date, and serde_v8 serializes safe-range
    /// values as a JS `number` rather than a `bigint`.
    pub timestamp_ms: f64,
    /// Whether the record came from the Deno side or the renderer.
    pub source: ConsoleSource,
}

/// Bounded FIFO buffer of recent console records, stored in `OpState`.
pub struct ConsoleState {
    buffer: VecDeque<ConsoleRecord>,
    capacity: usize,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl ConsoleState {
    /// Create a state with a specific retention capacity (minimum 1).
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Append a record, evicting the oldest if at capacity.
    pub fn push(&mut self, record: ConsoleRecord) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(record);
    }

    /// Number of records currently buffered.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Return the last `n` records in chronological (oldest-first) order.
    ///
    /// `n` larger than the buffer returns the whole buffer; `n == 0` returns an
    /// empty vector.
    pub fn tail(&self, n: usize) -> Vec<ConsoleRecord> {
        let start = self.buffer.len().saturating_sub(n);
        self.buffer.iter().skip(start).cloned().collect()
    }

    /// Clear all buffered records, returning how many were removed (saturating
    /// at `u32::MAX`, the op/SDK type, rather than wrapping).
    pub fn clear(&mut self) -> u32 {
        let removed = u32::try_from(self.buffer.len()).unwrap_or(u32::MAX);
        self.buffer.clear();
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rec(level: &str, msg: &str) -> ConsoleRecord {
        ConsoleRecord {
            level: level.to_string(),
            args: vec![json!(msg)],
            timestamp_ms: 0.0,
            source: ConsoleSource::Deno,
        }
    }

    #[test]
    fn tail_returns_last_n_in_order() {
        let mut state = ConsoleState::with_capacity(10);
        for i in 0..5 {
            state.push(rec("log", &format!("m{i}")));
        }
        let last3 = state.tail(3);
        assert_eq!(last3.len(), 3);
        assert_eq!(last3[0].args[0], json!("m2"));
        assert_eq!(last3[2].args[0], json!("m4"));
    }

    #[test]
    fn tail_larger_than_buffer_returns_all() {
        let mut state = ConsoleState::with_capacity(10);
        state.push(rec("log", "only"));
        assert_eq!(state.tail(100).len(), 1);
        assert_eq!(state.tail(0).len(), 0);
    }

    #[test]
    fn capacity_evicts_oldest() {
        let mut state = ConsoleState::with_capacity(3);
        for i in 0..5 {
            state.push(rec("log", &format!("m{i}")));
        }
        assert_eq!(state.len(), 3);
        let all = state.tail(10);
        // Oldest two (m0, m1) evicted; m2..m4 remain in order.
        assert_eq!(all[0].args[0], json!("m2"));
        assert_eq!(all[2].args[0], json!("m4"));
    }

    #[test]
    fn clear_reports_count_and_empties() {
        let mut state = ConsoleState::with_capacity(10);
        state.push(rec("warn", "a"));
        state.push(rec("error", "b"));
        assert_eq!(state.clear(), 2);
        assert!(state.is_empty());
        assert_eq!(state.clear(), 0);
    }

    #[test]
    fn capacity_floor_is_one() {
        let mut state = ConsoleState::with_capacity(0);
        state.push(rec("log", "x"));
        state.push(rec("log", "y"));
        assert_eq!(state.len(), 1);
        assert_eq!(state.tail(1)[0].args[0], json!("y"));
    }
}
