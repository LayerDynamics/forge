//! OS signal subscription extension for Forge host runtime.
//! Provides async subscriptions to POSIX signals for apps.

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Error, JsError)]
pub enum SignalsError {
    #[error("Signals are not supported on this platform")]
    #[class(generic)]
    UnsupportedPlatform,

    #[error("Unknown or unsupported signal: {0}")]
    #[class(generic)]
    InvalidSignal(String),

    #[error("Subscription not found")]
    #[class(generic)]
    SubscriptionNotFound,

    #[error("Failed to initialize signal handler: {0}")]
    #[class(generic)]
    SignalInit(String),
}

#[weld_struct]
#[derive(Debug, Serialize)]
struct SignalEvent {
    signal: String,
}

struct SignalSubscription {
    receiver: Option<mpsc::Receiver<SignalEvent>>,
    tasks: Vec<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

#[derive(Default)]
struct SignalsState {
    next_id: u64,
    subscriptions: HashMap<u64, SignalSubscription>,
}

#[cfg(unix)]
fn parse_signals(
    signals: &[String],
) -> Result<Vec<(String, tokio::signal::unix::SignalKind)>, SignalsError> {
    use tokio::signal::unix::SignalKind;

    if signals.is_empty() {
        return Err(SignalsError::InvalidSignal("<empty>".into()));
    }

    signals
        .iter()
        .map(|s| {
            let upper = s.to_ascii_uppercase();
            let kind = match upper.as_str() {
                "SIGINT" => SignalKind::interrupt(),
                "SIGTERM" => SignalKind::terminate(),
                "SIGHUP" => SignalKind::hangup(),
                "SIGQUIT" => SignalKind::quit(),
                "SIGUSR1" => SignalKind::user_defined1(),
                "SIGUSR2" => SignalKind::user_defined2(),
                "SIGALRM" => SignalKind::alarm(),
                "SIGCHLD" => SignalKind::child(),
                "SIGPIPE" => SignalKind::pipe(),
                _ => return Err(SignalsError::InvalidSignal(s.clone())),
            };
            Ok((upper, kind))
        })
        .collect()
}

#[cfg(not(unix))]
fn parse_signals(_signals: &[String]) -> Result<Vec<(String, ())>, SignalsError> {
    Err(SignalsError::UnsupportedPlatform)
}

/// Return the set of signals supported on this platform.
#[weld_op]
#[op2]
#[serde]
fn op_signals_supported() -> Vec<String> {
    #[cfg(unix)]
    {
        vec![
            "SIGINT", "SIGTERM", "SIGHUP", "SIGQUIT", "SIGUSR1", "SIGUSR2", "SIGALRM", "SIGCHLD",
            "SIGPIPE",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[cfg(not(unix))]
    {
        Vec::new()
    }
}

/// Subscribe to a set of OS signals. Returns a subscription id.
#[weld_op(async)]
#[op2(async)]
#[bigint]
async fn op_signals_subscribe(
    state: Rc<RefCell<OpState>>,
    #[serde] signals: Vec<String>,
) -> Result<u64, SignalsError> {
    #[cfg(unix)]
    {
        let parsed = parse_signals(&signals)?;
        let (tx, rx) = mpsc::channel(64);
        let running = Arc::new(AtomicBool::new(true));
        let mut tasks = Vec::with_capacity(parsed.len());

        for (name, kind) in parsed {
            let mut stream = tokio::signal::unix::signal(kind)
                .map_err(|e| SignalsError::SignalInit(e.to_string()))?;
            let tx = tx.clone();
            let running_flag = running.clone();
            let signal_name = name.clone();

            let handle = tokio::spawn(async move {
                while running_flag.load(Ordering::SeqCst) {
                    if stream.recv().await.is_none() {
                        break;
                    }
                    if !running_flag.load(Ordering::SeqCst) {
                        break;
                    }
                    if tx
                        .send(SignalEvent {
                            signal: signal_name.clone(),
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });

            tasks.push(handle);
        }

        let id = {
            let mut s = state.borrow_mut();
            let signals_state = s.borrow_mut::<SignalsState>();
            let next = signals_state.next_id.wrapping_add(1).max(1);
            signals_state.next_id = next;
            signals_state.subscriptions.insert(
                next,
                SignalSubscription {
                    receiver: Some(rx),
                    tasks,
                    running,
                },
            );
            next
        };

        Ok(id)
    }

    #[cfg(not(unix))]
    {
        let _ = parse_signals(&signals)?;
        let _ = state;
        Err(SignalsError::UnsupportedPlatform)
    }
}

/// Receive the next signal event for a subscription.
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_signals_next(
    state: Rc<RefCell<OpState>>,
    #[bigint] id: u64,
) -> Result<Option<SignalEvent>, SignalsError> {
    let mut rx = {
        let mut s = state.borrow_mut();
        let signals_state = s.borrow_mut::<SignalsState>();
        let Some(sub) = signals_state.subscriptions.get_mut(&id) else {
            return Err(SignalsError::SubscriptionNotFound);
        };
        sub.receiver
            .take()
            .ok_or(SignalsError::SubscriptionNotFound)?
    };

    let result = rx.recv().await;

    {
        let mut s = state.borrow_mut();
        let signals_state = s.borrow_mut::<SignalsState>();
        if let Some(sub) = signals_state.subscriptions.get_mut(&id) {
            sub.receiver = Some(rx);
        }
    }

    Ok(result)
}

/// Cancel a subscription and stop listening for its signals.
#[weld_op]
#[op2(fast)]
fn op_signals_unsubscribe(state: &mut OpState, #[bigint] id: u64) -> bool {
    let Some(sub) = state.borrow_mut::<SignalsState>().subscriptions.remove(&id) else {
        return false;
    };

    sub.running.store(false, Ordering::SeqCst);
    for handle in sub.tasks {
        handle.abort();
    }

    true
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn signals_extension() -> Extension {
    runtime_signals::ext()
}

/// Initialize signals state in OpState - must be called after creating JsRuntime
pub fn init_signals_state(op_state: &mut OpState) {
    op_state.put::<SignalsState>(SignalsState::default());
}
