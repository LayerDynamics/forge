use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

// ============================================================================
// Error Types (7000+ range - ext_window uses 6000)
// ============================================================================

/// Error codes for IPC operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IpcErrorCode {
    /// Channel send error
    ChannelSend = 7000,
    /// Channel receive error
    ChannelRecv = 7001,
    /// Permission denied by capability system
    PermissionDenied = 7002,
    /// Window not found
    WindowNotFound = 7003,
}

/// Custom error type for IPC operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum IpcError {
    #[error("[{code}] Channel send error: {message}")]
    #[class(generic)]
    ChannelSend { code: u32, message: String },

    #[error("[{code}] Channel receive error: {message}")]
    #[class(generic)]
    ChannelRecv { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Window not found: {window_id}")]
    #[class(generic)]
    WindowNotFound { code: u32, window_id: String },
}

impl IpcError {
    pub fn channel_send(message: impl Into<String>) -> Self {
        Self::ChannelSend {
            code: IpcErrorCode::ChannelSend as u32,
            message: message.into(),
        }
    }

    pub fn channel_recv(message: impl Into<String>) -> Self {
        Self::ChannelRecv {
            code: IpcErrorCode::ChannelRecv as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: IpcErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn window_not_found(window_id: impl Into<String>) -> Self {
        Self::WindowNotFound {
            code: IpcErrorCode::WindowNotFound as u32,
            window_id: window_id.into(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Event sent from renderer (WebView) to Deno
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEvent {
    pub window_id: String,
    pub channel: String,
    pub payload: serde_json::Value,
    /// Event type for window events: "close", "focus", "blur", "resize", "move"
    pub event_type: Option<String>,
}

/// Command sent from Deno to renderer (WebView)
#[derive(Debug, Clone)]
pub enum ToRendererCmd {
    Send {
        window_id: String,
        channel: String,
        payload: serde_json::Value,
    },
}

// ============================================================================
// State Management
// ============================================================================

/// State stored in OpState for IPC operations
pub struct IpcState {
    pub to_renderer_tx: mpsc::Sender<ToRendererCmd>,
    pub to_deno_rx: Rc<RefCell<Option<mpsc::Receiver<IpcEvent>>>>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for IPC operations
pub trait IpcCapabilityChecker: Send + Sync {
    /// Check if a channel is allowed for IPC communication
    fn check_channel(
        &self,
        channel: &str,
        window_channels: Option<&[String]>,
    ) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveIpcChecker;

impl IpcCapabilityChecker for PermissiveIpcChecker {
    fn check_channel(
        &self,
        _channel: &str,
        _window_channels: Option<&[String]>,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store the capability checker in OpState
pub struct IpcCapabilities {
    pub checker: Arc<dyn IpcCapabilityChecker>,
}

impl Default for IpcCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveIpcChecker),
        }
    }
}

// ============================================================================
// IPC Operations
// ============================================================================

/// Helper to check IPC channel capability
fn check_ipc_capability(state: &OpState, channel: &str) -> Result<(), IpcError> {
    // Try to get capabilities - if not set, allow all (dev mode behavior)
    if let Some(caps) = state.try_borrow::<IpcCapabilities>() {
        // For outgoing messages from Deno, we use the global channel check (None for window_channels)
        // The host will perform additional per-window checks when delivering the message
        caps.checker
            .check_channel(channel, None)
            .map_err(IpcError::permission_denied)?;
    }
    Ok(())
}

/// Send a message to a specific window's renderer
///
/// Note: Channel permissions are enforced symmetrically - both outgoing messages from Deno
/// and incoming messages from the renderer are subject to the capability checker. The host
/// may perform additional per-window channel filtering when delivering the message.
#[weld_op(async)]
#[op2(async)]
async fn op_ipc_send(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    #[string] channel: String,
    #[serde] payload: serde_json::Value,
) -> Result<(), IpcError> {
    // Check channel capability before sending
    {
        let s = state.borrow();
        check_ipc_capability(&s, &channel)?;
    }

    let to_renderer_tx = {
        let s = state.borrow();
        let ipc_state = s.borrow::<IpcState>();
        ipc_state.to_renderer_tx.clone()
    };

    tracing::debug!(
        window_id = %window_id,
        channel = %channel,
        "Sending IPC message to renderer"
    );

    to_renderer_tx
        .send(ToRendererCmd::Send {
            window_id,
            channel,
            payload,
        })
        .await
        .map_err(|e| IpcError::channel_send(e.to_string()))?;

    Ok(())
}

/// Receive the next event from any window (blocking)
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_ipc_recv(state: Rc<RefCell<OpState>>) -> Result<Option<serde_json::Value>, IpcError> {
    let maybe_rx = {
        let s = state.borrow();
        let ipc_state = s.borrow::<IpcState>();
        let result = ipc_state.to_deno_rx.borrow_mut().take();
        result
    };

    if let Some(mut rx) = maybe_rx {
        let result = rx.recv().await;

        // Put the receiver back
        {
            let s = state.borrow();
            let ipc_state = s.borrow::<IpcState>();
            *ipc_state.to_deno_rx.borrow_mut() = Some(rx);
        }

        match result {
            Some(event) => {
                tracing::debug!(
                    window_id = %event.window_id,
                    channel = %event.channel,
                    "Received IPC event from renderer"
                );

                let mut json = serde_json::json!({
                    "windowId": event.window_id,
                    "channel": event.channel,
                    "payload": event.payload,
                });
                // Include event_type if present (for window system events)
                if let Some(ref event_type) = event.event_type {
                    json["type"] = serde_json::json!(event_type);
                }
                Ok(Some(json))
            }
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Build the IPC extension
pub fn ipc_extension() -> Extension {
    runtime_ipc::ext()
}

/// Initialize IPC state in OpState - must be called after creating JsRuntime
pub fn init_ipc_state(
    op_state: &mut OpState,
    to_renderer_tx: mpsc::Sender<ToRendererCmd>,
    to_deno_rx: mpsc::Receiver<IpcEvent>,
) {
    op_state.put(IpcState {
        to_renderer_tx,
        to_deno_rx: Rc::new(RefCell::new(Some(to_deno_rx))),
    });
}

/// Initialize IPC capabilities in OpState
pub fn init_ipc_capabilities(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn IpcCapabilityChecker>>,
) {
    if let Some(caps) = capabilities {
        op_state.put(IpcCapabilities { checker: caps });
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(IpcErrorCode::ChannelSend as u32, 7000);
        assert_eq!(IpcErrorCode::ChannelRecv as u32, 7001);
        assert_eq!(IpcErrorCode::PermissionDenied as u32, 7002);
        assert_eq!(IpcErrorCode::WindowNotFound as u32, 7003);
    }

    #[test]
    fn test_error_display() {
        let err = IpcError::channel_send("test error");
        assert!(err.to_string().contains("7000"));
        assert!(err.to_string().contains("test error"));

        let err = IpcError::window_not_found("win-1");
        assert!(err.to_string().contains("7003"));
        assert!(err.to_string().contains("win-1"));

        let err = IpcError::permission_denied("channel blocked");
        assert!(err.to_string().contains("7002"));
        assert!(err.to_string().contains("channel blocked"));
    }

    #[test]
    fn test_ipc_event_serialization() {
        let event = IpcEvent {
            window_id: "win-1".to_string(),
            channel: "test-channel".to_string(),
            payload: serde_json::json!({"key": "value"}),
            event_type: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("win-1"));
        assert!(json.contains("test-channel"));

        let parsed: IpcEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.window_id, "win-1");
        assert_eq!(parsed.channel, "test-channel");
    }

    #[test]
    fn test_ipc_event_with_event_type() {
        let event = IpcEvent {
            window_id: "win-1".to_string(),
            channel: "window".to_string(),
            payload: serde_json::Value::Null,
            event_type: Some("close".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("close"));

        let parsed: IpcEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type, Some("close".to_string()));
    }
}
