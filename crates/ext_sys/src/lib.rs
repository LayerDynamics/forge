//! host:sys extension - System operations for Forge apps
//!
//! Provides clipboard, notifications, system info, and environment access
//! with capability-based security.

use deno_core::{op2, Extension, OpState};
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::debug;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for system operations (for machine-readable errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SysErrorCode {
    /// Generic IO error
    Io = 2000,
    /// Permission denied by capability system
    PermissionDenied = 2001,
    /// Feature not supported on this platform
    NotSupported = 2002,
    /// Clipboard error
    Clipboard = 2003,
    /// Notification error
    Notification = 2004,
    /// Power/battery error
    Power = 2005,
}

/// Custom error type for Sys operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum SysError {
    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    Io { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Not supported: {message}")]
    #[class(generic)]
    NotSupported { code: u32, message: String },

    #[error("[{code}] Clipboard error: {message}")]
    #[class(generic)]
    Clipboard { code: u32, message: String },

    #[error("[{code}] Notification error: {message}")]
    #[class(generic)]
    Notification { code: u32, message: String },

    #[error("[{code}] Power error: {message}")]
    #[class(generic)]
    Power { code: u32, message: String },
}

impl SysError {
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            code: SysErrorCode::Io as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: SysErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn not_supported(message: impl Into<String>) -> Self {
        Self::NotSupported {
            code: SysErrorCode::NotSupported as u32,
            message: message.into(),
        }
    }

    pub fn clipboard(message: impl Into<String>) -> Self {
        Self::Clipboard {
            code: SysErrorCode::Clipboard as u32,
            message: message.into(),
        }
    }

    pub fn notification(message: impl Into<String>) -> Self {
        Self::Notification {
            code: SysErrorCode::Notification as u32,
            message: message.into(),
        }
    }

    pub fn power(message: impl Into<String>) -> Self {
        Self::Power {
            code: SysErrorCode::Power as u32,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for SysError {
    fn from(e: std::io::Error) -> Self {
        Self::io(e.to_string())
    }
}

impl From<arboard::Error> for SysError {
    fn from(e: arboard::Error) -> Self {
        Self::clipboard(e.to_string())
    }
}

// ============================================================================
// Types
// ============================================================================

/// System information
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: Option<String>,
    pub platform: String,
    pub cpu_count: usize,
}

/// Notification options
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct NotifyOpts {
    pub title: String,
    pub body: Option<String>,
    pub subtitle: Option<String>,
    pub sound: Option<bool>,
}

/// Power/battery information
#[derive(Debug, Clone, Serialize)]
pub struct PowerInfo {
    pub has_battery: bool,
    pub batteries: Vec<BatteryInfo>,
    pub ac_connected: bool,
}

/// Information about a single battery
#[derive(Debug, Clone, Serialize)]
pub struct BatteryInfo {
    pub charge_percent: f32,
    pub state: String, // "charging", "discharging", "full", "empty", "unknown"
    pub time_to_full_secs: Option<u64>,
    pub time_to_empty_secs: Option<u64>,
    pub health_percent: Option<f32>,
    pub cycle_count: Option<u32>,
    pub temperature_celsius: Option<f32>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for system operations
pub trait SysCapabilityChecker: Send + Sync {
    fn check_clipboard_read(&self) -> Result<(), String>;
    fn check_clipboard_write(&self) -> Result<(), String>;
    fn check_notify(&self) -> Result<(), String>;
    fn check_env(&self, key: &str) -> Result<(), String>;
    fn check_env_write(&self, key: &str) -> Result<(), String>;
    fn check_power(&self) -> Result<(), String>;
}

/// Default permissive checker
pub struct PermissiveSysChecker;

impl SysCapabilityChecker for PermissiveSysChecker {
    fn check_clipboard_read(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_clipboard_write(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_notify(&self) -> Result<(), String> {
        Ok(())
    }
    fn check_env(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_env_write(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_power(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store in OpState
pub struct SysCapabilities {
    pub checker: Arc<dyn SysCapabilityChecker>,
}

impl Default for SysCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveSysChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn check_clipboard_read(state: &OpState) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_clipboard_read()
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_clipboard_write(state: &OpState) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_clipboard_write()
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_notify(state: &OpState) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_notify()
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_env(state: &OpState, key: &str) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_env(key)
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_env_write(state: &OpState, key: &str) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_env_write(key)
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

fn check_power(state: &OpState) -> Result<(), SysError> {
    if let Some(caps) = state.try_borrow::<SysCapabilities>() {
        caps.checker
            .check_power()
            .map_err(SysError::permission_denied)
    } else {
        Ok(())
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Get system information (internal implementation)
fn get_system_info() -> SystemInfo {
    let platform = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .ok()
            .map(|h| h.to_string_lossy().to_string()),
        platform: platform.to_string(),
        cpu_count: std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1),
    }
}

/// Get system information
#[op2]
#[serde]
fn op_sys_info() -> SystemInfo {
    debug!("sys.info");
    get_system_info()
}

/// Get environment variable
#[op2]
#[string]
fn op_sys_env_get(state: &OpState, #[string] key: String) -> Result<Option<String>, SysError> {
    check_env(state, &key)?;
    debug!(key = %key, "sys.env_get");
    Ok(std::env::var(&key).ok())
}

/// Set environment variable
#[op2(fast)]
fn op_sys_env_set(
    state: &OpState,
    #[string] key: &str,
    #[string] value: &str,
) -> Result<(), SysError> {
    check_env_write(state, key)?;
    debug!(key = %key, "sys.env_set");
    // SAFETY: We have verified permission to write this env var.
    // The caller is responsible for ensuring no concurrent reads.
    unsafe { std::env::set_var(key, value) };
    Ok(())
}

/// Get current working directory
#[op2]
#[string]
fn op_sys_cwd() -> Result<String, SysError> {
    debug!("sys.cwd");
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| SysError::io(e.to_string()))
}

/// Get home directory
#[op2]
#[string]
fn op_sys_home_dir() -> Option<String> {
    debug!("sys.home_dir");
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

/// Get temp directory
#[op2]
#[string]
fn op_sys_temp_dir() -> String {
    debug!("sys.temp_dir");
    std::env::temp_dir().to_string_lossy().to_string()
}

/// Read clipboard text
#[op2(async)]
#[string]
async fn op_sys_clipboard_read(state: Rc<RefCell<OpState>>) -> Result<String, SysError> {
    {
        let s = state.borrow();
        check_clipboard_read(&s)?;
    }

    debug!("sys.clipboard_read");

    // Clipboard operations need to happen on main thread, so we use spawn_blocking
    tokio::task::spawn_blocking(|| {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.get_text().map_err(SysError::from)
    })
    .await
    .map_err(|e| SysError::io(e.to_string()))?
}

/// Write clipboard text
#[op2(async)]
async fn op_sys_clipboard_write(
    state: Rc<RefCell<OpState>>,
    #[string] text: String,
) -> Result<(), SysError> {
    {
        let s = state.borrow();
        check_clipboard_write(&s)?;
    }

    debug!(text_len = text.len(), "sys.clipboard_write");

    // Clipboard operations need to happen on main thread
    tokio::task::spawn_blocking(move || {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(&text).map_err(SysError::from)
    })
    .await
    .map_err(|e| SysError::io(e.to_string()))?
}

/// Show desktop notification
#[op2(async)]
async fn op_sys_notify(
    state: Rc<RefCell<OpState>>,
    #[string] title: String,
    #[string] body: String,
) -> Result<(), SysError> {
    {
        let s = state.borrow();
        check_notify(&s)?;
    }

    debug!(title = %title, "sys.notify");

    #[cfg(target_os = "macos")]
    {
        tokio::task::spawn_blocking(move || {
            mac_notification_sys::send_notification(&title, None::<&str>, &body, None)
                .map_err(|e| SysError::notification(format!("{:?}", e)))
        })
        .await
        .map_err(|e| SysError::io(e.to_string()))??;
    }

    #[cfg(not(target_os = "macos"))]
    {
        tokio::task::spawn_blocking(move || {
            notify_rust::Notification::new()
                .summary(&title)
                .body(&body)
                .show()
                .map_err(|e| SysError::notification(e.to_string()))
        })
        .await
        .map_err(|e| SysError::io(e.to_string()))??;
    }

    Ok(())
}

/// Show desktop notification with more options
#[op2(async)]
async fn op_sys_notify_ext(
    state: Rc<RefCell<OpState>>,
    #[serde] opts: NotifyOpts,
) -> Result<(), SysError> {
    {
        let s = state.borrow();
        check_notify(&s)?;
    }

    debug!(title = %opts.title, "sys.notify_ext");

    #[cfg(target_os = "macos")]
    {
        let title = opts.title;
        let subtitle = opts.subtitle;
        let body = opts.body.unwrap_or_default();
        let sound = opts.sound.unwrap_or(false);

        tokio::task::spawn_blocking(move || {
            if sound {
                let mut notification = mac_notification_sys::Notification::new();
                notification.sound("default");
                mac_notification_sys::send_notification(
                    &title,
                    subtitle.as_deref(),
                    &body,
                    Some(&notification),
                )
                .map_err(|e| SysError::notification(format!("{:?}", e)))
            } else {
                mac_notification_sys::send_notification(&title, subtitle.as_deref(), &body, None)
                    .map_err(|e| SysError::notification(format!("{:?}", e)))
            }
        })
        .await
        .map_err(|e| SysError::io(e.to_string()))??;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let title = opts.title;
        let body = opts.body.unwrap_or_default();

        tokio::task::spawn_blocking(move || {
            notify_rust::Notification::new()
                .summary(&title)
                .body(&body)
                .show()
                .map_err(|e| SysError::notification(e.to_string()))
        })
        .await
        .map_err(|e| SysError::io(e.to_string()))??;
    }

    Ok(())
}

/// Get power/battery information
#[op2(async)]
#[serde]
async fn op_sys_power_info(state: Rc<RefCell<OpState>>) -> Result<PowerInfo, SysError> {
    {
        let s = state.borrow();
        check_power(&s)?;
    }

    debug!("sys.power_info");

    // Battery operations should be done in a blocking task
    tokio::task::spawn_blocking(|| {
        use battery::units::ratio::percent;
        use battery::units::thermodynamic_temperature::degree_celsius;
        use battery::units::time::second;

        let manager = battery::Manager::new().map_err(|e| SysError::power(e.to_string()))?;

        let mut batteries = Vec::new();
        let mut has_battery = false;
        let mut ac_connected = false;

        for maybe_battery in manager
            .batteries()
            .map_err(|e| SysError::power(e.to_string()))?
        {
            match maybe_battery {
                Ok(batt) => {
                    has_battery = true;

                    let state_str = match batt.state() {
                        battery::State::Charging => {
                            ac_connected = true;
                            "charging"
                        }
                        battery::State::Discharging => "discharging",
                        battery::State::Full => {
                            ac_connected = true;
                            "full"
                        }
                        battery::State::Empty => "empty",
                        _ => "unknown",
                    };

                    let charge_percent = batt.state_of_charge().get::<percent>();

                    let time_to_full_secs = batt.time_to_full().map(|t| t.get::<second>() as u64);
                    let time_to_empty_secs = batt.time_to_empty().map(|t| t.get::<second>() as u64);

                    let health_percent = batt.state_of_health().get::<percent>();

                    let cycle_count = batt.cycle_count();

                    let temperature_celsius = batt.temperature().map(|t| t.get::<degree_celsius>());

                    batteries.push(BatteryInfo {
                        charge_percent,
                        state: state_str.to_string(),
                        time_to_full_secs,
                        time_to_empty_secs,
                        health_percent: Some(health_percent),
                        cycle_count,
                        temperature_celsius,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to read battery: {}", e);
                }
            }
        }

        // If no batteries found but we're a desktop, we're likely on AC
        if !has_battery {
            ac_connected = true;
        }

        Ok(PowerInfo {
            has_battery,
            batteries,
            ac_connected,
        })
    })
    .await
    .map_err(|e| SysError::io(e.to_string()))?
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize sys state in OpState
pub fn init_sys_state(op_state: &mut OpState, capabilities: Option<Arc<dyn SysCapabilityChecker>>) {
    if let Some(caps) = capabilities {
        op_state.put(SysCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs (contains transpiled TypeScript)
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn sys_extension() -> Extension {
    host_sys::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = SysError::permission_denied("test");
        match err {
            SysError::PermissionDenied { code, .. } => {
                assert_eq!(code, SysErrorCode::PermissionDenied as u32);
            }
            _ => panic!("Wrong error type"),
        }

        let err = SysError::clipboard("test");
        match err {
            SysError::Clipboard { code, .. } => {
                assert_eq!(code, SysErrorCode::Clipboard as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_sys_info() {
        let info = get_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.cpu_count >= 1);
    }
}
