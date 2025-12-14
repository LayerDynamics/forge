use deno_core::{op2, Extension, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Options for opening a new window
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenOpts {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub title: Option<String>,
    pub resizable: Option<bool>,
    pub decorations: Option<bool>,
    /// Channel allowlist for this window - only these channels can be used for IPC
    /// If None, uses the default from manifest; if empty Vec, no channels allowed
    pub channels: Option<Vec<String>>,
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

/// Result from message dialog
#[derive(Debug, Clone, Serialize)]
pub struct MessageDialogResult {
    pub button: usize,
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
#[derive(Debug, Clone, Serialize)]
pub struct MenuEvent {
    /// Source of the menu event: "app" for app menu, "context" for context menu, "tray" for tray menu
    pub menu_id: String,
    /// The id of the menu item that was clicked (from MenuItem.id)
    pub item_id: String,
    /// The label of the menu item
    pub label: String,
}

// IPC types (IpcEvent, ToRendererCmd) have been moved to ext_ipc module
// Re-export them for backwards compatibility
pub use ext_ipc::{IpcEvent, ToRendererCmd};

/// Command sent from Deno to the Host (for window creation, etc.)
#[derive(Debug)]
pub enum FromDenoCmd {
    CreateWindow {
        opts: OpenOpts,
        respond: tokio::sync::oneshot::Sender<String>,
    },
    CloseWindow {
        window_id: String,
        respond: tokio::sync::oneshot::Sender<bool>,
    },
    SetWindowTitle {
        window_id: String,
        title: String,
    },
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
    // Menu operations
    SetAppMenu {
        items: Vec<MenuItem>,
        respond: tokio::sync::oneshot::Sender<bool>,
    },
    ShowContextMenu {
        window_id: Option<String>,
        items: Vec<MenuItem>,
        respond: tokio::sync::oneshot::Sender<Option<String>>,
    },
    // Tray operations
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
}

/// State stored in OpState for UI operations
/// Note: IPC channels (to_renderer_tx, to_deno_rx) have moved to ext_ipc
pub struct UiState {
    pub from_deno_tx: mpsc::Sender<FromDenoCmd>,
    /// Channel for receiving menu events
    pub menu_events_rx: Rc<RefCell<Option<mpsc::Receiver<MenuEvent>>>>,
}

/// Custom error type for UI operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum UiError {
    #[error("Channel send error: {0}")]
    #[class(generic)]
    ChannelSend(String),
    #[error("Channel receive error: {0}")]
    #[class(generic)]
    ChannelRecv(String),
    #[error("Window not found: {0}")]
    #[class(generic)]
    WindowNotFound(String),
    #[error("Dialog cancelled")]
    #[class(generic)]
    DialogCancelled,
    #[error("Permission denied: {0}")]
    #[class(generic)]
    PermissionDenied(String),
}

/// Capability checker trait for UI operations
pub trait UiCapabilityChecker: Send + Sync {
    fn check_windows(&self) -> Result<(), String>;
    fn check_menus(&self) -> Result<(), String>;
    fn check_dialogs(&self) -> Result<(), String>;
    fn check_tray(&self) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveUiChecker;

impl UiCapabilityChecker for PermissiveUiChecker {
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
}

/// Wrapper to store the capability checker in OpState
pub struct UiCapabilities {
    pub checker: Arc<dyn UiCapabilityChecker>,
}

impl Default for UiCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveUiChecker),
        }
    }
}

/// Helper to check ui windows capability
fn check_ui_windows(state: &OpState) -> Result<(), UiError> {
    if let Some(caps) = state.try_borrow::<UiCapabilities>() {
        caps.checker
            .check_windows()
            .map_err(UiError::PermissionDenied)
    } else {
        Ok(())
    }
}

/// Helper to check ui dialogs capability
fn check_ui_dialogs(state: &OpState) -> Result<(), UiError> {
    if let Some(caps) = state.try_borrow::<UiCapabilities>() {
        caps.checker
            .check_dialogs()
            .map_err(UiError::PermissionDenied)
    } else {
        Ok(())
    }
}

/// Helper to check ui menus capability
fn check_ui_menus(state: &OpState) -> Result<(), UiError> {
    if let Some(caps) = state.try_borrow::<UiCapabilities>() {
        caps.checker
            .check_menus()
            .map_err(UiError::PermissionDenied)
    } else {
        Ok(())
    }
}

/// Helper to check ui tray capability
fn check_ui_tray(state: &OpState) -> Result<(), UiError> {
    if let Some(caps) = state.try_borrow::<UiCapabilities>() {
        caps.checker.check_tray().map_err(UiError::PermissionDenied)
    } else {
        Ok(())
    }
}

/// Open a new window - async op that sends request to host and waits for window ID
#[op2(async)]
#[string]
async fn op_ui_open_window(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: OpenOpts,
) -> Result<String, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_windows(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::CreateWindow {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let window_id = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(window_id)
}

/// Close a window by ID
#[op2(async)]
async fn op_ui_close_window(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, UiError> {
    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::CloseWindow {
            window_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

/// Set window title
#[op2(async)]
async fn op_ui_set_window_title(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    #[string] title: String,
) -> Result<(), UiError> {
    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    from_deno_tx
        .send(FromDenoCmd::SetWindowTitle { window_id, title })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    Ok(())
}

// IPC ops (op_ui_window_send, op_ui_window_recv) have been moved to ext_ipc module
// Use host:ipc for sendToWindow, recvWindowEvent, windowEvents

/// Show file open dialog
#[op2(async)]
#[serde]
async fn op_ui_dialog_open(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: FileDialogOpts,
) -> Result<Option<Vec<String>>, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_dialogs(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::ShowOpenDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

/// Show file save dialog - returns path as JSON (null if cancelled)
#[op2(async)]
#[serde]
async fn op_ui_dialog_save(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: FileDialogOpts,
) -> Result<serde_json::Value, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_dialogs(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::ShowSaveDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result
        .map(|s| serde_json::json!(s))
        .unwrap_or(serde_json::Value::Null))
}

/// Show message dialog
#[op2(async)]
async fn op_ui_dialog_message(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: MessageDialogOpts,
) -> Result<u32, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_dialogs(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::ShowMessageDialog {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result as u32)
}

// ============================================================================
// Menu Operations
// ============================================================================

/// Set the application menu bar
#[op2(async)]
async fn op_ui_set_app_menu(
    state: Rc<RefCell<OpState>>,
    #[serde] items: Vec<MenuItem>,
) -> Result<bool, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_menus(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::SetAppMenu {
            items,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

/// Show a context menu at the current cursor position
/// Returns the selected menu item ID or empty string if cancelled
#[op2(async)]
#[string]
async fn op_ui_show_context_menu(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: Option<String>,
    #[serde] items: Vec<MenuItem>,
) -> Result<String, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_menus(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::ShowContextMenu {
            window_id,
            items,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    // Return empty string if None (cancelled)
    Ok(result.unwrap_or_default())
}

// ============================================================================
// Tray Operations
// ============================================================================

/// Create a system tray icon
#[op2(async)]
#[string]
async fn op_ui_create_tray(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: TrayOpts,
) -> Result<String, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_tray(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::CreateTray {
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

/// Update an existing system tray icon
#[op2(async)]
async fn op_ui_update_tray(
    state: Rc<RefCell<OpState>>,
    #[string] tray_id: String,
    #[serde] opts: TrayOpts,
) -> Result<bool, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_tray(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::UpdateTray {
            tray_id,
            opts,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

/// Destroy a system tray icon
#[op2(async)]
async fn op_ui_destroy_tray(
    state: Rc<RefCell<OpState>>,
    #[string] tray_id: String,
) -> Result<bool, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_tray(&s)?;
    }

    let from_deno_tx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        ui_state.from_deno_tx.clone()
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    from_deno_tx
        .send(FromDenoCmd::DestroyTray {
            tray_id,
            respond: respond_tx,
        })
        .await
        .map_err(|e| UiError::ChannelSend(e.to_string()))?;

    let result = respond_rx
        .await
        .map_err(|e| UiError::ChannelRecv(e.to_string()))?;

    Ok(result)
}

// ============================================================================
// Menu Event Operations
// ============================================================================

/// Receive the next menu event (blocking)
/// Returns null when no more events are available
#[op2(async)]
#[serde]
async fn op_ui_menu_recv(state: Rc<RefCell<OpState>>) -> Result<Option<MenuEvent>, UiError> {
    // Check capability
    {
        let s = state.borrow();
        check_ui_menus(&s)?;
    }

    let maybe_rx = {
        let s = state.borrow();
        let ui_state = s.borrow::<UiState>();
        let result = ui_state.menu_events_rx.borrow_mut().take();
        result
    };

    if let Some(mut rx) = maybe_rx {
        let result = rx.recv().await;

        // Put the receiver back
        {
            let s = state.borrow();
            let ui_state = s.borrow::<UiState>();
            *ui_state.menu_events_rx.borrow_mut() = Some(rx);
        }

        Ok(result)
    } else {
        Ok(None)
    }
}

// Include generated extension! macro from build.rs (contains transpiled TypeScript)
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Build the UI extension
/// Note: IPC channels are initialized via init_ui_state() after JsRuntime creation
pub fn ui_extension() -> Extension {
    host_ui::ext()
}

/// Initialize UI state in OpState - must be called after creating JsRuntime
/// Note: IPC channels are now initialized via ext_ipc::init_ipc_state
pub fn init_ui_state(
    op_state: &mut OpState,
    from_deno_tx: mpsc::Sender<FromDenoCmd>,
    menu_events_rx: mpsc::Receiver<MenuEvent>,
) {
    op_state.put(UiState {
        from_deno_tx,
        menu_events_rx: Rc::new(RefCell::new(Some(menu_events_rx))),
    });
}

/// Initialize UI capabilities in OpState
pub fn init_ui_capabilities(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn UiCapabilityChecker>>,
) {
    if let Some(caps) = capabilities {
        op_state.put(UiCapabilities { checker: caps });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = UiError::ChannelSend("test error".to_string());
        assert!(err.to_string().contains("Channel send error"));

        let err = UiError::WindowNotFound("win-1".to_string());
        assert!(err.to_string().contains("Window not found"));

        let err = UiError::DialogCancelled;
        assert_eq!(err.to_string(), "Dialog cancelled");

        let err = UiError::PermissionDenied("windows".to_string());
        assert!(err.to_string().contains("Permission denied"));
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
    fn test_menu_item_with_submenu() {
        let item = MenuItem {
            id: None,
            label: "File".to_string(),
            accelerator: None,
            enabled: None,
            checked: None,
            submenu: Some(vec![
                MenuItem {
                    id: Some("open".to_string()),
                    label: "Open".to_string(),
                    accelerator: Some("Ctrl+O".to_string()),
                    enabled: Some(true),
                    checked: None,
                    submenu: None,
                    item_type: None,
                },
                MenuItem {
                    id: None,
                    label: "".to_string(),
                    accelerator: None,
                    enabled: None,
                    checked: None,
                    submenu: None,
                    item_type: Some("separator".to_string()),
                },
            ]),
            item_type: None,
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: MenuItem = serde_json::from_str(&json).unwrap();
        assert!(parsed.submenu.is_some());
        assert_eq!(parsed.submenu.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_open_opts_defaults() {
        let json = r#"{"url": "app://index.html"}"#;
        let opts: OpenOpts = serde_json::from_str(json).unwrap();
        assert_eq!(opts.url, Some("app://index.html".to_string()));
        assert!(opts.width.is_none());
        assert!(opts.height.is_none());
        assert!(opts.channels.is_none());
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
    fn test_menu_event_serialization() {
        let event = MenuEvent {
            menu_id: "app".to_string(),
            item_id: "file-open".to_string(),
            label: "Open".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("app"));
        assert!(json.contains("file-open"));
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
}
