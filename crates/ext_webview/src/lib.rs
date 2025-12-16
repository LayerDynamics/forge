//! runtime:webview extension
//!
//! Lightweight wrapper that reuses the runtime:window runtime to create and control
//! webviews from Deno. Provides a small API similar to the reference Deno plugin
//! shown in the request (new/exit/eval/title/fullscreen/background color).

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
