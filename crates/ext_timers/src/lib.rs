//! runtime:timers extension - setTimeout/setInterval support for Forge apps
//!
//! Provides timer functionality using tokio timers.

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tracing::debug;

static TIMER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

// ============================================================================
// Error Types
// ============================================================================

/// Custom error type for timer operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum TimerError {
    #[error("Timer error: {0}")]
    #[class(generic)]
    Generic(String),

    #[error("Timer not found: {0}")]
    #[class(generic)]
    NotFound(String),
}

impl TimerError {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn not_found(id: u64) -> Self {
        Self::NotFound(format!("Timer {} not found", id))
    }
}

// ============================================================================
// State Types
// ============================================================================

/// Timer info stored in op state
#[derive(Debug)]
pub struct TimerInfo {
    pub cancel_tx: mpsc::Sender<()>,
}

/// State for managing timers
#[derive(Default)]
pub struct TimerState {
    pub timers: HashMap<u64, TimerInfo>,
}

// ============================================================================
// Operations
// ============================================================================

/// Result of creating a timer
#[weld_struct]
#[derive(Debug, Serialize, Deserialize)]
pub struct TimerResult {
    pub id: u64,
}

/// Options for creating a timer
#[derive(Debug, Deserialize)]
pub struct TimerOptions {
    pub delay_ms: u64,
    pub repeat: bool,
}

/// Create a timer and return its ID
#[weld_op]
#[op2]
#[serde]
pub fn op_host_timer_create(
    state: &mut OpState,
    #[serde] options: TimerOptions,
) -> TimerResult {
    let timer_id = TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    debug!(timer_id = timer_id, delay_ms = options.delay_ms, repeat = options.repeat, "timer.create");

    // Create a channel to cancel the timer
    let (cancel_tx, _cancel_rx) = mpsc::channel::<()>(1);

    // Store timer info
    let timer_state = state.borrow_mut::<TimerState>();
    timer_state.timers.insert(timer_id, TimerInfo { cancel_tx });

    TimerResult { id: timer_id }
}

/// Cancel a timer
#[weld_op]
#[op2(fast)]
pub fn op_host_timer_cancel(state: &mut OpState, #[bigint] timer_id: u64) -> bool {
    debug!(timer_id = timer_id, "timer.cancel");

    let timer_state = state.borrow_mut::<TimerState>();
    if let Some(info) = timer_state.timers.remove(&timer_id) {
        // Send cancel signal (ignore if receiver dropped)
        let _ = info.cancel_tx.try_send(());
        true
    } else {
        false
    }
}

/// Sleep for specified milliseconds (async)
/// Returns true if completed, false if cancelled
#[weld_op(async)]
#[op2(async)]
pub async fn op_host_timer_sleep(
    state: Rc<RefCell<OpState>>,
    #[bigint] timer_id: u64,
    #[bigint] delay_ms: u64,
) -> bool {
    use tokio::time::{sleep, Duration};

    debug!(timer_id = timer_id, delay_ms = delay_ms, "timer.sleep");

    // Get cancel receiver
    let cancel_rx = {
        let mut state = state.borrow_mut();
        let timer_state = state.borrow_mut::<TimerState>();
        if let Some(_info) = timer_state.timers.get(&timer_id) {
            // Create a new channel for this sleep
            let (new_tx, rx) = mpsc::channel::<()>(1);
            // Replace the sender
            if let Some(info) = timer_state.timers.get_mut(&timer_id) {
                info.cancel_tx = new_tx;
            }
            Some(rx)
        } else {
            None
        }
    };

    if let Some(mut cancel_rx) = cancel_rx {
        tokio::select! {
            _ = sleep(Duration::from_millis(delay_ms)) => {
                debug!(timer_id = timer_id, "timer.sleep completed");
                true // Timer completed
            }
            _ = cancel_rx.recv() => {
                debug!(timer_id = timer_id, "timer.sleep cancelled");
                false // Timer cancelled
            }
        }
    } else {
        // Timer was already cancelled/removed
        debug!(timer_id = timer_id, "timer.sleep - timer not found");
        false
    }
}

/// Check if a timer exists
#[weld_op]
#[op2(fast)]
pub fn op_host_timer_exists(state: &mut OpState, #[bigint] timer_id: u64) -> bool {
    let timer_state = state.borrow::<TimerState>();
    timer_state.timers.contains_key(&timer_id)
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize timer state in OpState
pub fn init_timer_state(op_state: &mut OpState) {
    op_state.put(TimerState::default());
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn timers_extension() -> Extension {
    runtime_timers::ext()
}
