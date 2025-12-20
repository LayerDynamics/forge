//! runtime:webview extension - Lightweight WebView creation and management
//!
//! Provides a simple API for creating and controlling WebView windows, built as a
//! wrapper around the [`ext_window`] runtime. This extension offers a streamlined
//! interface for common WebView operations without requiring direct window management.
//!
//! **Runtime Module:** `runtime:webview`
//!
//! ## Overview
//!
//! `ext_webview` is a lightweight wrapper around the window management system that
//! simplifies WebView creation and control. It translates WebView-specific operations
//! into [`WindowCmd`] messages sent through the ext_window command channel.
//!
//! This design provides:
//! - **Simplified API**: Focus on WebView concerns without window management complexity
//! - **Centralized Event Loop**: All window events handled by Forge's main loop
//! - **Type Safety**: Strongly-typed operations with automatic error handling
//! - **Permission Integration**: Uses ext_window's capability-based security
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ TypeScript Application (runtime:webview)                     │
//! │  - webviewNew(), webviewEval()                               │
//! │  - webviewSetTitle(), webviewSetFullscreen()                 │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ Deno Ops (op_host_webview_*)
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ext_webview Operations                                       │
//! │  - Convert to WindowCmd messages                             │
//! │  - Check permissions via WindowCapabilities                  │
//! │  - Forward to ext_window command channel                     │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ WindowCmd::{Create, Close, EvalJs, ...}
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ext_window (WindowRuntimeState)                              │
//! │  - Process window commands                                   │
//! │  - Manage wry/tao window instances                           │
//! │  - Handle window events                                      │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ wry/tao native window APIs
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ Native Window System (WebKit/WebView2/WebKitGTK)             │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Operations
//!
//! The extension provides 8 operations, each mapped to a window command:
//!
//! | Operation | Window Command | Purpose |
//! |-----------|---------------|---------|
//! | `op_host_webview_new` | `WindowCmd::Create` | Create new WebView window |
//! | `op_host_webview_exit` | `WindowCmd::Close` | Close WebView window |
//! | `op_host_webview_eval` | `WindowCmd::EvalJs` | Execute JavaScript in WebView |
//! | `op_host_webview_set_color` | `WindowCmd::InjectCss` | Set background color |
//! | `op_host_webview_set_title` | `WindowCmd::SetTitle` | Update window title |
//! | `op_host_webview_set_fullscreen` | `WindowCmd::SetFullscreen` | Toggle fullscreen |
//! | `op_host_webview_loop` | *No-op* | Event loop compatibility shim |
//! | `op_host_webview_run` | *No-op* | Run loop compatibility shim |
//!
//! ## Error Handling
//!
//! All operations use the [`WebViewError`] enum with two error codes:
//!
//! | Code | Error | Description |
//! |------|-------|-------------|
//! | 9000 | Generic | General WebView operation failure |
//! | 9001 | PermissionDenied | Window creation permission denied |
//!
//! Errors are automatically converted to JavaScript exceptions via the `#[derive(JsError)]`
//! macro from `deno_error`.
//!
//! ## Permission Model
//!
//! WebView operations require window creation permissions checked via
//! [`WindowCapabilities`]. The permission is defined in the app's `manifest.app.toml`:
//!
//! ```toml
//! [permissions.ui]
//! windows = true
//! ```
//!
//! Operations fail with error code 9001 if permissions are not granted.
//!
//! ## Event Loop Integration
//!
//! Unlike standalone WebView libraries, ext_webview does not provide its own event loop.
//! The `op_host_webview_loop` and `op_host_webview_run` operations are no-ops that exist
//! only for API compatibility with reference WebView plugins.
//!
//! All window and WebView events are handled by Forge's centralized event loop in the
//! runtime, eliminating the need for manual event loop management.
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { webviewNew, webviewEval, webviewExit } from "runtime:webview";
//!
//! // Create WebView window
//! const webview = await webviewNew({
//!   title: "My App",
//!   url: "https://example.com",
//!   width: 800,
//!   height: 600,
//!   resizable: true,
//!   debug: false,
//!   frameless: false
//! });
//!
//! // Execute JavaScript
//! await webviewEval(webview.id, "console.log('Hello from WebView!')");
//!
//! // Close when done
//! await webviewExit(webview.id);
//! ```
//!
//! ## Implementation Details
//!
//! ### Window Creation
//!
//! `op_host_webview_new` converts [`WebViewNewParams`] to [`WindowOpts`] and sends a
//! `WindowCmd::Create` command:
//!
//! 1. Check permissions via `check_window_caps()`
//! 2. Convert parameters (frameless -> !decorations, debug -> devtools)
//! 3. Send `WindowCmd::Create` to ext_window command channel
//! 4. Await response with window ID
//! 5. Return [`WebViewNewResult`] containing window ID
//!
//! ### JavaScript Evaluation
//!
//! `op_host_webview_eval` sends the JavaScript code through the window command channel:
//!
//! 1. Check permissions
//! 2. Send `WindowCmd::EvalJs` with window ID and script
//! 3. Await completion (no return value captured)
//!
//! ### Background Color Setting
//!
//! `op_host_webview_set_color` uses CSS injection to set the body background:
//!
//! 1. Convert RGBA values to CSS rgba() format
//! 2. Generate CSS rule: `body { background-color: rgba(...); }`
//! 3. Send `WindowCmd::InjectCss` to inject the rule
//!
//! ## Platform Support
//!
//! | Platform | WebView Backend | Status |
//! |----------|----------------|--------|
//! | macOS | WebKit (WKWebView) | ✅ Full support |
//! | Windows | WebView2 (Edge) | ✅ Full support |
//! | Linux | WebKitGTK | ✅ Full support |
//!
//! Platform-specific behavior is handled by the underlying `wry` crate.
//!
//! ## Dependencies
//!
//! | Dependency | Version | Purpose |
//! |-----------|---------|---------|
//! | `deno_core` | 0.373 | Op definitions and runtime integration |
//! | `ext_window` | 0.1.0-alpha.1 | Window management and command channel |
//! | `tokio` | 1.x | Async oneshot channels for command responses |
//! | `serde` | 1.x | Serialization framework |
//! | `thiserror` | 2.x | Error type definitions |
//! | `deno_error` | 0.x | JavaScript error conversion |
//! | `forge-weld-macro` | 0.1 | TypeScript binding generation |
//!
//! ## Testing
//!
//! ```bash
//! # Run all tests
//! cargo test -p ext_webview
//!
//! # Run with output
//! cargo test -p ext_webview -- --nocapture
//!
//! # With debug logging
//! RUST_LOG=ext_webview=debug cargo test -p ext_webview -- --nocapture
//! ```
//!
//! ## See Also
//!
//! - [`ext_window`] - Window management extension
//! - [`ext_ipc`] - IPC communication extension
//! - [`ext_devtools`] - Developer tools extension
//! - [wry documentation](https://docs.rs/wry) - WebView rendering library
//! - [tao documentation](https://docs.rs/tao) - Cross-platform window creation

use deno_core::{op2, Extension, OpState};
use ext_window::{WindowCapabilities, WindowCmd, WindowOpts, WindowRuntimeState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;
use tokio::sync::oneshot;

// Include generated extension glue
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Errors surfaced to JS for webview operations
#[derive(Debug, Error, deno_error::JsError)]
pub enum WebViewError {
    #[error("[9000] {0}")]
    #[class(generic)]
    Generic(String),

    #[error("[9001] Permission denied: {0}")]
    #[class(generic)]
    PermissionDenied(String),
}

impl WebViewError {
    fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebViewNewParams {
    title: String,
    url: String,
    width: i32,
    height: i32,
    resizable: bool,
    debug: bool,
    frameless: bool,
}

#[weld_struct]
#[derive(Debug, Serialize)]
struct WebViewNewResult {
    id: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_new(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewNewParams,
) -> Result<WebViewNewResult, WebViewError> {
    check_window_caps(&state)?;

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = oneshot::channel();
    let opts = WindowOpts {
        title: Some(params.title),
        url: Some(params.url),
        width: Some(params.width as u32),
        height: Some(params.height as u32),
        resizable: Some(params.resizable),
        decorations: Some(!params.frameless),
        visible: Some(true),
        devtools: Some(params.debug),
        ..Default::default()
    };

    cmd_tx
        .send(WindowCmd::Create {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?;

    let win_id = respond_rx
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?
        .map_err(WebViewError::generic)?;

    Ok(WebViewNewResult { id: win_id })
}

#[derive(Debug, Deserialize)]
struct WebViewIdParams {
    id: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_exit(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewIdParams,
) -> Result<(), WebViewError> {
    check_window_caps(&state)?;

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = oneshot::channel();
    cmd_tx
        .send(WindowCmd::Close {
            window_id: params.id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct WebViewEvalParams {
    id: String,
    js: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_eval(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewEvalParams,
) -> Result<(), WebViewError> {
    check_window_caps(&state)?;

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = oneshot::channel();
    cmd_tx
        .send(WindowCmd::EvalJs {
            window_id: params.id,
            script: params.js,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?
        .map_err(WebViewError::generic)
}

#[derive(Debug, Deserialize)]
struct WebViewColorParams {
    id: String,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_set_color(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewColorParams,
) -> Result<(), WebViewError> {
    check_window_caps(&state)?;

    // Implement by injecting a CSS rule on body background.
    let rgba = format!(
        "rgba({},{},{},{:.3})",
        params.r,
        params.g,
        params.b,
        (params.a as f32) / 255.0
    );
    let css = format!("body {{ background-color: {}; }}", rgba);

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = oneshot::channel();
    cmd_tx
        .send(WindowCmd::InjectCss {
            window_id: params.id,
            css,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))?
        .map_err(WebViewError::generic)
}

#[derive(Debug, Deserialize)]
struct WebViewTitleParams {
    id: String,
    title: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_set_title(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewTitleParams,
) -> Result<(), WebViewError> {
    check_window_caps(&state)?;

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetTitle {
            window_id: params.id,
            title: params.title,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))
}

#[derive(Debug, Deserialize)]
struct WebViewFullscreenParams {
    id: String,
    fullscreen: bool,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_set_fullscreen(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewFullscreenParams,
) -> Result<(), WebViewError> {
    check_window_caps(&state)?;

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetFullscreen {
            window_id: params.id,
            fullscreen: params.fullscreen,
        })
        .await
        .map_err(|e| WebViewError::generic(e.to_string()))
}

#[derive(Debug, Deserialize)]
struct WebViewLoopParams {
    id: String,
    blocking: i32,
}

#[weld_struct]
#[derive(Debug, Serialize)]
struct WebViewLoopResult {
    code: i32,
}

/// Event loop shim. We already run a central event loop in the host, so this is a no-op
/// that returns success to mirror the reference plugin API.
/// The `id` identifies the webview and `blocking` controls loop behavior (0 = non-blocking).
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_loop(
    state: Rc<RefCell<OpState>>,
    #[serde] params: WebViewLoopParams,
) -> Result<WebViewLoopResult, WebViewError> {
    check_window_caps(&state)?;

    // Validate the webview ID is provided
    if params.id.is_empty() {
        return Err(WebViewError::generic("webview id is required"));
    }

    // Log the loop request for debugging (blocking: 0 = non-blocking, 1 = blocking)
    tracing::debug!(
        webview_id = %params.id,
        blocking = params.blocking,
        "webview loop requested (no-op in centralized event loop)"
    );

    // Return success - the central event loop handles all window events
    Ok(WebViewLoopResult { code: 0 })
}

/// Run loop shim: same rationale as `op_host_webview_loop`.
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_host_webview_run(
    _state: Rc<RefCell<OpState>>,
    #[serde] _params: WebViewIdParams,
) -> Result<(), WebViewError> {
    Ok(())
}

fn check_window_caps(state: &Rc<RefCell<OpState>>) -> Result<(), WebViewError> {
    if let Some(caps) = state.borrow().try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_windows()
            .map_err(WebViewError::permission_denied)
    } else {
        Ok(())
    }
}

/// Build the extension
pub fn webview_extension() -> Extension {
    runtime_webview::ext()
}
