//! Shell operations extension for Forge
//!
//! Provides shell integration operations including:
//! - Opening URLs in default browser
//! - Opening files/folders with default applications
//! - Revealing files in file manager
//! - Moving files to trash
//! - System beep
//! - File icon retrieval
//! - Default application queries

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::weld_op;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, error};

// Include the generated extension code from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

// ============================================================================
// Error Types
// ============================================================================

/// Error codes for shell operations (8200-8209)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellErrorCode {
    /// Failed to open external URL (8200)
    OpenExternalFailed = 8200,
    /// Failed to open path (8201)
    OpenPathFailed = 8201,
    /// Failed to show item in folder (8202)
    ShowItemFailed = 8202,
    /// Failed to move to trash (8203)
    TrashFailed = 8203,
    /// Failed to play beep (8204)
    BeepFailed = 8204,
    /// Failed to get file icon (8205)
    IconFailed = 8205,
    /// Failed to get default app (8206)
    DefaultAppFailed = 8206,
    /// Invalid path provided (8207)
    InvalidPath = 8207,
    /// Permission denied (8208)
    PermissionDenied = 8208,
    /// Operation not supported on this platform (8209)
    NotSupported = 8209,
}

impl std::fmt::Display for ShellErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as i32)
    }
}

/// Errors that can occur during shell operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum ShellError {
    #[error("[{code}] Failed to open external URL: {message}")]
    #[class(generic)]
    OpenExternalFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to open path: {message}")]
    #[class(generic)]
    OpenPathFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to show item in folder: {message}")]
    #[class(generic)]
    ShowItemFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to move to trash: {message}")]
    #[class(generic)]
    TrashFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to play beep: {message}")]
    #[class(generic)]
    BeepFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to get file icon: {message}")]
    #[class(generic)]
    IconFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Failed to get default app: {message}")]
    #[class(generic)]
    DefaultAppFailed {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Invalid path: {message}")]
    #[class(generic)]
    InvalidPath {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied {
        code: ShellErrorCode,
        message: String,
    },

    #[error("[{code}] Operation not supported: {message}")]
    #[class(generic)]
    NotSupported {
        code: ShellErrorCode,
        message: String,
    },
}

impl ShellError {
    pub fn open_external_failed(message: impl Into<String>) -> Self {
        Self::OpenExternalFailed {
            code: ShellErrorCode::OpenExternalFailed,
            message: message.into(),
        }
    }

    pub fn open_path_failed(message: impl Into<String>) -> Self {
        Self::OpenPathFailed {
            code: ShellErrorCode::OpenPathFailed,
            message: message.into(),
        }
    }

    pub fn show_item_failed(message: impl Into<String>) -> Self {
        Self::ShowItemFailed {
            code: ShellErrorCode::ShowItemFailed,
            message: message.into(),
        }
    }

    pub fn trash_failed(message: impl Into<String>) -> Self {
        Self::TrashFailed {
            code: ShellErrorCode::TrashFailed,
            message: message.into(),
        }
    }

    pub fn beep_failed(message: impl Into<String>) -> Self {
        Self::BeepFailed {
            code: ShellErrorCode::BeepFailed,
            message: message.into(),
        }
    }

    pub fn icon_failed(message: impl Into<String>) -> Self {
        Self::IconFailed {
            code: ShellErrorCode::IconFailed,
            message: message.into(),
        }
    }

    pub fn default_app_failed(message: impl Into<String>) -> Self {
        Self::DefaultAppFailed {
            code: ShellErrorCode::DefaultAppFailed,
            message: message.into(),
        }
    }

    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::InvalidPath {
            code: ShellErrorCode::InvalidPath,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: ShellErrorCode::PermissionDenied,
            message: message.into(),
        }
    }

    pub fn not_supported(message: impl Into<String>) -> Self {
        Self::NotSupported {
            code: ShellErrorCode::NotSupported,
            message: message.into(),
        }
    }
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Trait for checking shell operation permissions
pub trait ShellCapabilityChecker: Send + Sync + 'static {
    /// Check if opening external URLs is allowed
    fn can_open_external(&self) -> bool {
        true
    }

    /// Check if opening paths is allowed
    fn can_open_path(&self) -> bool {
        true
    }

    /// Check if showing items in folder is allowed
    fn can_show_item(&self) -> bool {
        true
    }

    /// Check if moving to trash is allowed
    fn can_trash(&self) -> bool {
        true
    }

    /// Check if file icon retrieval is allowed
    fn can_get_icon(&self) -> bool {
        true
    }
}

/// Default capability checker that allows all operations
pub struct DefaultShellCapabilityChecker;

impl ShellCapabilityChecker for DefaultShellCapabilityChecker {}

// ============================================================================
// Types
// ============================================================================

/// Information about a file's icon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIcon {
    /// Base64-encoded PNG data of the icon
    pub data: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Information about a default application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultAppInfo {
    /// Application name
    pub name: Option<String>,
    /// Application path
    pub path: Option<String>,
    /// Bundle identifier (macOS) or program ID (Windows)
    pub identifier: Option<String>,
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize shell state in the OpState
pub fn init_shell_state<C: ShellCapabilityChecker>(state: &mut OpState, checker: Option<C>) {
    let checker: Box<dyn ShellCapabilityChecker> = match checker {
        Some(c) => Box::new(c),
        None => Box::new(DefaultShellCapabilityChecker),
    };
    state.put(checker);
}

// ============================================================================
// Operations
// ============================================================================

/// Open a URL in the default browser
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_open_external(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] url: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_external() {
            return Err(ShellError::permission_denied(
                "Opening external URLs is not allowed",
            ));
        }
    }

    debug!("Opening external URL: {}", url);

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("mailto:") {
        return Err(ShellError::invalid_path(format!(
            "URL must start with http://, https://, or mailto:// - got: {}",
            url
        )));
    }

    open::that(&url).map_err(|e| {
        error!("Failed to open URL {}: {}", url, e);
        ShellError::open_external_failed(e.to_string())
    })?;

    Ok(())
}

/// Open a file or folder with the default application
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_open_path(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_path() {
            return Err(ShellError::permission_denied(
                "Opening paths is not allowed",
            ));
        }
    }

    debug!("Opening path: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    open::that(&path).map_err(|e| {
        error!("Failed to open path {}: {}", path, e);
        ShellError::open_path_failed(e.to_string())
    })?;

    Ok(())
}

/// Show a file in its containing folder (reveal in Finder/Explorer)
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_show_item_in_folder(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_show_item() {
            return Err(ShellError::permission_denied(
                "Showing items in folder is not allowed",
            ));
        }
    }

    debug!("Showing item in folder: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| ShellError::show_item_failed(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| ShellError::show_item_failed(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Try dbus-send first (works with most file managers)
        let dbus_result = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:file://{}", path),
                "string:",
            ])
            .spawn();

        if dbus_result.is_err() {
            // Fallback: open the containing folder
            if let Some(parent) = path_obj.parent() {
                open::that(parent).map_err(|e| ShellError::show_item_failed(e.to_string()))?;
            }
        }
    }

    Ok(())
}

/// Move a file or folder to the trash/recycle bin
#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_move_to_trash(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_trash() {
            return Err(ShellError::permission_denied(
                "Moving to trash is not allowed",
            ));
        }
    }

    debug!("Moving to trash: {}", path);

    let path_obj = Path::new(&path);
    if !path_obj.exists() {
        return Err(ShellError::invalid_path(format!(
            "Path does not exist: {}",
            path
        )));
    }

    trash::delete(&path).map_err(|e| {
        error!("Failed to move to trash {}: {}", path, e);
        ShellError::trash_failed(e.to_string())
    })?;

    Ok(())
}

/// Play the system beep sound
#[weld_op]
#[op2(fast)]
pub fn op_shell_beep() -> Result<(), ShellError> {
    debug!("Playing system beep");

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("osascript")
            .args(["-e", "beep"])
            .spawn()
            .map_err(|e| ShellError::beep_failed(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows beep using powershell
        use std::process::Command;
        Command::new("powershell")
            .args(["-c", "[console]::beep(800,200)"])
            .spawn()
            .map_err(|e| ShellError::beep_failed(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try multiple methods for Linux
        use std::process::Command;
        let result = Command::new("paplay")
            .args(["/usr/share/sounds/freedesktop/stereo/bell.oga"])
            .spawn();

        if result.is_err() {
            // Fallback to console bell
            print!("\x07");
        }
    }

    Ok(())
}

/// Get the icon for a file type
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shell_get_file_icon(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path: String,
    #[smi] size: i32,
) -> Result<FileIcon, ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_get_icon() {
            return Err(ShellError::permission_denied(
                "Getting file icons is not allowed",
            ));
        }
    }

    // Use default size of 32 if size <= 0
    let _size = if size <= 0 { 32 } else { size as u32 };
    debug!("Getting file icon for: {} (size: {})", path, _size);

    // File icon retrieval is platform-specific and complex
    // For now, return a placeholder indicating the feature is available but limited
    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use NSWorkspace to get icons
        // This requires more complex Objective-C bridging
        return Err(ShellError::not_supported(
            "File icon retrieval requires additional native bindings",
        ));
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we could use SHGetFileInfo
        return Err(ShellError::not_supported(
            "File icon retrieval requires additional native bindings",
        ));
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, we could query the icon theme
        return Err(ShellError::not_supported(
            "File icon retrieval requires additional native bindings",
        ));
    }

    #[allow(unreachable_code)]
    Err(ShellError::not_supported(
        "File icon retrieval not implemented for this platform",
    ))
}

/// Get the default application for a file type
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_shell_get_default_app(
    state: std::rc::Rc<std::cell::RefCell<OpState>>,
    #[string] path_or_extension: String,
) -> Result<DefaultAppInfo, ShellError> {
    // Check capability
    {
        let state = state.borrow();
        let checker = state.borrow::<Box<dyn ShellCapabilityChecker>>();
        if !checker.can_open_path() {
            return Err(ShellError::permission_denied(
                "Querying default apps is not allowed",
            ));
        }
    }

    debug!("Getting default app for: {}", path_or_extension);

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Use LSCopyDefaultApplicationURLForURL via mdls or other tools
        // For now, use a simpler approach with `open -Ra`
        let output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "osascript -e 'POSIX path of (path to app id (do shell script \"mdls -name kMDItemContentType -raw {} 2>/dev/null | xargs -I{{}} defaults read /System/Library/CoreServices/CoreTypes.bundle/Contents/Info CFBundleDocumentTypes | grep -A1 \\\"{{}}\\\" | grep CFBundleTypeRole | head -1\"))' 2>/dev/null || echo ''",
                    path_or_extension
                ),
            ])
            .output()
            .ok();

        if let Some(out) = output {
            let app_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !app_path.is_empty() {
                return Ok(DefaultAppInfo {
                    name: app_path.split('/').last().map(|s| s.replace(".app", "")),
                    path: Some(app_path),
                    identifier: None,
                });
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // Query Windows registry for file associations
        let ext = if path_or_extension.starts_with('.') {
            path_or_extension.clone()
        } else {
            Path::new(&path_or_extension)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default()
        };

        if !ext.is_empty() {
            let output = Command::new("cmd")
                .args(["/c", "assoc", &ext])
                .output()
                .ok();

            if let Some(out) = output {
                let assoc = String::from_utf8_lossy(&out.stdout);
                if let Some(prog_id) = assoc.split('=').nth(1) {
                    return Ok(DefaultAppInfo {
                        name: Some(prog_id.trim().to_string()),
                        path: None,
                        identifier: Some(prog_id.trim().to_string()),
                    });
                }
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Use xdg-mime to query default applications
        let mime_output = Command::new("xdg-mime")
            .args(["query", "filetype", &path_or_extension])
            .output()
            .ok();

        if let Some(out) = mime_output {
            let mime_type = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !mime_type.is_empty() {
                let app_output = Command::new("xdg-mime")
                    .args(["query", "default", &mime_type])
                    .output()
                    .ok();

                if let Some(app_out) = app_output {
                    let desktop_file = String::from_utf8_lossy(&app_out.stdout).trim().to_string();
                    if !desktop_file.is_empty() {
                        return Ok(DefaultAppInfo {
                            name: Some(desktop_file.replace(".desktop", "")),
                            path: None,
                            identifier: Some(desktop_file),
                        });
                    }
                }
            }
        }

        return Ok(DefaultAppInfo {
            name: None,
            path: None,
            identifier: None,
        });
    }

    #[allow(unreachable_code)]
    Ok(DefaultAppInfo {
        name: None,
        path: None,
        identifier: None,
    })
}

// ============================================================================
// Extension Export
// ============================================================================

/// Get the shell extension for registration with Deno runtime
pub fn shell_extension() -> Extension {
    runtime_shell::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ShellErrorCode::OpenExternalFailed as i32, 8200);
        assert_eq!(ShellErrorCode::OpenPathFailed as i32, 8201);
        assert_eq!(ShellErrorCode::ShowItemFailed as i32, 8202);
        assert_eq!(ShellErrorCode::TrashFailed as i32, 8203);
        assert_eq!(ShellErrorCode::BeepFailed as i32, 8204);
        assert_eq!(ShellErrorCode::IconFailed as i32, 8205);
        assert_eq!(ShellErrorCode::DefaultAppFailed as i32, 8206);
        assert_eq!(ShellErrorCode::InvalidPath as i32, 8207);
        assert_eq!(ShellErrorCode::PermissionDenied as i32, 8208);
        assert_eq!(ShellErrorCode::NotSupported as i32, 8209);
    }

    #[test]
    fn test_error_messages() {
        let err = ShellError::open_external_failed("test error");
        assert!(err.to_string().contains("8200"));
        assert!(err.to_string().contains("test error"));

        let err = ShellError::invalid_path("bad path");
        assert!(err.to_string().contains("8207"));
        assert!(err.to_string().contains("bad path"));
    }

    #[test]
    fn test_default_capability_checker() {
        let checker = DefaultShellCapabilityChecker;
        assert!(checker.can_open_external());
        assert!(checker.can_open_path());
        assert!(checker.can_show_item());
        assert!(checker.can_trash());
        assert!(checker.can_get_icon());
    }

    #[test]
    fn test_file_icon_serialization() {
        let icon = FileIcon {
            data: "base64data".to_string(),
            width: 32,
            height: 32,
        };
        let json = serde_json::to_string(&icon).unwrap();
        assert!(json.contains("base64data"));
        assert!(json.contains("32"));
    }

    #[test]
    fn test_default_app_info_serialization() {
        let info = DefaultAppInfo {
            name: Some("TextEdit".to_string()),
            path: Some("/Applications/TextEdit.app".to_string()),
            identifier: Some("com.apple.TextEdit".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("TextEdit"));
        assert!(json.contains("/Applications/TextEdit.app"));
    }
}
