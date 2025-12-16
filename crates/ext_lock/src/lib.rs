//! Named async locks for coordinating work across host ops.

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tokio::time::{timeout, Duration};
use thiserror::Error;

#[derive(Debug, Error, JsError)]
pub enum LockError {
    #[error("Lock acquisition timed out")]
    #[class(generic)]
    Timeout,
}

/// Per-lock entry with an async mutex and current holder token.
struct LockEntry {
    mutex: Arc<AsyncMutex<()>>,
    holder: Mutex<Option<(u64, OwnedMutexGuard<()>)>>,
}

impl LockEntry {
    fn new() -> Self {
        Self {
            mutex: Arc::new(AsyncMutex::new(())),
            holder: Mutex::new(None),
        }
    }
}

/// Shared lock state stored in OpState.
#[derive(Default)]
struct LockState {
    locks: HashMap<String, Arc<LockEntry>>,
    next_token: u64,
}

#[weld_struct]
#[derive(Serialize)]
struct LockInfo {
    name: String,
    locked: bool,
}

/// Acquire a named lock, waiting up to `timeout_ms` if provided.
#[weld_op(async)]
#[op2(async)]
#[bigint]
async fn op_lock_acquire(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
    #[serde] timeout_ms: Option<u64>,
) -> Result<u64, LockError> {
    let entry = {
        let mut s = state.borrow_mut();
        let lock_state = s.borrow_mut::<LockState>();
        lock_state
            .locks
            .entry(name.clone())
            .or_insert_with(|| Arc::new(LockEntry::new()))
            .clone()
    };

    let lock_future = entry.mutex.clone().lock_owned();
    let guard = match timeout_ms {
        Some(ms) => {
            let dur = Duration::from_millis(ms);
            timeout(dur, lock_future)
                .await
                .map_err(|_| LockError::Timeout)?
        }
        None => lock_future.await,
    };

    let token = {
        let mut s = state.borrow_mut();
        let lock_state = s.borrow_mut::<LockState>();
        lock_state.next_token = lock_state.next_token.wrapping_add(1).max(1);
        lock_state.next_token
    };

    {
        let mut holder = entry.holder.lock().unwrap();
        *holder = Some((token, guard));
    }

    Ok(token)
}

/// Try to acquire a lock without waiting.
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_lock_try(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
) -> Result<Option<u64>, LockError> {
    let entry = {
        let mut s = state.borrow_mut();
        let lock_state = s.borrow_mut::<LockState>();
        lock_state
            .locks
            .entry(name.clone())
            .or_insert_with(|| Arc::new(LockEntry::new()))
            .clone()
    };

    let guard = match entry.mutex.clone().try_lock_owned() {
        Ok(g) => g,
        Err(_) => return Ok(None),
    };

    let token = {
        let mut s = state.borrow_mut();
        let lock_state = s.borrow_mut::<LockState>();
        lock_state.next_token = lock_state.next_token.wrapping_add(1).max(1);
        lock_state.next_token
    };

    {
        let mut holder = entry.holder.lock().unwrap();
        *holder = Some((token, guard));
    }

    Ok(Some(token))
}

/// Release a lock if the token matches.
#[weld_op]
#[op2(fast)]
fn op_lock_release(
    state: &mut OpState,
    #[string] name: String,
    #[bigint] token: u64,
) -> Result<bool, LockError> {
    let entry = {
        let lock_state = state.borrow_mut::<LockState>();
        lock_state.locks.get(&name).cloned()
    };

    let Some(entry) = entry else {
        return Ok(false);
    };

    let mut holder = entry.holder.lock().unwrap();
    if let Some((held_token, guard)) = holder.take() {
        if held_token == token {
            drop(guard);
            return Ok(true);
        }
        *holder = Some((held_token, guard));
    }

    Ok(false)
}

/// List locks currently known to the runtime.
#[weld_op]
#[op2]
#[serde]
fn op_lock_list(state: &mut OpState) -> Vec<LockInfo> {
    let lock_state = state.borrow_mut::<LockState>();
    lock_state
        .locks
        .iter()
        .map(|(name, entry)| LockInfo {
            name: name.clone(),
            locked: entry.holder.lock().unwrap().is_some(),
        })
        .collect()
}

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn init_lock_state(op_state: &mut OpState) {
    op_state.put::<LockState>(LockState::default());
}

pub fn lock_extension() -> Extension {
    runtime_lock::ext()
}
