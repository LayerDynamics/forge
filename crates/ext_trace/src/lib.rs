//! Lightweight tracing extension for app-level spans and events.

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
struct SpanRecord {
    id: u64,
    name: String,
    started_at: u128,
    duration_ms: f64,
    attributes: Option<Value>,
    result: Option<Value>,
}

#[derive(Default)]
struct TraceState {
    next_id: u64,
    active: HashMap<u64, ActiveSpan>,
    finished: Vec<SpanRecord>,
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
    let span = trace_state.active.remove(&id).ok_or(TraceError::SpanNotFound)?;
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
