//! ext_dock - macOS Dock customization extension
//!
//! Provides APIs for dock icon manipulation, badge text, bounce animations,
//! and dock menu management. These are macOS-only features and will no-op
//! on other platforms.

use deno_core::{op2, OpState};
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, warn};

// ============================================================================
// Error Types
// ============================================================================

/// Error codes for dock operations (8800-8809)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockErrorCode {
    /// General dock error
    DockError = 8800,
    /// Icon error
    IconError = 8801,
    /// Badge error
    BadgeError = 8802,
    /// Bounce error
    BounceError = 8803,
    /// Menu error
    MenuError = 8804,
    /// Platform not supported
    PlatformNotSupported = 8805,
    /// Invalid parameter
    InvalidParameter = 8806,
}

/// Dock operation errors
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum DockError {
    #[error("[{code}] Dock error: {message}")]
    #[class(generic)]
    DockError { code: u32, message: String },

    #[error("[{code}] Icon error: {message}")]
    #[class(generic)]
    IconError { code: u32, message: String },

    #[error("[{code}] Badge error: {message}")]
    #[class(generic)]
    BadgeError { code: u32, message: String },

    #[error("[{code}] Bounce error: {message}")]
    #[class(generic)]
    BounceError { code: u32, message: String },

    #[error("[{code}] Menu error: {message}")]
    #[class(generic)]
    MenuError { code: u32, message: String },

    #[error("[{code}] Platform not supported: {message}")]
    #[class(generic)]
    PlatformNotSupported { code: u32, message: String },

    #[error("[{code}] Invalid parameter: {message}")]
    #[class(generic)]
    InvalidParameter { code: u32, message: String },
}

impl DockError {
    pub fn dock_error(message: impl Into<String>) -> Self {
        Self::DockError {
            code: DockErrorCode::DockError as u32,
            message: message.into(),
        }
    }

    pub fn icon_error(message: impl Into<String>) -> Self {
        Self::IconError {
            code: DockErrorCode::IconError as u32,
            message: message.into(),
        }
    }

    pub fn badge_error(message: impl Into<String>) -> Self {
        Self::BadgeError {
            code: DockErrorCode::BadgeError as u32,
            message: message.into(),
        }
    }

    pub fn bounce_error(message: impl Into<String>) -> Self {
        Self::BounceError {
            code: DockErrorCode::BounceError as u32,
            message: message.into(),
        }
    }

    pub fn menu_error(message: impl Into<String>) -> Self {
        Self::MenuError {
            code: DockErrorCode::MenuError as u32,
            message: message.into(),
        }
    }

    pub fn platform_not_supported(message: impl Into<String>) -> Self {
        Self::PlatformNotSupported {
            code: DockErrorCode::PlatformNotSupported as u32,
            message: message.into(),
        }
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            code: DockErrorCode::InvalidParameter as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Extension information
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub status: String,
}

/// Bounce type for dock icon animation
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum BounceType {
    /// Critical bounce - continues until app is activated
    Critical,
    /// Informational bounce - bounces once
    #[default]
    Informational,
}

/// Menu item for dock menu
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Unique identifier for the menu item
    pub id: Option<String>,
    /// Display label
    pub label: String,
    /// Keyboard shortcut
    pub accelerator: Option<String>,
    /// Whether the item is enabled
    pub enabled: Option<bool>,
    /// Whether the item is checked (for checkbox items)
    pub checked: Option<bool>,
    /// Submenu items
    pub submenu: Option<Vec<MenuItem>>,
    /// Item type: "normal", "checkbox", "separator"
    #[serde(rename = "type")]
    pub item_type: Option<String>,
}

/// Result of a bounce operation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceResult {
    /// Bounce request ID (used to cancel)
    pub id: u64,
    /// Whether the bounce was started successfully
    pub success: bool,
}

// ============================================================================
// State
// ============================================================================

/// Counter for bounce request IDs
static BOUNCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Runtime state for dock operations
pub struct DockState {
    /// Current badge text
    pub badge_text: String,
    /// Whether dock icon is visible
    pub is_visible: bool,
}

impl Default for DockState {
    fn default() -> Self {
        Self {
            badge_text: String::new(),
            is_visible: true,
        }
    }
}

// ============================================================================
// Extension Definition
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Get the dock extension
pub fn dock_extension() -> deno_core::Extension {
    runtime_dock::ext()
}

/// Initialize dock state - call after creating JsRuntime
pub fn init_dock_state(op_state: &mut OpState) {
    op_state.put(DockState::default());
}

// ============================================================================
// Operations
// ============================================================================

/// Get extension information
#[weld_op]
#[op2]
#[serde]
pub fn op_dock_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_dock".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        status: "active".to_string(),
    }
}

/// Bounce the dock icon
#[weld_op]
#[op2]
#[serde]
pub fn op_dock_bounce(
    _state: &mut OpState,
    #[serde] bounce_type: Option<BounceType>,
) -> Result<BounceResult, DockError> {
    let bounce_type = bounce_type.unwrap_or(BounceType::Informational);
    let bounce_id = BOUNCE_COUNTER.fetch_add(1, Ordering::SeqCst);

    debug!(?bounce_type, bounce_id, "dock.bounce");

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSRequestUserAttentionType};
        use cocoa::base::nil;

        unsafe {
            let app = NSApp();
            let request_type = match bounce_type {
                BounceType::Critical => NSRequestUserAttentionType::NSCriticalRequest,
                BounceType::Informational => NSRequestUserAttentionType::NSInformationalRequest,
            };
            app.requestUserAttention_(request_type);
        }

        Ok(BounceResult {
            id: bounce_id,
            success: true,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.bounce is only supported on macOS");
        Ok(BounceResult {
            id: bounce_id,
            success: false,
        })
    }
}

/// Cancel a dock icon bounce
#[weld_op]
#[op2(fast)]
pub fn op_dock_cancel_bounce(
    _state: &mut OpState,
    #[smi] _bounce_id: u64,
) -> Result<(), DockError> {
    debug!(_bounce_id, "dock.cancel_bounce");

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSApp;
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let app = NSApp();
            // Cancel user attention request (pass 0 to cancel all)
            let _: () = msg_send![app, cancelUserAttentionRequest: 0i64];
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.cancel_bounce is only supported on macOS");
    }

    Ok(())
}

/// Set the dock badge text
#[weld_op]
#[op2(fast)]
pub fn op_dock_set_badge(state: &mut OpState, #[string] text: String) -> Result<(), DockError> {
    debug!(text = %text, "dock.set_badge");

    // Update state
    if let Some(dock_state) = state.try_borrow_mut::<DockState>() {
        dock_state.badge_text = text.clone();
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSApp;
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSString;
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let app = NSApp();
            let dock_tile: id = msg_send![app, dockTile];

            let badge_label: id = if text.is_empty() {
                nil
            } else {
                NSString::alloc(nil).init_str(&text)
            };

            let _: () = msg_send![dock_tile, setBadgeLabel: badge_label];
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.set_badge is only supported on macOS");
    }

    Ok(())
}

/// Get the current dock badge text
#[weld_op]
#[op2]
#[string]
pub fn op_dock_get_badge(state: &mut OpState) -> Result<String, DockError> {
    debug!("dock.get_badge");

    if let Some(dock_state) = state.try_borrow::<DockState>() {
        Ok(dock_state.badge_text.clone())
    } else {
        Ok(String::new())
    }
}

/// Hide the dock icon
#[weld_op]
#[op2(fast)]
pub fn op_dock_hide(state: &mut OpState) -> Result<(), DockError> {
    debug!("dock.hide");

    // Update state
    if let Some(dock_state) = state.try_borrow_mut::<DockState>() {
        dock_state.is_visible = false;
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};

        unsafe {
            let app = NSApp();
            app.setActivationPolicy_(
                NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
            );
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.hide is only supported on macOS");
    }

    Ok(())
}

/// Show the dock icon
#[weld_op]
#[op2(fast)]
pub fn op_dock_show(state: &mut OpState) -> Result<(), DockError> {
    debug!("dock.show");

    // Update state
    if let Some(dock_state) = state.try_borrow_mut::<DockState>() {
        dock_state.is_visible = true;
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};

        unsafe {
            let app = NSApp();
            app.setActivationPolicy_(
                NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular,
            );
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.show is only supported on macOS");
    }

    Ok(())
}

/// Check if dock icon is visible
#[weld_op]
#[op2(fast)]
pub fn op_dock_is_visible(state: &mut OpState) -> Result<bool, DockError> {
    debug!("dock.is_visible");

    if let Some(dock_state) = state.try_borrow::<DockState>() {
        Ok(dock_state.is_visible)
    } else {
        Ok(true)
    }
}

/// Set the dock icon from a file path
/// Pass empty string to reset to default icon
#[weld_op]
#[op2(fast)]
pub fn op_dock_set_icon(#[string] icon_path: String) -> Result<bool, DockError> {
    debug!(icon_path = %icon_path, "dock.set_icon");

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSApp, NSApplication, NSImage};
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSData;

        if !icon_path.is_empty() {
            // Read image from file
            let image_data = std::fs::read(&icon_path)
                .map_err(|e| DockError::icon_error(format!("Failed to read icon file: {}", e)))?;

            // Validate and process image
            let img = image::load_from_memory(&image_data)
                .map_err(|e| DockError::icon_error(format!("Failed to load image: {}", e)))?;

            // Convert to PNG bytes
            let mut png_bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .map_err(|e| DockError::icon_error(format!("Failed to encode image: {}", e)))?;

            unsafe {
                let app = NSApp();
                let ns_data: id = NSData::dataWithBytes_length_(
                    nil,
                    png_bytes.as_ptr() as *const std::ffi::c_void,
                    png_bytes.len() as u64,
                );
                let ns_image: id = NSImage::initWithData_(NSImage::alloc(nil), ns_data);

                if ns_image != nil {
                    app.setApplicationIconImage_(ns_image);
                    return Ok(true);
                } else {
                    return Err(DockError::icon_error("Failed to create NSImage"));
                }
            }
        } else {
            // Reset to default icon
            unsafe {
                let app = NSApp();
                app.setApplicationIconImage_(nil);
            }
            return Ok(true);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = icon_path;
        warn!("dock.set_icon is only supported on macOS");
        Ok(false)
    }
}

/// Set the dock menu
#[weld_op]
#[op2]
pub fn op_dock_set_menu(#[serde] _menu: Vec<MenuItem>) -> bool {
    debug!("dock.set_menu");

    #[cfg(target_os = "macos")]
    {
        // TODO: Implement dock menu using NSMenu
        // This requires creating NSMenu and setting it via setMenu on the dock tile
        warn!("dock.set_menu is not yet implemented");
        false
    }

    #[cfg(not(target_os = "macos"))]
    {
        warn!("dock.set_menu is only supported on macOS");
        false
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_info() {
        let info = ExtensionInfo {
            name: "ext_dock".to_string(),
            version: "0.1.0".to_string(),
            status: "active".to_string(),
        };
        assert_eq!(info.name, "ext_dock");
    }

    #[test]
    fn test_bounce_type_serialization() {
        let critical = BounceType::Critical;
        let json = serde_json::to_string(&critical).unwrap();
        assert!(json.contains("critical"));

        let informational = BounceType::Informational;
        let json = serde_json::to_string(&informational).unwrap();
        assert!(json.contains("informational"));
    }

    #[test]
    fn test_bounce_result_serialization() {
        let result = BounceResult {
            id: 42,
            success: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_menu_item_serialization() {
        let item = MenuItem {
            id: Some("test".to_string()),
            label: "Test Item".to_string(),
            accelerator: Some("Cmd+T".to_string()),
            enabled: Some(true),
            checked: None,
            submenu: None,
            item_type: Some("normal".to_string()),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test Item"));
    }

    #[test]
    fn test_error_codes_in_range() {
        assert_eq!(DockErrorCode::DockError as u32, 8800);
        assert_eq!(DockErrorCode::IconError as u32, 8801);
        assert_eq!(DockErrorCode::BadgeError as u32, 8802);
        assert_eq!(DockErrorCode::BounceError as u32, 8803);
        assert_eq!(DockErrorCode::MenuError as u32, 8804);
        assert_eq!(DockErrorCode::PlatformNotSupported as u32, 8805);
        assert_eq!(DockErrorCode::InvalidParameter as u32, 8806);
    }

    #[test]
    fn test_dock_state_default() {
        let state = DockState::default();
        assert!(state.badge_text.is_empty());
        assert!(state.is_visible);
    }
}
