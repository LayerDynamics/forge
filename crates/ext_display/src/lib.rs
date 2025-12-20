//! Display/Monitor extension for Forge.
//!
//! Provides display/monitor information, cursor position tracking,
//! and monitor change event subscriptions.
//!
//! Error codes: 9900-9999

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use tao::event_loop::EventLoopBuilder;
use tao::monitor::MonitorHandle;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};

// ============================================================================
// Error Types (Error codes 9900-9999)
// ============================================================================

/// Error codes for display operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DisplayErrorCode {
    /// Generic display error
    Generic = 9900,
    /// Failed to query display information
    QueryFailed = 9901,
    /// Monitor not found
    MonitorNotFound = 9902,
    /// Platform not supported
    PlatformNotSupported = 9903,
    /// Invalid subscription ID
    InvalidSubscription = 9904,
    /// Subscription limit exceeded
    SubscriptionLimitExceeded = 9905,
    /// Event loop unavailable
    EventLoopUnavailable = 9906,
}

/// Display extension errors
#[derive(Debug, Error, JsError)]
pub enum DisplayError {
    #[error("[{code}] Display error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Query failed: {message}")]
    #[class(generic)]
    QueryFailed { code: u32, message: String },

    #[error("[{code}] Monitor not found: {message}")]
    #[class(generic)]
    MonitorNotFound { code: u32, message: String },

    #[error("[{code}] Platform not supported: {message}")]
    #[class(generic)]
    PlatformNotSupported { code: u32, message: String },

    #[error("[{code}] Invalid subscription: {message}")]
    #[class(generic)]
    InvalidSubscription { code: u32, message: String },

    #[error("[{code}] Subscription limit exceeded: {message}")]
    #[class(generic)]
    SubscriptionLimitExceeded { code: u32, message: String },

    #[error("[{code}] Event loop unavailable: {message}")]
    #[class(generic)]
    EventLoopUnavailable { code: u32, message: String },
}

impl DisplayError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: DisplayErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn query_failed(message: impl Into<String>) -> Self {
        Self::QueryFailed {
            code: DisplayErrorCode::QueryFailed as u32,
            message: message.into(),
        }
    }

    pub fn monitor_not_found(message: impl Into<String>) -> Self {
        Self::MonitorNotFound {
            code: DisplayErrorCode::MonitorNotFound as u32,
            message: message.into(),
        }
    }

    pub fn platform_not_supported(message: impl Into<String>) -> Self {
        Self::PlatformNotSupported {
            code: DisplayErrorCode::PlatformNotSupported as u32,
            message: message.into(),
        }
    }

    pub fn invalid_subscription(message: impl Into<String>) -> Self {
        Self::InvalidSubscription {
            code: DisplayErrorCode::InvalidSubscription as u32,
            message: message.into(),
        }
    }

    pub fn subscription_limit_exceeded(message: impl Into<String>) -> Self {
        Self::SubscriptionLimitExceeded {
            code: DisplayErrorCode::SubscriptionLimitExceeded as u32,
            message: message.into(),
        }
    }

    pub fn event_loop_unavailable(message: impl Into<String>) -> Self {
        Self::EventLoopUnavailable {
            code: DisplayErrorCode::EventLoopUnavailable as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// Screen position
#[weld_struct]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Screen size
#[weld_struct]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Monitor/display information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    /// Unique identifier for the monitor
    pub id: String,
    /// Human-readable name of the monitor
    pub name: Option<String>,
    /// Position of the monitor in virtual screen coordinates
    pub position: Position,
    /// Size of the monitor in pixels
    pub size: Size,
    /// Scale factor (DPI scaling)
    pub scale_factor: f64,
    /// Whether this is the primary monitor
    pub is_primary: bool,
    /// Refresh rate in millihertz (e.g., 60000 for 60Hz)
    pub refresh_rate_millihertz: Option<u32>,
}

/// Cursor position with optional monitor context
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct CursorPosition {
    /// X coordinate in virtual screen space
    pub x: f64,
    /// Y coordinate in virtual screen space
    pub y: f64,
    /// ID of the monitor the cursor is on (if determinable)
    pub monitor_id: Option<String>,
}

/// Type of monitor change
#[weld_enum]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonitorChangeType {
    ScaleFactor,
    Position,
    Size,
    RefreshRate,
    Primary,
}

/// Display event types
#[weld_enum]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DisplayEvent {
    /// A new monitor was connected
    MonitorConnected { monitor: MonitorInfo },
    /// A monitor was disconnected
    MonitorDisconnected { monitor_id: String },
    /// A monitor's settings changed
    MonitorChanged {
        monitor: MonitorInfo,
        changes: Vec<MonitorChangeType>,
    },
}

/// Subscription options
#[weld_struct]
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeOptions {
    /// Polling interval in milliseconds (minimum 500ms)
    #[serde(default = "default_interval")]
    pub interval_ms: u64,
}

fn default_interval() -> u64 {
    1000
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            interval_ms: default_interval(),
        }
    }
}

/// Subscription information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionInfo {
    /// Unique subscription ID
    pub id: String,
    /// Polling interval in milliseconds
    pub interval_ms: u64,
    /// Whether subscription is active
    pub is_active: bool,
    /// Number of events delivered
    pub event_count: u64,
}

/// Legacy extension info for backward compatibility
#[weld_struct]
#[derive(Serialize)]
struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

// ============================================================================
// State Management
// ============================================================================

/// Internal subscription state
struct DisplaySubscription {
    id: String,
    options: SubscribeOptions,
    sender: mpsc::Sender<DisplayEvent>,
    receiver: Option<mpsc::Receiver<DisplayEvent>>,
    event_count: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

/// Display state stored in OpState
pub struct DisplayState {
    /// Cached monitor information
    monitors: HashMap<String, MonitorInfo>,
    /// Active subscriptions
    subscriptions: HashMap<String, DisplaySubscription>,
    /// Next subscription ID
    next_subscription_id: u64,
    /// Maximum allowed subscriptions
    max_subscriptions: usize,
    /// Last update timestamp
    last_update_ms: u64,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            monitors: HashMap::new(),
            subscriptions: HashMap::new(),
            next_subscription_id: 1,
            max_subscriptions: 10,
            last_update_ms: 0,
        }
    }
}

/// Initialize display state in OpState
pub fn init_display_state(op_state: &mut OpState) {
    debug!("Initializing display state");
    op_state.put(DisplayState::default());
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a unique monitor ID from MonitorHandle
fn monitor_id(handle: &MonitorHandle) -> String {
    // Use the monitor name and position to create a unique ID
    let name = handle.name().unwrap_or_default();
    let pos = handle.position();
    format!("{}:{}x{}", name, pos.x, pos.y)
}

/// Convert MonitorHandle to MonitorInfo
fn monitor_handle_to_info(handle: &MonitorHandle, is_primary: bool) -> MonitorInfo {
    let pos = handle.position();
    let size = handle.size();
    let video_mode = handle.video_modes().next();

    MonitorInfo {
        id: monitor_id(handle),
        name: handle.name(),
        position: Position { x: pos.x, y: pos.y },
        size: Size {
            width: size.width,
            height: size.height,
        },
        scale_factor: handle.scale_factor(),
        is_primary,
        refresh_rate_millihertz: video_mode.map(|m| m.refresh_rate() as u32 * 1000), // Convert Hz to millihertz
    }
}

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Query all monitors using tao event loop
fn query_monitors() -> Result<Vec<MonitorInfo>, DisplayError> {
    // Create a temporary event loop to query monitors
    // Note: In a real app, we'd share the event loop, but for standalone queries this works
    let event_loop = EventLoopBuilder::new().build();

    let primary = event_loop.primary_monitor();
    let primary_id = primary.as_ref().map(|m| monitor_id(m));

    let monitors: Vec<MonitorInfo> = event_loop
        .available_monitors()
        .map(|handle| {
            let id = monitor_id(&handle);
            let is_primary = primary_id.as_ref().map(|p| p == &id).unwrap_or(false);
            monitor_handle_to_info(&handle, is_primary)
        })
        .collect();

    Ok(monitors)
}

/// Detect changes between old and new monitor info
fn detect_changes(old: &MonitorInfo, new: &MonitorInfo) -> Vec<MonitorChangeType> {
    let mut changes = Vec::new();

    if (old.scale_factor - new.scale_factor).abs() > 0.001 {
        changes.push(MonitorChangeType::ScaleFactor);
    }
    if old.position != new.position {
        changes.push(MonitorChangeType::Position);
    }
    if old.size != new.size {
        changes.push(MonitorChangeType::Size);
    }
    if old.refresh_rate_millihertz != new.refresh_rate_millihertz {
        changes.push(MonitorChangeType::RefreshRate);
    }
    if old.is_primary != new.is_primary {
        changes.push(MonitorChangeType::Primary);
    }

    changes
}

// ============================================================================
// Legacy Operations (backward compatibility)
// ============================================================================

#[weld_op]
#[op2]
#[serde]
fn op_display_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_display",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_display_echo(#[string] message: String) -> String {
    message
}

// ============================================================================
// Display Query Operations
// ============================================================================

/// Get all connected monitors
#[weld_op]
#[op2]
#[serde]
pub fn op_display_get_all(state: &mut OpState) -> Result<Vec<MonitorInfo>, DisplayError> {
    debug!("display.get_all");

    let monitors = query_monitors()?;

    // Update cache
    if let Some(display_state) = state.try_borrow_mut::<DisplayState>() {
        display_state.monitors.clear();
        for monitor in &monitors {
            display_state
                .monitors
                .insert(monitor.id.clone(), monitor.clone());
        }
        display_state.last_update_ms = now_ms();
    }

    Ok(monitors)
}

/// Get the primary monitor
#[weld_op]
#[op2]
#[serde]
pub fn op_display_get_primary(state: &mut OpState) -> Result<Option<MonitorInfo>, DisplayError> {
    debug!("display.get_primary");

    let monitors = query_monitors()?;

    // Update cache
    if let Some(display_state) = state.try_borrow_mut::<DisplayState>() {
        display_state.monitors.clear();
        for monitor in &monitors {
            display_state
                .monitors
                .insert(monitor.id.clone(), monitor.clone());
        }
        display_state.last_update_ms = now_ms();
    }

    Ok(monitors.into_iter().find(|m| m.is_primary))
}

/// Get monitor by ID
#[weld_op]
#[op2]
#[serde]
pub fn op_display_get_by_id(
    state: &mut OpState,
    #[string] id: String,
) -> Result<Option<MonitorInfo>, DisplayError> {
    debug!(id = %id, "display.get_by_id");

    // First check cache
    if let Some(display_state) = state.try_borrow::<DisplayState>() {
        if let Some(monitor) = display_state.monitors.get(&id) {
            return Ok(Some(monitor.clone()));
        }
    }

    // Refresh and check again
    let monitors = query_monitors()?;

    // Update cache
    if let Some(display_state) = state.try_borrow_mut::<DisplayState>() {
        display_state.monitors.clear();
        for monitor in &monitors {
            display_state
                .monitors
                .insert(monitor.id.clone(), monitor.clone());
        }
        display_state.last_update_ms = now_ms();
    }

    Ok(monitors.into_iter().find(|m| m.id == id))
}

/// Get monitor at a specific point
#[weld_op]
#[op2]
#[serde]
pub fn op_display_get_at_point(
    state: &mut OpState,
    #[smi] x: i32,
    #[smi] y: i32,
) -> Result<Option<MonitorInfo>, DisplayError> {
    debug!(x = x, y = y, "display.get_at_point");

    let monitors = query_monitors()?;

    // Update cache
    if let Some(display_state) = state.try_borrow_mut::<DisplayState>() {
        display_state.monitors.clear();
        for monitor in &monitors {
            display_state
                .monitors
                .insert(monitor.id.clone(), monitor.clone());
        }
        display_state.last_update_ms = now_ms();
    }

    // Find monitor containing the point
    Ok(monitors.into_iter().find(|m| {
        x >= m.position.x
            && x < m.position.x + m.size.width as i32
            && y >= m.position.y
            && y < m.position.y + m.size.height as i32
    }))
}

/// Get current cursor position
#[weld_op]
#[op2]
#[serde]
pub fn op_display_get_cursor_position(_state: &OpState) -> Result<CursorPosition, DisplayError> {
    debug!("display.get_cursor_position");

    // Platform-specific cursor position retrieval
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // Use AppleScript to get cursor position on macOS
        let output = Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to get the position of the mouse cursor",
            ])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let parts: Vec<&str> = stdout.trim().split(", ").collect();
                if parts.len() == 2 {
                    let x: f64 = parts[0].parse().unwrap_or(0.0);
                    let y: f64 = parts[1].parse().unwrap_or(0.0);
                    return Ok(CursorPosition {
                        x,
                        y,
                        monitor_id: None,
                    });
                }
            }
            Err(e) => {
                warn!("Failed to get cursor position: {}", e);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

        unsafe {
            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_ok() {
                return Ok(CursorPosition {
                    x: point.x as f64,
                    y: point.y as f64,
                    monitor_id: None,
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Use xdotool on Linux
        let output = Command::new("xdotool").args(["getmouselocation"]).output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Format: x:123 y:456 screen:0 window:12345678
                let mut x = 0.0;
                let mut y = 0.0;
                for part in stdout.split_whitespace() {
                    if let Some(val) = part.strip_prefix("x:") {
                        x = val.parse().unwrap_or(0.0);
                    } else if let Some(val) = part.strip_prefix("y:") {
                        y = val.parse().unwrap_or(0.0);
                    }
                }
                return Ok(CursorPosition {
                    x,
                    y,
                    monitor_id: None,
                });
            }
            Err(e) => {
                warn!("Failed to get cursor position: {}", e);
            }
        }
    }

    // Fallback: return zero position
    Ok(CursorPosition {
        x: 0.0,
        y: 0.0,
        monitor_id: None,
    })
}

/// Get number of connected monitors
#[weld_op]
#[op2(fast)]
pub fn op_display_get_count(_state: &OpState) -> Result<u32, DisplayError> {
    debug!("display.get_count");

    let monitors = query_monitors()?;
    Ok(monitors.len() as u32)
}

// ============================================================================
// Subscription Operations
// ============================================================================

/// Subscribe to display change events
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_display_subscribe(
    state: Rc<RefCell<OpState>>,
    #[serde] options: SubscribeOptions,
) -> Result<String, DisplayError> {
    // Validate interval (minimum 500ms to prevent excessive load)
    let interval_ms = options.interval_ms.max(500);

    let subscription_id = {
        let mut s = state.borrow_mut();
        let display_state = s.borrow_mut::<DisplayState>();

        // Check subscription limit
        if display_state.subscriptions.len() >= display_state.max_subscriptions {
            return Err(DisplayError::subscription_limit_exceeded(format!(
                "Maximum {} subscriptions allowed",
                display_state.max_subscriptions
            )));
        }

        let id = format!("display-sub-{}", display_state.next_subscription_id);
        display_state.next_subscription_id += 1;

        let (tx, rx) = mpsc::channel(32);
        let cancel_token = CancellationToken::new();
        let event_count = Arc::new(AtomicU64::new(0));

        let subscription = DisplaySubscription {
            id: id.clone(),
            options: SubscribeOptions { interval_ms },
            sender: tx.clone(),
            receiver: Some(rx),
            event_count: event_count.clone(),
            cancel_token: cancel_token.clone(),
        };

        // Get initial monitor state for comparison
        let initial_monitors: HashMap<String, MonitorInfo> = display_state.monitors.clone();

        display_state.subscriptions.insert(id.clone(), subscription);

        // Spawn background polling task
        let id_clone = id.clone();
        tokio::spawn(async move {
            let mut previous_monitors = initial_monitors;
            let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        trace!("Display subscription {} cancelled", id_clone);
                        break;
                    }
                    _ = ticker.tick() => {
                        // Query current monitors
                        match query_monitors() {
                            Ok(current) => {
                                let current_map: HashMap<String, MonitorInfo> = current
                                    .into_iter()
                                    .map(|m| (m.id.clone(), m))
                                    .collect();

                                // Detect changes
                                let mut events = Vec::new();

                                // Check for new monitors
                                for (id, monitor) in &current_map {
                                    if !previous_monitors.contains_key(id) {
                                        events.push(DisplayEvent::MonitorConnected {
                                            monitor: monitor.clone(),
                                        });
                                    } else if let Some(old) = previous_monitors.get(id) {
                                        let changes = detect_changes(old, monitor);
                                        if !changes.is_empty() {
                                            events.push(DisplayEvent::MonitorChanged {
                                                monitor: monitor.clone(),
                                                changes,
                                            });
                                        }
                                    }
                                }

                                // Check for disconnected monitors
                                for id in previous_monitors.keys() {
                                    if !current_map.contains_key(id) {
                                        events.push(DisplayEvent::MonitorDisconnected {
                                            monitor_id: id.clone(),
                                        });
                                    }
                                }

                                // Send events
                                for event in events {
                                    if tx.send(event).await.is_err() {
                                        debug!("Display subscription {} receiver dropped", id_clone);
                                        break;
                                    }
                                    event_count.fetch_add(1, Ordering::Relaxed);
                                }

                                previous_monitors = current_map;
                            }
                            Err(e) => {
                                warn!("Failed to query monitors in subscription: {}", e);
                            }
                        }
                    }
                }
            }
        });

        id
    };

    debug!(id = %subscription_id, "display.subscribe");
    Ok(subscription_id)
}

/// Unsubscribe from display events
#[weld_op]
#[op2(fast)]
pub fn op_display_unsubscribe(
    state: &mut OpState,
    #[string] subscription_id: String,
) -> Result<(), DisplayError> {
    debug!(id = %subscription_id, "display.unsubscribe");

    let display_state = state.borrow_mut::<DisplayState>();

    if let Some(subscription) = display_state.subscriptions.remove(&subscription_id) {
        subscription.cancel_token.cancel();
        Ok(())
    } else {
        Err(DisplayError::invalid_subscription(subscription_id))
    }
}

/// Get next display event from subscription
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_display_next_event(
    state: Rc<RefCell<OpState>>,
    #[string] subscription_id: String,
) -> Result<Option<DisplayEvent>, DisplayError> {
    // Take receiver temporarily
    let maybe_receiver = {
        let mut s = state.borrow_mut();
        let display_state = s.borrow_mut::<DisplayState>();
        display_state
            .subscriptions
            .get_mut(&subscription_id)
            .and_then(|sub| sub.receiver.take())
    };

    let mut receiver = maybe_receiver
        .ok_or_else(|| DisplayError::invalid_subscription(subscription_id.clone()))?;

    let result = receiver.recv().await;

    // Put receiver back
    {
        let mut s = state.borrow_mut();
        let display_state = s.borrow_mut::<DisplayState>();
        if let Some(sub) = display_state.subscriptions.get_mut(&subscription_id) {
            sub.receiver = Some(receiver);
        }
    }

    Ok(result)
}

/// List active subscriptions
#[weld_op]
#[op2]
#[serde]
pub fn op_display_subscriptions(state: &OpState) -> Vec<SubscriptionInfo> {
    let display_state = state.borrow::<DisplayState>();

    display_state
        .subscriptions
        .values()
        .map(|sub| SubscriptionInfo {
            id: sub.id.clone(),
            interval_ms: sub.options.interval_ms,
            is_active: !sub.cancel_token.is_cancelled(),
            event_count: sub.event_count.load(Ordering::Relaxed),
        })
        .collect()
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn display_extension() -> Extension {
    runtime_display::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = DisplayError::monitor_not_found("test");
        match err {
            DisplayError::MonitorNotFound { code, .. } => {
                assert_eq!(code, DisplayErrorCode::MonitorNotFound as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_size_default() {
        let size = Size::default();
        assert_eq!(size.width, 0);
        assert_eq!(size.height, 0);
    }

    #[test]
    fn test_detect_changes() {
        let old = MonitorInfo {
            id: "test".to_string(),
            name: Some("Test".to_string()),
            position: Position { x: 0, y: 0 },
            size: Size {
                width: 1920,
                height: 1080,
            },
            scale_factor: 1.0,
            is_primary: true,
            refresh_rate_millihertz: Some(60000),
        };

        let mut new = old.clone();
        new.scale_factor = 2.0;
        new.is_primary = false;

        let changes = detect_changes(&old, &new);
        assert!(changes.contains(&MonitorChangeType::ScaleFactor));
        assert!(changes.contains(&MonitorChangeType::Primary));
        assert_eq!(changes.len(), 2);
    }
}
