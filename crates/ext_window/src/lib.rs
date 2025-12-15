use deno_core::{op2, Extension, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

// Manager module with platform implementation
pub mod manager;
pub use manager::*;

// ============================================================================
// Error Types (6000+ range)
// ============================================================================

/// Error codes for window operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WindowErrorCode {
    /// Generic window error
    Generic = 6000,
    /// Permission denied by capability system
    PermissionDenied = 6001,
    /// Window not found
    WindowNotFound = 6002,
    /// Failed to create window
    CreateFailed = 6003,
    /// Window already closed
    WindowClosed = 6004,
    /// Invalid window options
    InvalidOptions = 6005,
    /// Dialog cancelled by user
    DialogCancelled = 6006,
    /// Menu error
    MenuError = 6007,
    /// Tray error
    TrayError = 6008,
    /// Invalid tray ID
    InvalidTrayId = 6009,
    /// Channel send error
    ChannelSend = 6010,
    /// Channel receive error
    ChannelRecv = 6011,
    /// Invalid position or size
    InvalidGeometry = 6012,
    /// Fullscreen not supported
    FullscreenNotSupported = 6013,
    /// Native handle unavailable
    NativeHandleUnavailable = 6014,
}

/// Custom error type for window operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum WindowError {
    #[error("[{code}] {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Window not found: {window_id}")]
    #[class(generic)]
    WindowNotFound { code: u32, window_id: String },

    #[error("[{code}] Failed to create window: {message}")]
    #[class(generic)]
    CreateFailed { code: u32, message: String },

    #[error("[{code}] Window already closed: {window_id}")]
    #[class(generic)]
    WindowClosed { code: u32, window_id: String },

    #[error("[{code}] Invalid options: {message}")]
    #[class(generic)]
    InvalidOptions { code: u32, message: String },

    #[error("[{code}] Dialog cancelled")]
    #[class(generic)]
    DialogCancelled { code: u32 },

    #[error("[{code}] Menu error: {message}")]
    #[class(generic)]
    MenuError { code: u32, message: String },

    #[error("[{code}] Tray error: {message}")]
    #[class(generic)]
    TrayError { code: u32, message: String },

    #[error("[{code}] Invalid tray ID: {tray_id}")]
    #[class(generic)]
    InvalidTrayId { code: u32, tray_id: String },

    #[error("[{code}] Channel send error: {message}")]
    #[class(generic)]
    ChannelSend { code: u32, message: String },

    #[error("[{code}] Channel receive error: {message}")]
    #[class(generic)]
    ChannelRecv { code: u32, message: String },

    #[error("[{code}] Invalid geometry: {message}")]
    #[class(generic)]
    InvalidGeometry { code: u32, message: String },

    #[error("[{code}] Native handle unavailable")]
    #[class(generic)]
    NativeHandleUnavailable { code: u32 },
}

impl WindowError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: WindowErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: WindowErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn window_not_found(window_id: impl Into<String>) -> Self {
        Self::WindowNotFound {
            code: WindowErrorCode::WindowNotFound as u32,
            window_id: window_id.into(),
        }
    }

    pub fn create_failed(message: impl Into<String>) -> Self {
        Self::CreateFailed {
            code: WindowErrorCode::CreateFailed as u32,
            message: message.into(),
        }
    }

    pub fn channel_send(message: impl Into<String>) -> Self {
        Self::ChannelSend {
            code: WindowErrorCode::ChannelSend as u32,
            message: message.into(),
        }
    }

    pub fn channel_recv(message: impl Into<String>) -> Self {
        Self::ChannelRecv {
            code: WindowErrorCode::ChannelRecv as u32,
            message: message.into(),
        }
    }

    pub fn native_handle_unavailable() -> Self {
        Self::NativeHandleUnavailable {
            code: WindowErrorCode::NativeHandleUnavailable as u32,
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Options for creating a window
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WindowOpts {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub title: Option<String>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    pub visible: Option<bool>,
    pub transparent: Option<bool>,
    pub always_on_top: Option<bool>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    /// Channel allowlist for IPC
    pub channels: Option<Vec<String>>,
}

/// Window position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Window size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Native window handle (platform-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeHandle {
    /// Platform type: "windows", "macos", "linux-x11", "linux-wayland"
    pub platform: String,
    /// Raw handle value (HWND, NSView*, X11 window, etc.)
    pub handle: u64,
}

/// Window system event (sent from Host to Deno)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSystemEvent {
    pub window_id: String,
    pub event_type: String, // "close", "focus", "blur", "resize", "move", "minimize", "maximize", "restore"
    pub payload: serde_json::Value,
}

/// Window state query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub is_visible: bool,
    pub is_focused: bool,
    pub is_fullscreen: bool,
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_resizable: bool,
    pub has_decorations: bool,
    pub is_always_on_top: bool,
}

/// Options for file open dialog
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileDialogOpts {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Option<Vec<FileFilter>>,
    pub multiple: Option<bool>,
    pub directory: Option<bool>,
}

/// File filter for dialogs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Options for message dialog
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageDialogOpts {
    pub title: Option<String>,
    pub message: String,
    pub kind: Option<String>, // "info", "warning", "error"
    pub buttons: Option<Vec<String>>,
}

/// Menu item definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MenuItem {
    pub id: Option<String>,
    pub label: String,
    pub accelerator: Option<String>,
    pub enabled: Option<bool>,
    pub checked: Option<bool>,
    pub submenu: Option<Vec<MenuItem>>,
    #[serde(rename = "type")]
    pub item_type: Option<String>, // "normal", "checkbox", "separator"
}

/// Tray icon definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrayOpts {
    pub icon: Option<String>, // Path to icon file
    pub tooltip: Option<String>,
    pub menu: Option<Vec<MenuItem>>,
}

/// Menu event sent when a menu item is clicked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuEvent {
    /// Source of the menu event: "app" for app menu, "context" for context menu, "tray" for tray menu
    pub menu_id: String,
    /// The id of the menu item that was clicked (from MenuItem.id)
    pub item_id: String,
    /// The label of the menu item
    pub label: String,
}

// ============================================================================
// Command Enum (Deno -> Host communication)
// ============================================================================

/// Commands sent from Deno to Host for window operations
#[derive(Debug)]
pub enum WindowCmd {
    // === Window Lifecycle ===
    Create {
        opts: WindowOpts,
        respond: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    Close {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<bool>,
    },
    Minimize {
        window_id: String,
    },
    Maximize {
        window_id: String,
    },
    Unmaximize {
        window_id: String,
    },
    Restore {
        window_id: String,
    },
    SetFullscreen {
        window_id: String,
        fullscreen: bool,
    },
    Focus {
        window_id: String,
    },

    // === Window Properties ===
    GetPosition {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<Result<Position, String>>,
    },
    SetPosition {
        window_id: String,
        x: i32,
        y: i32,
    },
    GetSize {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<Result<Size, String>>,
    },
    SetSize {
        window_id: String,
        width: u32,
        height: u32,
    },
    GetTitle {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<Result<String, String>>,
    },
    SetTitle {
        window_id: String,
        title: String,
    },
    SetResizable {
        window_id: String,
        resizable: bool,
    },
    SetDecorations {
        window_id: String,
        decorations: bool,
    },
    SetAlwaysOnTop {
        window_id: String,
        always_on_top: bool,
    },
    SetVisible {
        window_id: String,
        visible: bool,
    },

    // === State Queries ===
    GetState {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<Result<WindowState, String>>,
    },

    // === Dialogs ===
    ShowOpenDialog {
        opts: FileDialogOpts,
        respond: tokio::sync::oneshot::Sender<Option<Vec<String>>>,
    },
    ShowSaveDialog {
        opts: FileDialogOpts,
        respond: tokio::sync::oneshot::Sender<Option<String>>,
    },
    ShowMessageDialog {
        opts: MessageDialogOpts,
        respond: tokio::sync::oneshot::Sender<usize>,
    },

    // === Menus ===
    SetAppMenu {
        items: Vec<MenuItem>,
        respond: tokio::sync::oneshot::Sender<bool>,
    },
    ShowContextMenu {
        window_id: Option<String>,
        items: Vec<MenuItem>,
        respond: tokio::sync::oneshot::Sender<Option<String>>,
    },

    // === Tray ===
    CreateTray {
        opts: TrayOpts,
        respond: tokio::sync::oneshot::Sender<String>,
    },
    UpdateTray {
        tray_id: String,
        opts: TrayOpts,
        respond: tokio::sync::oneshot::Sender<bool>,
    },
    DestroyTray {
        tray_id: String,
        respond: tokio::sync::oneshot::Sender<bool>,
    },

    // === Native Handle ===
    GetNativeHandle {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<Result<NativeHandle, String>>,
    },
}

// ============================================================================
// State Management
// ============================================================================

/// State stored in OpState for window operations
pub struct WindowRuntimeState {
    pub cmd_tx: mpsc::Sender<WindowCmd>,
    pub events_rx: Rc<RefCell<Option<mpsc::Receiver<WindowSystemEvent>>>>,
    pub menu_events_rx: Rc<RefCell<Option<mpsc::Receiver<MenuEvent>>>>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for window operations
pub trait WindowCapabilityChecker: Send + Sync {
    fn check_windows(&self) -> Result<(), String>;
    fn check_menus(&self) -> Result<(), String>;
    fn check_dialogs(&self) -> Result<(), String>;
    fn check_tray(&self) -> Result<(), String>;
    fn check_native_handle(&self) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveWindowChecker;

impl WindowCapabilityChecker for PermissiveWindowChecker {
    fn check_windows(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_menus(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_dialogs(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_tray(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_native_handle(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store the capability checker in OpState
pub struct WindowCapabilities {
    pub checker: Arc<dyn WindowCapabilityChecker>,
}

impl Default for WindowCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveWindowChecker),
        }
    }
}

// Helper functions for capability checks
fn check_window_capability(state: &OpState) -> Result<(), WindowError> {
    if let Some(caps) = state.try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_windows()
            .map_err(WindowError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_dialog_capability(state: &OpState) -> Result<(), WindowError> {
    if let Some(caps) = state.try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_dialogs()
            .map_err(WindowError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_menu_capability(state: &OpState) -> Result<(), WindowError> {
    if let Some(caps) = state.try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_menus()
            .map_err(WindowError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_tray_capability(state: &OpState) -> Result<(), WindowError> {
    if let Some(caps) = state.try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_tray()
            .map_err(WindowError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_native_handle_capability(state: &OpState) -> Result<(), WindowError> {
    if let Some(caps) = state.try_borrow::<WindowCapabilities>() {
        caps.checker
            .check_native_handle()
            .map_err(WindowError::permission_denied)
    } else {
        Ok(())
    }
}

// ============================================================================
// Window Lifecycle Ops (10)
// ============================================================================

/// Create a new window
#[op2(async)]
#[string]
async fn op_window_create(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: WindowOpts,
) -> Result<String, WindowError> {
    {
        let s = state.borrow();
        check_window_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::Create {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::create_failed)
}

/// Close a window by ID
#[op2(async)]
async fn op_window_close(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::Close {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

/// Minimize a window
#[op2(async)]
async fn op_window_minimize(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::Minimize { window_id })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Maximize a window
#[op2(async)]
async fn op_window_maximize(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::Maximize { window_id })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Unmaximize (restore from maximized)
#[op2(async)]
async fn op_window_unmaximize(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::Unmaximize { window_id })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Restore from minimized
#[op2(async)]
async fn op_window_restore(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::Restore { window_id })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Set fullscreen mode
#[op2(async)]
async fn op_window_set_fullscreen(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    fullscreen: bool,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetFullscreen {
            window_id,
            fullscreen,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if window is fullscreen
#[op2(async)]
async fn op_window_is_fullscreen(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_fullscreen)
}

/// Focus a window
#[op2(async)]
async fn op_window_focus(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::Focus { window_id })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if window is focused
#[op2(async)]
async fn op_window_is_focused(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_focused)
}

// ============================================================================
// Window Properties Ops (16)
// ============================================================================

/// Get window position
#[op2(async)]
#[serde]
async fn op_window_get_position(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<Position, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetPosition {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)
}

/// Set window position
#[op2(async)]
async fn op_window_set_position(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    x: i32,
    y: i32,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetPosition { window_id, x, y })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Get window size
#[op2(async)]
#[serde]
async fn op_window_get_size(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<Size, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetSize {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)
}

/// Set window size
#[op2(async)]
async fn op_window_set_size(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    width: u32,
    height: u32,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetSize {
            window_id,
            width,
            height,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Get window title
#[op2(async)]
#[string]
async fn op_window_get_title(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<String, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetTitle {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)
}

/// Set window title
#[op2(async)]
async fn op_window_set_title(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    #[string] title: String,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetTitle { window_id, title })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Set window resizable
#[op2(async)]
async fn op_window_set_resizable(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    resizable: bool,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetResizable {
            window_id,
            resizable,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if window is resizable
#[op2(async)]
async fn op_window_is_resizable(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_resizable)
}

/// Set window decorations
#[op2(async)]
async fn op_window_set_decorations(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    decorations: bool,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetDecorations {
            window_id,
            decorations,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if window has decorations
#[op2(async)]
async fn op_window_has_decorations(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.has_decorations)
}

/// Set always on top
#[op2(async)]
async fn op_window_set_always_on_top(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    always_on_top: bool,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetAlwaysOnTop {
            window_id,
            always_on_top,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if always on top
#[op2(async)]
async fn op_window_is_always_on_top(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_always_on_top)
}

/// Set window visibility
#[op2(async)]
async fn op_window_set_visible(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    visible: bool,
) -> Result<(), WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    cmd_tx
        .send(WindowCmd::SetVisible { window_id, visible })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))
}

/// Check if window is visible
#[op2(async)]
async fn op_window_is_visible(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_visible)
}

/// Check if window is maximized
#[op2(async)]
async fn op_window_is_maximized(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_maximized)
}

/// Check if window is minimized
#[op2(async)]
async fn op_window_is_minimized(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WindowError> {
    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetState {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let state = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(WindowError::window_not_found)?;

    Ok(state.is_minimized)
}

// ============================================================================
// Dialog Ops (3)
// ============================================================================

/// Show file open dialog
#[op2(async)]
#[serde]
async fn op_window_dialog_open(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: FileDialogOpts,
) -> Result<Option<Vec<String>>, WindowError> {
    {
        let s = state.borrow();
        check_dialog_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::ShowOpenDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

/// Show file save dialog
#[op2(async)]
#[serde]
async fn op_window_dialog_save(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: FileDialogOpts,
) -> Result<serde_json::Value, WindowError> {
    {
        let s = state.borrow();
        check_dialog_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::ShowSaveDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?;

    Ok(result
        .map(|s| serde_json::json!(s))
        .unwrap_or(serde_json::Value::Null))
}

/// Show message dialog
#[op2(async)]
async fn op_window_dialog_message(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: MessageDialogOpts,
) -> Result<u32, WindowError> {
    {
        let s = state.borrow();
        check_dialog_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::ShowMessageDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?;

    Ok(result as u32)
}

// ============================================================================
// Menu Ops (3)
// ============================================================================

/// Set the application menu bar
#[op2(async)]
async fn op_window_set_app_menu(
    state: Rc<RefCell<OpState>>,
    #[serde] items: Vec<MenuItem>,
) -> Result<bool, WindowError> {
    {
        let s = state.borrow();
        check_menu_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::SetAppMenu {
            items,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

/// Show a context menu
#[op2(async)]
#[string]
async fn op_window_show_context_menu(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: Option<String>,
    #[serde] items: Vec<MenuItem>,
) -> Result<String, WindowError> {
    {
        let s = state.borrow();
        check_menu_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::ShowContextMenu {
            window_id,
            items,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?;

    Ok(result.unwrap_or_default())
}

/// Receive menu events
#[op2(async)]
#[serde]
async fn op_window_menu_recv(
    state: Rc<RefCell<OpState>>,
) -> Result<Option<MenuEvent>, WindowError> {
    {
        let s = state.borrow();
        check_menu_capability(&s)?;
    }

    let maybe_rx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        let result = win_state.menu_events_rx.borrow_mut().take();
        result
    };

    if let Some(mut rx) = maybe_rx {
        let result = rx.recv().await;

        // Put the receiver back
        {
            let s = state.borrow();
            let win_state = s.borrow::<WindowRuntimeState>();
            *win_state.menu_events_rx.borrow_mut() = Some(rx);
        }

        Ok(result)
    } else {
        Ok(None)
    }
}

// ============================================================================
// Tray Ops (3)
// ============================================================================

/// Create a system tray icon
#[op2(async)]
#[string]
async fn op_window_create_tray(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: TrayOpts,
) -> Result<String, WindowError> {
    {
        let s = state.borrow();
        check_tray_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::CreateTray {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

/// Update an existing tray icon
#[op2(async)]
async fn op_window_update_tray(
    state: Rc<RefCell<OpState>>,
    #[string] tray_id: String,
    #[serde] opts: TrayOpts,
) -> Result<bool, WindowError> {
    {
        let s = state.borrow();
        check_tray_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::UpdateTray {
            tray_id,
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

/// Destroy a tray icon
#[op2(async)]
async fn op_window_destroy_tray(
    state: Rc<RefCell<OpState>>,
    #[string] tray_id: String,
) -> Result<bool, WindowError> {
    {
        let s = state.borrow();
        check_tray_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::DestroyTray {
            tray_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))
}

// ============================================================================
// Events & Native Ops (2)
// ============================================================================

/// Receive window system events
#[op2(async)]
#[serde]
async fn op_window_events_recv(
    state: Rc<RefCell<OpState>>,
) -> Result<Option<WindowSystemEvent>, WindowError> {
    let maybe_rx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        let result = win_state.events_rx.borrow_mut().take();
        result
    };

    if let Some(mut rx) = maybe_rx {
        let result = rx.recv().await;

        // Put the receiver back
        {
            let s = state.borrow();
            let win_state = s.borrow::<WindowRuntimeState>();
            *win_state.events_rx.borrow_mut() = Some(rx);
        }

        Ok(result)
    } else {
        Ok(None)
    }
}

/// Get native window handle
#[op2(async)]
#[serde]
async fn op_window_get_native_handle(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<NativeHandle, WindowError> {
    {
        let s = state.borrow();
        check_native_handle_capability(&s)?;
    }

    let cmd_tx = {
        let s = state.borrow();
        let win_state = s.borrow::<WindowRuntimeState>();
        win_state.cmd_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    cmd_tx
        .send(WindowCmd::GetNativeHandle {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| WindowError::channel_send(e.to_string()))?;

    respond_rx
        .await
        .map_err(|e| WindowError::channel_recv(e.to_string()))?
        .map_err(|_| WindowError::native_handle_unavailable())
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Build the window extension
pub fn window_extension() -> Extension {
    host_window::ext()
}

/// Initialize window state in OpState - must be called after creating JsRuntime
pub fn init_window_state(
    op_state: &mut OpState,
    cmd_tx: mpsc::Sender<WindowCmd>,
    events_rx: mpsc::Receiver<WindowSystemEvent>,
    menu_events_rx: mpsc::Receiver<MenuEvent>,
) {
    op_state.put(WindowRuntimeState {
        cmd_tx,
        events_rx: Rc::new(RefCell::new(Some(events_rx))),
        menu_events_rx: Rc::new(RefCell::new(Some(menu_events_rx))),
    });
}

/// Initialize window capabilities in OpState
pub fn init_window_capabilities(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn WindowCapabilityChecker>>,
) {
    if let Some(caps) = capabilities {
        op_state.put(WindowCapabilities { checker: caps });
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
        assert_eq!(WindowErrorCode::Generic as u32, 6000);
        assert_eq!(WindowErrorCode::PermissionDenied as u32, 6001);
        assert_eq!(WindowErrorCode::WindowNotFound as u32, 6002);
        assert_eq!(WindowErrorCode::NativeHandleUnavailable as u32, 6014);
    }

    #[test]
    fn test_error_display() {
        let err = WindowError::generic("test error");
        assert!(err.to_string().contains("6000"));
        assert!(err.to_string().contains("test error"));

        let err = WindowError::window_not_found("win-1");
        assert!(err.to_string().contains("6002"));
        assert!(err.to_string().contains("win-1"));
    }

    #[test]
    fn test_window_opts_default() {
        let opts = WindowOpts::default();
        assert!(opts.url.is_none());
        assert!(opts.width.is_none());
        assert!(opts.title.is_none());
    }

    #[test]
    fn test_window_opts_serialization() {
        let opts = WindowOpts {
            url: Some("app://index.html".to_string()),
            width: Some(800),
            height: Some(600),
            title: Some("Test Window".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("app://index.html"));
        assert!(json.contains("800"));

        let parsed: WindowOpts = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.url, Some("app://index.html".to_string()));
        assert_eq!(parsed.width, Some(800));
    }

    #[test]
    fn test_position_serialization() {
        let pos = Position { x: 100, y: 200 };
        let json = serde_json::to_string(&pos).unwrap();
        let parsed: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.x, 100);
        assert_eq!(parsed.y, 200);
    }

    #[test]
    fn test_size_serialization() {
        let size = Size {
            width: 800,
            height: 600,
        };
        let json = serde_json::to_string(&size).unwrap();
        let parsed: Size = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.width, 800);
        assert_eq!(parsed.height, 600);
    }

    #[test]
    fn test_native_handle_serialization() {
        let handle = NativeHandle {
            platform: "macos".to_string(),
            handle: 0x12345678,
        };
        let json = serde_json::to_string(&handle).unwrap();
        let parsed: NativeHandle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.platform, "macos");
        assert_eq!(parsed.handle, 0x12345678);
    }

    #[test]
    fn test_menu_item_serialization() {
        let item = MenuItem {
            id: Some("test-id".to_string()),
            label: "Test Label".to_string(),
            accelerator: Some("Ctrl+T".to_string()),
            enabled: Some(true),
            checked: None,
            submenu: None,
            item_type: Some("normal".to_string()),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Label"));

        let parsed: MenuItem = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, Some("test-id".to_string()));
        assert_eq!(parsed.label, "Test Label");
    }

    #[test]
    fn test_file_dialog_opts() {
        let opts = FileDialogOpts {
            title: Some("Select File".to_string()),
            default_path: None,
            filters: Some(vec![FileFilter {
                name: "Text Files".to_string(),
                extensions: vec!["txt".to_string(), "md".to_string()],
            }]),
            multiple: Some(true),
            directory: Some(false),
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: FileDialogOpts = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, Some("Select File".to_string()));
        assert_eq!(parsed.filters.as_ref().unwrap()[0].extensions.len(), 2);
    }

    #[test]
    fn test_tray_opts() {
        let opts = TrayOpts {
            icon: Some("/path/to/icon.png".to_string()),
            tooltip: Some("My App".to_string()),
            menu: Some(vec![MenuItem {
                id: Some("quit".to_string()),
                label: "Quit".to_string(),
                accelerator: None,
                enabled: Some(true),
                checked: None,
                submenu: None,
                item_type: None,
            }]),
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: TrayOpts = serde_json::from_str(&json).unwrap();
        assert!(parsed.menu.is_some());
        assert_eq!(parsed.menu.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_window_system_event() {
        let event = WindowSystemEvent {
            window_id: "win-1".to_string(),
            event_type: "resize".to_string(),
            payload: serde_json::json!({"width": 800, "height": 600}),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: WindowSystemEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.window_id, "win-1");
        assert_eq!(parsed.event_type, "resize");
    }
}
