//! runtime:devtools extension
//!
//! Thin wrapper to open/close devtools for an existing window via the ext_window runtime.

use deno_core::{op2, Extension, OpState};
use ext_window::WindowRuntimeState;
use forge_weld_macro::weld_op;
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

// Include generated extension glue
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

#[derive(Debug, Error, deno_error::JsError)]
pub enum DevtoolsError {
    #[error("[9100] {0}")]
    #[class(generic)]
    Generic(String),

    #[error("[9101] Permission denied: {0}")]
    #[class(generic)]
    PermissionDenied(String),
}

impl DevtoolsError {
    fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }
}

#[weld_op(async)]
#[op2(async)]
async fn op_devtools_open(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, DevtoolsError> {
    check_window_caps(&state)?;
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(ext_window::WindowCmd::OpenDevTools {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?
        .map_err(DevtoolsError::generic)?;
    Ok(true)
}

#[weld_op(async)]
#[op2(async)]
async fn op_devtools_close(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, DevtoolsError> {
    check_window_caps(&state)?;
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(ext_window::WindowCmd::CloseDevTools {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?
        .map_err(DevtoolsError::generic)?;
    Ok(true)
}

#[weld_op(async)]
#[op2(async)]
async fn op_devtools_is_open(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, DevtoolsError> {
    check_window_caps(&state)?;
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(ext_window::WindowCmd::IsDevToolsOpen {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?;

    let open = respond_rx
        .await
        .map_err(|e| DevtoolsError::generic(e.to_string()))?
        .unwrap_or(false);

    Ok(open)
}

fn check_window_caps(state: &Rc<RefCell<OpState>>) -> Result<(), DevtoolsError> {
    if let Some(caps) = state
        .borrow()
        .try_borrow::<ext_window::WindowCapabilities>()
    {
        caps.checker
            .check_windows()
            .map_err(DevtoolsError::permission_denied)
    } else {
        Ok(())
    }
}

pub fn devtools_extension() -> Extension {
    runtime_devtools::ext()
}
