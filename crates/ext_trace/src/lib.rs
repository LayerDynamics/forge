//! runtime:trace extension - Lightweight tracing for Forge applications
//!
//! Provides simple span-based performance tracking with manual start/end lifecycle
//! management. Spans measure operation duration using high-resolution timing
//! (`Instant::now()`), support arbitrary JSON attributes/results, and can be
//! batched for export via `flush()`.
//!
//! **Runtime Module:** `runtime:trace`
//!
//! ## Overview
//!
//! `ext_trace` is a minimalist tracing extension designed for application-level
//! performance instrumentation. Unlike full distributed tracing systems (OpenTelemetry,
//! Jaeger), it provides simple in-memory span tracking with manual lifecycle management.
//!
//! Key design characteristics:
//! - **Manual Lifecycle**: Spans are started/ended explicitly (no automatic scoping)
//! - **Flat Structure**: No parent-child relationships tracked (independent spans)
//! - **In-Memory Buffer**: Finished spans stored in `Vec<SpanRecord>` until flushed
//! - **Synchronous Operations**: All ops are synchronous (except timing measurement)
//! - **No Sampling**: Every span is recorded (application decides what to trace)
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │ TypeScript Application (runtime:trace)                     │
//! │  spanId = start(name, attrs?) -> end(spanId, result?)      │
//! └────────────────┬───────────────────────────────────────────┘
//!                  │ Deno Ops (op_trace_*)
//!                  ↓
//! ┌────────────────────────────────────────────────────────────┐
//! │ ext_trace (TraceState in OpState)                          │
//! │  - active: HashMap<u64, ActiveSpan>                        │
//! │  - finished: Vec<SpanRecord>                               │
//! │  - next_id: u64 (monotonic counter)                        │
//! └────────────────┬───────────────────────────────────────────┘
//!                  │ std::time::Instant/SystemTime
//!                  ↓
//! ┌────────────────────────────────────────────────────────────┐
//! │ High-Resolution Timing                                     │
//! │  - Instant::now() for duration measurement                 │
//! │  - SystemTime for wall-clock timestamps (UNIX epoch)       │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Operations
//!
//! The extension provides 5 operations:
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_trace_info` | `ExtensionInfo` | Extension metadata (name, version, status) |
//! | `op_trace_start` | `u64` | Start span, return unique ID |
//! | `op_trace_end` | `SpanRecord` | End span by ID, return completed record |
//! | `op_trace_instant` | `SpanRecord` | Record zero-duration event |
//! | `op_trace_flush` | `Vec<SpanRecord>` | Drain finished spans buffer |
//!
//! ## Data Structures
//!
//! ### TraceState
//!
//! Main state struct stored in `OpState`:
//! ```ignore
//! pub struct TraceState {
//!     next_id: u64,                      // Monotonic span ID counter
//!     active: HashMap<u64, ActiveSpan>,  // Spans currently timing
//!     finished: Vec<SpanRecord>,         // Completed spans (until flushed)
//! }
//! ```
//!
//! ### ActiveSpan (Internal)
//!
//! Active span tracking (not exposed to TypeScript):
//! ```ignore
//! struct ActiveSpan {
//!     id: u64,
//!     name: String,
//!     started: Instant,           // For duration calculation
//!     wall_clock: SystemTime,     // For timestamp export
//!     attributes: Option<Value>,  // JSON metadata
//! }
//! ```
//!
//! ### SpanRecord (Public)
//!
//! Completed span record (returned to TypeScript):
//! ```ignore
//! #[weld_struct]
//! pub struct SpanRecord {
//!     pub id: u64,                    // Span ID
//!     pub name: String,               // Span name
//!     pub started_at: u128,           // Millis since UNIX epoch
//!     pub duration_ms: f64,           // Elapsed milliseconds
//!     pub attributes: Option<Value>,  // Original attributes
//!     pub result: Option<Value>,      // Result from end()
//! }
//! ```
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { start, end, instant, flush } from "runtime:trace";
//!
//! // Manual span tracking
//! async function fetchUser(id: number) {
//!   const spanId = start("fetchUser", { userId: id });
//!
//!   try {
//!     const response = await fetch(`/api/users/${id}`);
//!     const user = await response.json();
//!     return end(spanId, { status: response.status });
//!   } catch (error) {
//!     return end(spanId, { error: error.message });
//!   }
//! }
//!
//! // Point-in-time events
//! instant("cache_miss", { key: "user:123" });
//!
//! // Periodic export (every 60 seconds)
//! setInterval(() => {
//!   const spans = flush();
//!   if (spans.length > 0) {
//!     console.log(`Exporting ${spans.length} spans`);
//!     // Send to backend, write to file, etc.
//!   }
//! }, 60000);
//! ```
//!
//! ## Implementation Details
//!
//! ### Span ID Generation
//!
//! Span IDs are generated using a wrapping monotonic counter:
//! ```ignore
//! let id = trace_state.next_id.wrapping_add(1).max(1);
//! trace_state.next_id = id;
//! ```
//!
//! - IDs start at 1 (never 0)
//! - Wraps to 1 on overflow (not 0)
//! - Monotonically increasing within a runtime instance
//! - Not globally unique (reset on application restart)
//!
//! ### Duration Measurement
//!
//! Duration is calculated using `Instant::elapsed()`:
//! - `Instant::now()` captured on `start()`
//! - `Instant::elapsed()` called on `end()` to get duration
//! - Monotonic (unaffected by system clock changes)
//! - High precision (typically nanosecond resolution)
//!
//! ### Wall-Clock Timestamps
//!
//! `started_at` uses `SystemTime` for export compatibility:
//! ```ignore
//! SystemTime::now()
//!     .duration_since(SystemTime::UNIX_EPOCH)
//!     .unwrap_or(Duration::ZERO)
//!     .as_millis()
//! ```
//!
//! - Milliseconds since January 1, 1970 00:00:00 UTC
//! - Compatible with JavaScript Date, database timestamps, etc.
//! - May jump backward if system clock adjusted
//!
//! ### Memory Management
//!
//! - **Active Spans**: Stored in `HashMap` until `end()` called
//!   - Memory grows unbounded if `end()` never called
//!   - Recommend try/finally to ensure `end()` always called
//! - **Finished Spans**: Stored in `Vec` until `flush()` called
//!   - Memory grows until explicitly flushed
//!   - Recommend periodic `flush()` to prevent unbounded growth
//!
//! ### Instant Events
//!
//! `op_trace_instant` creates a `SpanRecord` with `duration_ms: 0`:
//! - Generates new ID but never adds to `active` HashMap
//! - Directly appends to `finished` Vec
//! - Useful for marking checkpoints, state changes, discrete events
//!
//! ## Error Handling
//!
//! Only one error type:
//! ```ignore
//! #[derive(Debug, Error, JsError)]
//! pub enum TraceError {
//!     #[error("Span not found")]
//!     #[class(generic)]
//!     SpanNotFound,
//! }
//! ```
//!
//! Thrown when `end()` called with invalid or already-finished span ID.
//!
//! ## Helper Methods
//!
//! `TraceState` provides introspection methods for debugging:
//! - `active_count()` - Number of currently active spans
//! - `finished_count()` - Number of finished spans in buffer
//! - `finished_spans()` - Read-only view of finished spans (no drain)
//! - `active_spans()` - List of (id, name) tuples for active spans
//!
//! These are not exposed to TypeScript but useful for Rust-side debugging.
//!
//! ## Platform Support
//!
//! | Platform | Support | Notes |
//! |----------|---------|-------|
//! | macOS | ✓ | Full support |
//! | Windows | ✓ | Full support |
//! | Linux | ✓ | Full support |
//! | FreeBSD | ✓ | Full support (via std::time) |
//! | OpenBSD | ✓ | Full support (via std::time) |
//! | NetBSD | ✓ | Full support (via std::time) |
//!
//! Uses only `std::time` primitives - no platform-specific code.
//!
//! ## Dependencies
//!
//! | Dependency | Version | Purpose |
//! |------------|---------|---------|
//! | `deno_core` | workspace | Op definitions, Extension, OpState |
//! | `serde` | workspace | Serialization derive macros |
//! | `serde_json` | workspace | JSON Value for attributes/results |
//! | `thiserror` | workspace | Error type derivation |
//! | `deno_error` | workspace | JsError derive for TraceError |
//! | `forge-weld` | workspace | Build-time code generation |
//! | `forge-weld-macro` | workspace | `#[weld_op]`, `#[weld_struct]` macros |
//! | `linkme` | workspace | Compile-time symbol collection |
//!
//! ## Testing
//!
//! ```bash
//! # Run extension tests
//! cargo test -p ext_trace
//!
//! # Run with debug logging
//! RUST_LOG=ext_trace=trace cargo test -p ext_trace
//! ```
//!
//! ## Common Pitfalls
//!
//! 1. **Forgetting to call `end()`**
//!    - Active spans accumulate in memory
//!    - Use try/finally to ensure `end()` always called
//!
//! 2. **Forgetting to call `flush()`**
//!    - Finished spans accumulate indefinitely
//!    - Set up periodic `setInterval(() => flush(), 60000)`
//!
//! 3. **Reusing span IDs**
//!    - Once ended, a span ID cannot be reused
//!    - Calling `end()` twice with same ID throws `SpanNotFound`
//!
//! 4. **Assuming zero-overhead**
//!    - Every `start()` allocates memory (HashMap entry)
//!    - Every `end()` allocates memory (Vec entry)
//!    - For critical hot paths, consider sampling at application level

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

#[weld_struct]
#[derive(Serialize)]
struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

#[derive(Debug, Error, JsError)]
pub enum TraceError {
    #[error("Span not found")]
    #[class(generic)]
    SpanNotFound,
}

#[derive(Debug)]
struct ActiveSpan {
    id: u64,
    name: String,
    started: Instant,
    wall_clock: SystemTime,
    attributes: Option<Value>,
}

#[weld_struct]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpanRecord {
    pub id: u64,
    pub name: String,
    pub started_at: u128,
    pub duration_ms: f64,
    pub attributes: Option<Value>,
    pub result: Option<Value>,
}

/// Trace state stored in OpState - tracks active spans and completed span records
#[derive(Default)]
pub struct TraceState {
    next_id: u64,
    active: HashMap<u64, ActiveSpan>,
    finished: Vec<SpanRecord>,
}

impl TraceState {
    /// Get the count of currently active spans
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Get the count of finished spans in the buffer
    pub fn finished_count(&self) -> usize {
        self.finished.len()
    }

    /// Clear all active and finished spans, returning the total number removed.
    pub fn clear(&mut self) -> u32 {
        let removed = (self.active.len() + self.finished.len()) as u32;
        self.active.clear();
        self.finished.clear();
        removed
    }

    /// Get a read-only snapshot of finished spans (without clearing)
    pub fn finished_spans(&self) -> &[SpanRecord] {
        &self.finished
    }

    /// Get active span names and IDs
    pub fn active_spans(&self) -> Vec<(u64, String)> {
        self.active
            .iter()
            .map(|(id, span)| (*id, span.name.clone()))
            .collect()
    }
}

#[weld_op]
#[op2]
#[serde]
fn op_trace_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_trace",
        version: env!("CARGO_PKG_VERSION"),
        status: "ready",
    }
}

/// Start a span and return its id.
#[weld_op]
#[op2]
#[bigint]
fn op_trace_start(
    state: &mut OpState,
    #[string] name: String,
    #[serde] attributes: Option<Value>,
) -> u64 {
    let trace_state = state.borrow_mut::<TraceState>();
    let id = trace_state.next_id.wrapping_add(1).max(1);

    trace_state.active.insert(
        id,
        ActiveSpan {
            id,
            name,
            started: Instant::now(),
            wall_clock: SystemTime::now(),
            attributes,
        },
    );

    trace_state.next_id = id;
    id
}

/// End a span and return the completed record.
#[weld_op]
#[op2]
#[serde]
fn op_trace_end(
    state: &mut OpState,
    #[bigint] id: u64,
    #[serde] result: Option<Value>,
) -> Result<SpanRecord, TraceError> {
    let trace_state = state.borrow_mut::<TraceState>();
    let span = trace_state
        .active
        .remove(&id)
        .ok_or(TraceError::SpanNotFound)?;
    let duration = span.started.elapsed_or_zero();

    let record = SpanRecord {
        id: span.id,
        name: span.name,
        started_at: span
            .wall_clock
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis(),
        duration_ms: duration.as_secs_f64() * 1000.0,
        attributes: span.attributes,
        result,
    };

    trace_state.finished.push(record.clone());
    Ok(record)
}

/// Record a point-in-time event.
#[weld_op]
#[op2]
#[serde]
fn op_trace_instant(
    state: &mut OpState,
    #[string] name: String,
    #[serde] attributes: Option<Value>,
) -> SpanRecord {
    let trace_state = state.borrow_mut::<TraceState>();
    let id = trace_state.next_id.wrapping_add(1).max(1);
    trace_state.next_id = id;

    let record = SpanRecord {
        id,
        name,
        started_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis(),
        duration_ms: 0.0,
        attributes,
        result: None,
    };

    trace_state.finished.push(record.clone());
    record
}

/// Return all finished spans and clear the buffer.
#[weld_op]
#[op2]
#[serde]
fn op_trace_flush(state: &mut OpState) -> Vec<SpanRecord> {
    let trace_state = state.borrow_mut::<TraceState>();
    let result = trace_state.finished.clone();
    trace_state.finished.clear();
    result
}

/// Clear all spans (active and finished). Returns the number of spans removed.
#[weld_op]
#[op2(fast)]
fn op_trace_clear(state: &mut OpState) -> u32 {
    state.borrow_mut::<TraceState>().clear()
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn trace_extension() -> Extension {
    runtime_trace::ext()
}

/// Initialize trace state in OpState - must be called after creating JsRuntime
pub fn init_trace_state(op_state: &mut OpState) {
    op_state.put::<TraceState>(TraceState::default());
}

trait InstantExt {
    fn elapsed_or_zero(&self) -> Duration;
}

impl InstantExt for Instant {
    fn elapsed_or_zero(&self) -> Duration {
        self.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Instant, SystemTime};

    fn active_span(id: u64) -> ActiveSpan {
        ActiveSpan {
            id,
            name: format!("span-{id}"),
            started: Instant::now(),
            wall_clock: SystemTime::now(),
            attributes: None,
        }
    }

    fn finished_span(id: u64) -> SpanRecord {
        SpanRecord {
            id,
            name: format!("done-{id}"),
            started_at: 0,
            duration_ms: 1.0,
            attributes: None,
            result: None,
        }
    }

    // H4 regression: clear() must actually empty both span collections and
    // report the real number removed (the CDP clear path previously returned a
    // hardcoded 0 without touching any state).
    #[test]
    fn clear_removes_all_spans_and_returns_count() {
        let mut state = TraceState::default();
        state.active.insert(1, active_span(1));
        state.active.insert(2, active_span(2));
        state.finished.push(finished_span(3));
        assert_eq!(state.active_count(), 2);
        assert_eq!(state.finished_count(), 1);

        let removed = state.clear();

        assert_eq!(removed, 3, "clear must report every removed span");
        assert_eq!(state.active_count(), 0);
        assert_eq!(state.finished_count(), 0);
    }

    #[test]
    fn clear_on_empty_state_returns_zero() {
        let mut state = TraceState::default();
        assert_eq!(state.clear(), 0);
    }
}
