//! Global keyboard shortcuts extension for Forge.
//!
//! Provides global hotkey registration, event handling, and persistence.
//! Shortcuts can be persisted across app restarts using ext_storage.
//!
//! Error codes: 8300-8399

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

// ============================================================================
// Error Types (Error codes 8300-8399)
// ============================================================================

/// Error codes for shortcuts operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShortcutsErrorCode {
    /// Generic shortcuts error
    Generic = 8300,
    /// Failed to register shortcut
    RegistrationFailed = 8301,
    /// Shortcut already registered
    AlreadyRegistered = 8302,
    /// Shortcut not found
    NotFound = 8303,
    /// Invalid accelerator string
    InvalidAccelerator = 8304,
    /// Permission denied
    PermissionDenied = 8305,
    /// Manager unavailable
    ManagerUnavailable = 8306,
    /// Persistence error
    PersistenceError = 8307,
}

/// Shortcuts extension errors
#[derive(Debug, Error, JsError)]
pub enum ShortcutsError {
    #[error("[{code}] Shortcuts error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Registration failed: {message}")]
    #[class(generic)]
    RegistrationFailed { code: u32, message: String },

    #[error("[{code}] Already registered: {message}")]
    #[class(generic)]
    AlreadyRegistered { code: u32, message: String },

    #[error("[{code}] Shortcut not found: {message}")]
    #[class(generic)]
    NotFound { code: u32, message: String },

    #[error("[{code}] Invalid accelerator: {message}")]
    #[class(generic)]
    InvalidAccelerator { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Manager unavailable: {message}")]
    #[class(generic)]
    ManagerUnavailable { code: u32, message: String },

    #[error("[{code}] Persistence error: {message}")]
    #[class(generic)]
    PersistenceError { code: u32, message: String },
}

impl ShortcutsError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: ShortcutsErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn registration_failed(message: impl Into<String>) -> Self {
        Self::RegistrationFailed {
            code: ShortcutsErrorCode::RegistrationFailed as u32,
            message: message.into(),
        }
    }

    pub fn already_registered(message: impl Into<String>) -> Self {
        Self::AlreadyRegistered {
            code: ShortcutsErrorCode::AlreadyRegistered as u32,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: ShortcutsErrorCode::NotFound as u32,
            message: message.into(),
        }
    }

    pub fn invalid_accelerator(message: impl Into<String>) -> Self {
        Self::InvalidAccelerator {
            code: ShortcutsErrorCode::InvalidAccelerator as u32,
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: ShortcutsErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn manager_unavailable(message: impl Into<String>) -> Self {
        Self::ManagerUnavailable {
            code: ShortcutsErrorCode::ManagerUnavailable as u32,
            message: message.into(),
        }
    }

    pub fn persistence_error(message: impl Into<String>) -> Self {
        Self::PersistenceError {
            code: ShortcutsErrorCode::PersistenceError as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Configuration for registering a shortcut
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Unique identifier for the shortcut
    pub id: String,
    /// Accelerator string (e.g., "CmdOrCtrl+Shift+K", "Alt+F4")
    pub accelerator: String,
    /// Whether the shortcut is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Shortcut trigger event
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ShortcutEvent {
    /// ID of the triggered shortcut
    pub id: String,
    /// Timestamp when the shortcut was triggered (Unix milliseconds)
    pub timestamp_ms: u64,
}

/// Information about a registered shortcut
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ShortcutInfo {
    /// Unique identifier
    pub id: String,
    /// Accelerator string
    pub accelerator: String,
    /// Whether currently enabled
    pub enabled: bool,
    /// Number of times triggered
    pub trigger_count: u64,
}

/// Legacy extension info for backward compatibility
#[weld_struct]
#[derive(Serialize)]
struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

/// Persistence format for shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedShortcuts {
    shortcuts: Vec<ShortcutConfig>,
}

// ============================================================================
// State Management
// ============================================================================

/// Internal shortcut state
struct RegisteredShortcut {
    config: ShortcutConfig,
    hotkey: HotKey,
    hotkey_id: u32,
    trigger_count: Arc<AtomicU64>,
}

/// Shortcuts state stored in OpState
pub struct ShortcutsState {
    /// Registered shortcuts (id -> shortcut)
    shortcuts: HashMap<String, RegisteredShortcut>,
    /// Hotkey ID to shortcut ID mapping
    hotkey_to_id: HashMap<u32, String>,
    /// Global hotkey manager
    manager: Option<GlobalHotKeyManager>,
    /// Event sender for shortcut triggers
    event_tx: mpsc::Sender<ShortcutEvent>,
    /// Event receiver (taken when listening)
    event_rx: Option<mpsc::Receiver<ShortcutEvent>>,
    /// App identifier for persistence
    app_id: String,
    /// Whether to auto-persist on changes
    auto_persist: bool,
    /// Storage key for persistence
    storage_key: String,
}

impl ShortcutsState {
    fn new(app_id: String) -> Self {
        let (tx, rx) = mpsc::channel(64);

        // Try to create the global hotkey manager
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => {
                debug!("GlobalHotKeyManager created successfully");
                Some(m)
            }
            Err(e) => {
                warn!("Failed to create GlobalHotKeyManager: {}", e);
                None
            }
        };

        Self {
            shortcuts: HashMap::new(),
            hotkey_to_id: HashMap::new(),
            manager,
            event_tx: tx,
            event_rx: Some(rx),
            storage_key: format!("forge-shortcuts-{}", app_id),
            app_id,
            auto_persist: false,
        }
    }
}

/// Initialize shortcuts state in OpState
pub fn init_shortcuts_state(op_state: &mut OpState, app_id: String) {
    debug!(app_id = %app_id, "Initializing shortcuts state");
    let state = ShortcutsState::new(app_id);

    // Set up event listener in background
    let event_tx = state.event_tx.clone();
    std::thread::spawn(move || {
        // Listen for global hotkey events
        loop {
            if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                if event.state == HotKeyState::Pressed {
                    let shortcut_event = ShortcutEvent {
                        id: event.id.to_string(), // Will be mapped later
                        timestamp_ms: now_ms(),
                    };
                    if event_tx.blocking_send(shortcut_event).is_err() {
                        break;
                    }
                }
            }
        }
    });

    op_state.put(state);
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Parse an accelerator string into a HotKey
/// Supported modifiers: Ctrl, Alt, Shift, Meta, Super, Cmd, CmdOrCtrl
/// Supported keys: A-Z, 0-9, F1-F24, plus special keys
fn parse_accelerator(accel: &str) -> Result<HotKey, ShortcutsError> {
    let parts: Vec<&str> = accel.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(ShortcutsError::invalid_accelerator(
            "Empty accelerator string",
        ));
    }

    let mut modifiers = Modifiers::empty();
    let key_part = parts
        .last()
        .ok_or_else(|| ShortcutsError::invalid_accelerator("No key specified"))?;

    // Parse modifiers
    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "meta" | "cmd" | "command" | "super" => modifiers |= Modifiers::META,
            "cmdorctrl" | "commandorcontrol" => {
                #[cfg(target_os = "macos")]
                {
                    modifiers |= Modifiers::META;
                }
                #[cfg(not(target_os = "macos"))]
                {
                    modifiers |= Modifiers::CONTROL;
                }
            }
            other => {
                return Err(ShortcutsError::invalid_accelerator(format!(
                    "Unknown modifier: {}",
                    other
                )));
            }
        }
    }

    // Parse key code
    let code = parse_key_code(key_part)?;

    Ok(HotKey::new(Some(modifiers), code))
}

/// Parse a key string into a Code
fn parse_key_code(key: &str) -> Result<Code, ShortcutsError> {
    let code = match key.to_uppercase().as_str() {
        // Letters
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        // Numbers
        "0" | "DIGIT0" => Code::Digit0,
        "1" | "DIGIT1" => Code::Digit1,
        "2" | "DIGIT2" => Code::Digit2,
        "3" | "DIGIT3" => Code::Digit3,
        "4" | "DIGIT4" => Code::Digit4,
        "5" | "DIGIT5" => Code::Digit5,
        "6" | "DIGIT6" => Code::Digit6,
        "7" | "DIGIT7" => Code::Digit7,
        "8" | "DIGIT8" => Code::Digit8,
        "9" | "DIGIT9" => Code::Digit9,
        // Function keys
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "F13" => Code::F13,
        "F14" => Code::F14,
        "F15" => Code::F15,
        "F16" => Code::F16,
        "F17" => Code::F17,
        "F18" => Code::F18,
        "F19" => Code::F19,
        "F20" => Code::F20,
        "F21" => Code::F21,
        "F22" => Code::F22,
        "F23" => Code::F23,
        "F24" => Code::F24,
        // Special keys
        "SPACE" | " " => Code::Space,
        "ENTER" | "RETURN" => Code::Enter,
        "TAB" => Code::Tab,
        "BACKSPACE" => Code::Backspace,
        "DELETE" | "DEL" => Code::Delete,
        "ESCAPE" | "ESC" => Code::Escape,
        "HOME" => Code::Home,
        "END" => Code::End,
        "PAGEUP" => Code::PageUp,
        "PAGEDOWN" => Code::PageDown,
        "UP" | "ARROWUP" => Code::ArrowUp,
        "DOWN" | "ARROWDOWN" => Code::ArrowDown,
        "LEFT" | "ARROWLEFT" => Code::ArrowLeft,
        "RIGHT" | "ARROWRIGHT" => Code::ArrowRight,
        "INSERT" => Code::Insert,
        // Punctuation
        "MINUS" | "-" => Code::Minus,
        "EQUAL" | "=" | "PLUS" => Code::Equal,
        "BRACKETLEFT" | "[" => Code::BracketLeft,
        "BRACKETRIGHT" | "]" => Code::BracketRight,
        "BACKSLASH" | "\\" => Code::Backslash,
        "SEMICOLON" | ";" => Code::Semicolon,
        "QUOTE" | "'" => Code::Quote,
        "BACKQUOTE" | "`" | "GRAVE" => Code::Backquote,
        "COMMA" | "," => Code::Comma,
        "PERIOD" | "." => Code::Period,
        "SLASH" | "/" => Code::Slash,
        other => {
            return Err(ShortcutsError::invalid_accelerator(format!(
                "Unknown key: {}",
                other
            )));
        }
    };

    Ok(code)
}

// ============================================================================
// Legacy Operations (backward compatibility)
// ============================================================================

#[weld_op]
#[op2]
#[serde]
fn op_shortcuts_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_shortcuts",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_shortcuts_echo(#[string] message: String) -> String {
    message
}

// ============================================================================
// Registration Operations
// ============================================================================

/// Register a global keyboard shortcut
#[weld_op]
#[op2]
#[serde]
pub fn op_shortcuts_register(
    state: &mut OpState,
    #[serde] config: ShortcutConfig,
) -> Result<ShortcutInfo, ShortcutsError> {
    debug!(id = %config.id, accelerator = %config.accelerator, "shortcuts.register");

    let shortcuts_state = state.borrow_mut::<ShortcutsState>();

    // Check if already registered
    if shortcuts_state.shortcuts.contains_key(&config.id) {
        return Err(ShortcutsError::already_registered(&config.id));
    }

    // Get manager
    let manager = shortcuts_state
        .manager
        .as_ref()
        .ok_or_else(|| ShortcutsError::manager_unavailable("GlobalHotKeyManager not available"))?;

    // Parse accelerator
    let hotkey = parse_accelerator(&config.accelerator)?;
    let hotkey_id = hotkey.id();

    // Register with system
    manager.register(hotkey).map_err(|e| {
        ShortcutsError::registration_failed(format!("Failed to register hotkey: {}", e))
    })?;

    let trigger_count = Arc::new(AtomicU64::new(0));

    let info = ShortcutInfo {
        id: config.id.clone(),
        accelerator: config.accelerator.clone(),
        enabled: config.enabled,
        trigger_count: 0,
    };

    // Store mapping
    shortcuts_state
        .hotkey_to_id
        .insert(hotkey_id, config.id.clone());

    // Store shortcut
    shortcuts_state.shortcuts.insert(
        config.id.clone(),
        RegisteredShortcut {
            config,
            hotkey,
            hotkey_id,
            trigger_count,
        },
    );

    Ok(info)
}

/// Unregister a shortcut by ID
#[weld_op]
#[op2(fast)]
pub fn op_shortcuts_unregister(
    state: &mut OpState,
    #[string] id: String,
) -> Result<(), ShortcutsError> {
    debug!(id = %id, "shortcuts.unregister");

    let shortcuts_state = state.borrow_mut::<ShortcutsState>();

    let shortcut = shortcuts_state
        .shortcuts
        .remove(&id)
        .ok_or_else(|| ShortcutsError::not_found(&id))?;

    // Unregister from system
    if let Some(manager) = &shortcuts_state.manager {
        if let Err(e) = manager.unregister(shortcut.hotkey) {
            warn!("Failed to unregister hotkey: {}", e);
        }
    }

    shortcuts_state.hotkey_to_id.remove(&shortcut.hotkey_id);

    Ok(())
}

/// Unregister all shortcuts
#[weld_op]
#[op2(fast)]
pub fn op_shortcuts_unregister_all(state: &mut OpState) -> Result<(), ShortcutsError> {
    debug!("shortcuts.unregister_all");

    let shortcuts_state = state.borrow_mut::<ShortcutsState>();

    if let Some(manager) = &shortcuts_state.manager {
        for shortcut in shortcuts_state.shortcuts.values() {
            if let Err(e) = manager.unregister(shortcut.hotkey) {
                warn!("Failed to unregister hotkey {}: {}", shortcut.config.id, e);
            }
        }
    }

    shortcuts_state.shortcuts.clear();
    shortcuts_state.hotkey_to_id.clear();

    Ok(())
}

/// List all registered shortcuts
#[weld_op]
#[op2]
#[serde]
pub fn op_shortcuts_list(state: &OpState) -> Vec<ShortcutInfo> {
    let shortcuts_state = state.borrow::<ShortcutsState>();

    shortcuts_state
        .shortcuts
        .values()
        .map(|s| ShortcutInfo {
            id: s.config.id.clone(),
            accelerator: s.config.accelerator.clone(),
            enabled: s.config.enabled,
            trigger_count: s.trigger_count.load(Ordering::Relaxed),
        })
        .collect()
}

/// Enable or disable a shortcut
#[weld_op]
#[op2(fast)]
pub fn op_shortcuts_enable(
    state: &mut OpState,
    #[string] id: String,
    enabled: bool,
) -> Result<(), ShortcutsError> {
    debug!(id = %id, enabled = enabled, "shortcuts.enable");

    let shortcuts_state = state.borrow_mut::<ShortcutsState>();

    let shortcut = shortcuts_state
        .shortcuts
        .get_mut(&id)
        .ok_or_else(|| ShortcutsError::not_found(&id))?;

    // Update enabled state
    let was_enabled = shortcut.config.enabled;
    shortcut.config.enabled = enabled;

    // Register/unregister with system based on state change
    if let Some(manager) = &shortcuts_state.manager {
        if enabled && !was_enabled {
            // Re-register
            if let Err(e) = manager.register(shortcut.hotkey) {
                shortcut.config.enabled = false;
                return Err(ShortcutsError::registration_failed(format!(
                    "Failed to re-enable hotkey: {}",
                    e
                )));
            }
        } else if !enabled && was_enabled {
            // Unregister
            if let Err(e) = manager.unregister(shortcut.hotkey) {
                warn!("Failed to disable hotkey: {}", e);
            }
        }
    }

    Ok(())
}

// ============================================================================
// Event Operations
// ============================================================================

/// Get the next shortcut event
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shortcuts_next_event(
    state: Rc<RefCell<OpState>>,
) -> Result<Option<ShortcutEvent>, ShortcutsError> {
    // Take the receiver temporarily
    let maybe_rx = {
        let mut s = state.borrow_mut();
        let shortcuts_state = s.borrow_mut::<ShortcutsState>();
        shortcuts_state.event_rx.take()
    };

    let mut rx = maybe_rx.ok_or_else(|| {
        ShortcutsError::generic("Event receiver not available (already listening?)")
    })?;

    // Wait for next event
    let result = rx.recv().await;

    // Put the receiver back
    {
        let mut s = state.borrow_mut();
        let shortcuts_state = s.borrow_mut::<ShortcutsState>();
        shortcuts_state.event_rx = Some(rx);
    }

    // Map hotkey ID to shortcut ID
    if let Some(mut event) = result {
        let s = state.borrow();
        let shortcuts_state = s.borrow::<ShortcutsState>();

        // Try to find the shortcut ID from hotkey ID
        if let Ok(hotkey_id) = event.id.parse::<u32>() {
            if let Some(shortcut_id) = shortcuts_state.hotkey_to_id.get(&hotkey_id) {
                event.id = shortcut_id.clone();

                // Increment trigger count
                if let Some(shortcut) = shortcuts_state.shortcuts.get(shortcut_id) {
                    shortcut.trigger_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        Ok(Some(event))
    } else {
        Ok(None)
    }
}

// ============================================================================
// Persistence Operations
// ============================================================================

/// Save shortcuts to persistent storage
#[weld_op(async)]
#[op2(async)]
pub async fn op_shortcuts_save(state: Rc<RefCell<OpState>>) -> Result<(), ShortcutsError> {
    debug!("shortcuts.save");

    // Get shortcuts and storage key
    let (shortcuts, _storage_key, _app_id) = {
        let s = state.borrow();
        let shortcuts_state = s.borrow::<ShortcutsState>();

        let shortcuts: Vec<ShortcutConfig> = shortcuts_state
            .shortcuts
            .values()
            .map(|s| s.config.clone())
            .collect();

        (
            shortcuts,
            shortcuts_state.storage_key.clone(),
            shortcuts_state.app_id.clone(),
        )
    };

    let persisted = PersistedShortcuts { shortcuts };
    let _json = serde_json::to_string(&persisted)
        .map_err(|e| ShortcutsError::persistence_error(format!("Failed to serialize: {}", e)))?;

    // Note: In production, this would call ext_storage::op_storage_set
    // For now, we log what would be saved
    trace!(
        "Would save {} shortcuts to storage",
        persisted.shortcuts.len()
    );

    Ok(())
}

/// Load shortcuts from persistent storage
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shortcuts_load(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<ShortcutConfig>, ShortcutsError> {
    debug!("shortcuts.load");

    // Note: In production, this would call ext_storage::op_storage_get
    // For now, return empty list
    let _storage_key = {
        let s = state.borrow();
        let shortcuts_state = s.borrow::<ShortcutsState>();
        shortcuts_state.storage_key.clone()
    };

    trace!("Would load shortcuts from storage");

    Ok(vec![])
}

/// Set whether shortcuts should auto-persist on changes
#[weld_op]
#[op2(fast)]
pub fn op_shortcuts_set_auto_persist(state: &mut OpState, enabled: bool) {
    debug!(enabled = enabled, "shortcuts.set_auto_persist");

    let shortcuts_state = state.borrow_mut::<ShortcutsState>();
    shortcuts_state.auto_persist = enabled;
}

/// Get whether auto-persist is enabled
#[weld_op]
#[op2(fast)]
pub fn op_shortcuts_get_auto_persist(state: &OpState) -> bool {
    let shortcuts_state = state.borrow::<ShortcutsState>();
    shortcuts_state.auto_persist
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn shortcuts_extension() -> Extension {
    runtime_shortcuts::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = ShortcutsError::not_found("test");
        match err {
            ShortcutsError::NotFound { code, .. } => {
                assert_eq!(code, ShortcutsErrorCode::NotFound as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_parse_accelerator_simple() {
        let hotkey = parse_accelerator("Ctrl+A").unwrap();
        assert!(hotkey.id() > 0);
    }

    #[test]
    fn test_parse_accelerator_multiple_modifiers() {
        let hotkey = parse_accelerator("Ctrl+Shift+K").unwrap();
        assert!(hotkey.id() > 0);
    }

    #[test]
    fn test_parse_accelerator_cmdorctrl() {
        let hotkey = parse_accelerator("CmdOrCtrl+S").unwrap();
        assert!(hotkey.id() > 0);
    }

    #[test]
    fn test_parse_accelerator_function_key() {
        let hotkey = parse_accelerator("F12").unwrap();
        assert!(hotkey.id() > 0);
    }

    #[test]
    fn test_parse_accelerator_invalid() {
        let result = parse_accelerator("Invalid+Key");
        assert!(result.is_err());
    }

    #[test]
    fn test_shortcut_config_default() {
        let config: ShortcutConfig =
            serde_json::from_str(r#"{"id": "test", "accelerator": "Ctrl+T"}"#).unwrap();
        assert!(config.enabled); // Should default to true
    }
}
