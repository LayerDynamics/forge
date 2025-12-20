//! runtime:devtools extension - Developer tools control for Forge runtime
//!
//! Provides a simple API for opening, closing, and checking the state of browser
//! DevTools for WebView windows. Built as a thin wrapper around the [`ext_window`]
//! runtime, this extension offers programmatic control over the DevTools panel that
//! developers use for debugging and inspecting web content.
//!
//! **Runtime Module:** `runtime:devtools`
//!
//! ## Overview
//!
//! `ext_devtools` is a lightweight wrapper around the window management system that
//! provides dedicated DevTools control operations. It translates DevTools-specific
//! commands into [`ext_window::WindowCmd`] messages sent through the ext_window
//! command channel.
//!
//! This design provides:
//! - **Simplified API**: Three focused operations (open, close, isOpen)
//! - **Centralized Management**: All window operations through ext_window
//! - **Type Safety**: Boolean return values with structured error handling
//! - **Permission Integration**: Uses ext_window's capability-based security
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ TypeScript Application (runtime:devtools)                    │
//! │  - open(), close(), isOpen()                                 │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ Deno Ops (op_devtools_*)
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ext_devtools Operations                                      │
//! │  - Convert to WindowCmd messages                             │
//! │  - Check permissions via WindowCapabilities                  │
//! │  - Forward to ext_window command channel                     │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ WindowCmd::{OpenDevTools, CloseDevTools, IsDevToolsOpen}
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ext_window (WindowRuntimeState)                              │
//! │  - Process window commands                                   │
//! │  - Manage wry/tao window instances                           │
//! │  - Control DevTools panel state                              │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ wry DevTools API
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ Native WebView DevTools (WebKit/WebView2/WebKitGTK)          │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Operations
//!
//! The extension provides 3 operations, each mapped to a window command:
//!
//! | Operation | Window Command | Purpose |
//! |-----------|---------------|---------|
//! | `op_devtools_open` | `WindowCmd::OpenDevTools` | Open DevTools panel |
//! | `op_devtools_close` | `WindowCmd::CloseDevTools` | Close DevTools panel |
//! | `op_devtools_is_open` | `WindowCmd::IsDevToolsOpen` | Check DevTools state |
//!
//! ## Error Handling
//!
//! All operations use the [`DevtoolsError`] enum with two error codes:
//!
//! | Code | Error | Description |
//! |------|-------|-------------|
//! | 9100 | Generic | General DevTools operation failure |
//! | 9101 | PermissionDenied | Window management permission denied |
//!
//! Errors are automatically converted to JavaScript exceptions via the `#[derive(JsError)]`
//! macro from `deno_error`.
//!
//! ## Permission Model
//!
//! DevTools operations require window management permissions checked via
//! [`ext_window::WindowCapabilities`]. The permission is defined in the app's
//! `manifest.app.toml`:
//!
//! ```toml
//! [permissions.ui]
//! windows = true
//! ```
//!
//! Operations fail with error code 9101 if permissions are not granted.
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { open, close, isOpen } from "runtime:devtools";
//! import { webviewNew } from "runtime:webview";
//!
//! // Create window with DevTools available
//! const window = await webviewNew({
//!   title: "Debug Window",
//!   url: "app://index.html",
//!   width: 1200,
//!   height: 800,
//!   resizable: true,
//!   debug: true,  // DevTools available
//!   frameless: false
//! });
//!
//! // Open DevTools programmatically
//! await open(window.id);
//!
//! // Check state
//! const devToolsOpen = await isOpen(window.id);
//! console.log("DevTools open:", devToolsOpen); // true
//!
//! // Close when done
//! await close(window.id);
//! ```
//!
//! ## Implementation Details
//!
//! ### Opening DevTools
//!
//! `op_devtools_open` sends `WindowCmd::OpenDevTools` through the window command channel:
//!
//! 1. Check permissions via `check_window_caps()`
//! 2. Send `WindowCmd::OpenDevTools` to ext_window command channel
//! 3. Await response confirmation
//! 4. Return `true` on success
//!
//! ### Closing DevTools
//!
//! `op_devtools_close` sends `WindowCmd::CloseDevTools`:
//!
//! 1. Check permissions
//! 2. Send `WindowCmd::CloseDevTools` with window ID
//! 3. Await response confirmation
//! 4. Return `true` on success
//!
//! ### Checking State
//!
//! `op_devtools_is_open` queries the DevTools state:
//!
//! 1. Check permissions
//! 2. Send `WindowCmd::IsDevToolsOpen` with window ID
//! 3. Await boolean response from window manager
//! 4. Return DevTools open state (defaults to `false` if query fails)
//!
//! ## Platform Support
//!
//! | Platform | DevTools Backend | Status |
//! |----------|-----------------|--------|
//! | macOS | WebKit Inspector | ✅ Full support |
//! | Windows | Edge DevTools (F12) | ✅ Full support |
//! | Linux | WebKit Inspector | ✅ Full support |
//!
//! Platform-specific DevTools behavior is handled by the underlying `wry` crate.
//!
//! ## Dependencies
//!
//! | Dependency | Version | Purpose |
//! |-----------|---------|---------|
//! | `deno_core` | 0.373 | Op definitions and runtime integration |
//! | `ext_window` | 0.1.0-alpha.1 | Window management and command channel |
//! | `tokio` | 1.x | Async oneshot channels for command responses |
//! | `thiserror` | 2.x | Error type definitions |
//! | `deno_error` | 0.x | JavaScript error conversion |
//! | `forge-weld-macro` | 0.1 | TypeScript binding generation |
//!
//! ## Testing
//!
//! ```bash
//! # Run all tests
//! cargo test -p ext_devtools
//!
//! # Run with output
//! cargo test -p ext_devtools -- --nocapture
//!
//! # With debug logging
//! RUST_LOG=ext_devtools=debug cargo test -p ext_devtools -- --nocapture
//! ```
//!
//! ## See Also
//!
//! - [`ext_window`] - Window management extension
//! - [`ext_webview`] - WebView creation extension
//! - [wry documentation](https://docs.rs/wry) - WebView rendering library

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
