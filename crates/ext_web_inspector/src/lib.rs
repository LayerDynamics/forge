//! WebView DevTools Inspector Extension for Forge
//!
//! Provides integration with native WebView DevTools (Safari Web Inspector, WebView2 DevTools)
//! by exposing Forge-specific data through custom CDP (Chrome DevTools Protocol) domains.
//!
//! Custom CDP domains:
//! - `Forge.Monitor` - System and runtime metrics
//! - `Forge.Trace` - Span and trace data
//! - `Forge.Signals` - OS signal handling
//! - `Forge.Runtime` - App info, windows, IPC channels
//!
//! Error codes: 9700-9799

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, trace, warn};

// Sub-modules
pub mod bridge;
pub mod cdp;
pub mod platform;

// Re-export key types
pub use cdp::{CdpEvent, CdpMessage, CdpResponse};
pub use platform::PlatformAdapter;

// ============================================================================
// Error Types (Error codes 9700-9799)
// ============================================================================

/// Error codes for web inspector operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WebInspectorErrorCode {
    /// Generic inspector error
    Generic = 9700,
    /// Not connected to inspector
    NotConnected = 9701,
    /// Platform not supported
    PlatformUnsupported = 9702,
    /// Panel injection failed
    InjectionFailed = 9703,
    /// CDP protocol error
    CdpError = 9704,
    /// Bridge communication error
    BridgeError = 9705,
    /// Session not found
    SessionNotFound = 9706,
    /// Domain not enabled
    DomainNotEnabled = 9707,
    /// Invalid CDP message
    InvalidMessage = 9708,
    /// Subscription error
    SubscriptionError = 9709,
}

/// Web inspector extension errors
#[derive(Debug, Error, JsError)]
pub enum WebInspectorError {
    #[error("[{code}] Inspector error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Not connected: {message}")]
    #[class(generic)]
    NotConnected { code: u32, message: String },

    #[error("[{code}] Platform not supported: {message}")]
    #[class(generic)]
    PlatformUnsupported { code: u32, message: String },

    #[error("[{code}] Injection failed: {message}")]
    #[class(generic)]
    InjectionFailed { code: u32, message: String },

    #[error("[{code}] CDP error: {message}")]
    #[class(generic)]
    CdpError { code: u32, message: String },

    #[error("[{code}] Session not found: {message}")]
    #[class(generic)]
    SessionNotFound { code: u32, message: String },

    #[error("[{code}] Domain not enabled: {message}")]
    #[class(generic)]
    DomainNotEnabled { code: u32, message: String },

    #[error("[{code}] Subscription error: {message}")]
    #[class(generic)]
    SubscriptionError { code: u32, message: String },
}

impl WebInspectorError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: WebInspectorErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn not_connected(message: impl Into<String>) -> Self {
        Self::NotConnected {
            code: WebInspectorErrorCode::NotConnected as u32,
            message: message.into(),
        }
    }

    pub fn platform_unsupported(message: impl Into<String>) -> Self {
        Self::PlatformUnsupported {
            code: WebInspectorErrorCode::PlatformUnsupported as u32,
            message: message.into(),
        }
    }

    pub fn injection_failed(message: impl Into<String>) -> Self {
        Self::InjectionFailed {
            code: WebInspectorErrorCode::InjectionFailed as u32,
            message: message.into(),
        }
    }

    pub fn cdp_error(message: impl Into<String>) -> Self {
        Self::CdpError {
            code: WebInspectorErrorCode::CdpError as u32,
            message: message.into(),
        }
    }

    pub fn session_not_found(message: impl Into<String>) -> Self {
        Self::SessionNotFound {
            code: WebInspectorErrorCode::SessionNotFound as u32,
            message: message.into(),
        }
    }

    pub fn domain_not_enabled(message: impl Into<String>) -> Self {
        Self::DomainNotEnabled {
            code: WebInspectorErrorCode::DomainNotEnabled as u32,
            message: message.into(),
        }
    }

    pub fn subscription_error(message: impl Into<String>) -> Self {
        Self::SubscriptionError {
            code: WebInspectorErrorCode::SubscriptionError as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// CDP Types
// ============================================================================

/// CDP domain identifier
#[weld_enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CdpDomain {
    #[serde(rename = "Forge.Monitor")]
    ForgeMonitor,
    #[serde(rename = "Forge.Trace")]
    ForgeTrace,
    #[serde(rename = "Forge.Signals")]
    ForgeSignals,
    #[serde(rename = "Forge.Runtime")]
    ForgeRuntime,
}

impl CdpDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            CdpDomain::ForgeMonitor => "Forge.Monitor",
            CdpDomain::ForgeTrace => "Forge.Trace",
            CdpDomain::ForgeSignals => "Forge.Signals",
            CdpDomain::ForgeRuntime => "Forge.Runtime",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Forge.Monitor" => Some(CdpDomain::ForgeMonitor),
            "Forge.Trace" => Some(CdpDomain::ForgeTrace),
            "Forge.Signals" => Some(CdpDomain::ForgeSignals),
            "Forge.Runtime" => Some(CdpDomain::ForgeRuntime),
            _ => None,
        }
    }
}

// ============================================================================
// Session Types
// ============================================================================

/// Inspector session information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct InspectorSessionInfo {
    /// Window ID this session is attached to
    pub window_id: String,
    /// Whether the session is connected
    pub is_connected: bool,
    /// Whether the custom Forge panel has been injected
    pub panel_injected: bool,
    /// Enabled CDP domains
    pub enabled_domains: Vec<String>,
    /// Session creation timestamp (Unix millis)
    pub created_at_ms: u64,
}

/// Internal session state
struct InspectorSession {
    window_id: String,
    connected: bool,
    panel_injected: bool,
    enabled_domains: HashSet<CdpDomain>,
    created_at_ms: u64,
}

impl InspectorSession {
    fn new(window_id: String) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            window_id,
            connected: false,
            panel_injected: false,
            enabled_domains: HashSet::new(),
            created_at_ms,
        }
    }

    fn to_info(&self) -> InspectorSessionInfo {
        InspectorSessionInfo {
            window_id: self.window_id.clone(),
            is_connected: self.connected,
            panel_injected: self.panel_injected,
            enabled_domains: self
                .enabled_domains
                .iter()
                .map(|d| d.as_str().to_string())
                .collect(),
            created_at_ms: self.created_at_ms,
        }
    }
}

// ============================================================================
// Event Types
// ============================================================================

/// Inspector event for subscriptions
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct InspectorEvent {
    /// Event type/name
    pub event_type: String,
    /// Source domain
    pub domain: String,
    /// Event payload
    pub payload: Value,
    /// Timestamp (Unix millis)
    pub timestamp_ms: u64,
}

/// Event subscription state
struct EventSubscription {
    id: String,
    receiver: Option<mpsc::Receiver<InspectorEvent>>,
    event_count: Arc<AtomicU64>,
}

// ============================================================================
// Aggregated Metrics Types
// ============================================================================

/// Aggregated metrics from all Forge extensions
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct AggregatedMetrics {
    /// System metrics available
    pub system_available: bool,
    /// Runtime metrics available
    pub runtime_available: bool,
    /// Trace data available
    pub trace_available: bool,
    /// Number of active spans (if trace enabled)
    pub active_span_count: u32,
    /// Number of finished spans (if trace enabled)
    pub finished_span_count: u32,
    /// Signal subscriptions active
    pub signal_subscriptions: u32,
    /// Window count
    pub window_count: u32,
    /// IPC channel count
    pub ipc_channel_count: u32,
}

// ============================================================================
// Extension Info
// ============================================================================

#[weld_struct]
#[derive(Serialize)]
pub struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
    supported_domains: Vec<&'static str>,
}

// ============================================================================
// State Management
// ============================================================================

/// Web inspector state stored in OpState
pub struct WebInspectorState {
    /// Active inspector sessions per window
    sessions: HashMap<String, InspectorSession>,
    /// Event broadcast sender
    event_tx: broadcast::Sender<InspectorEvent>,
    /// Event subscriptions
    subscriptions: HashMap<String, EventSubscription>,
    /// Next subscription ID
    next_subscription_id: u64,
    /// Platform name (for info)
    platform: String,
}

impl Default for WebInspectorState {
    fn default() -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            sessions: HashMap::new(),
            event_tx,
            subscriptions: HashMap::new(),
            next_subscription_id: 1,
            platform: detect_platform(),
        }
    }
}

/// Detect current platform
fn detect_platform() -> String {
    #[cfg(target_os = "macos")]
    {
        "WebKit".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "WebView2".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "WebKitGTK".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Unknown".to_string()
    }
}

/// Initialize web inspector state in OpState
pub fn init_web_inspector_state(op_state: &mut OpState) {
    let state = WebInspectorState::default();
    debug!(platform = %state.platform, "Initializing web inspector state");
    op_state.put(state);
}

// ============================================================================
// Operations - Extension Info
// ============================================================================

#[weld_op]
#[op2]
#[serde]
fn op_web_inspector_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_web_inspector",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
        supported_domains: vec![
            "Forge.Monitor",
            "Forge.Trace",
            "Forge.Signals",
            "Forge.Runtime",
        ],
    }
}

// ============================================================================
// Operations - Session Management
// ============================================================================

/// Connect to inspector for a window
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_web_inspector_connect(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<InspectorSessionInfo, WebInspectorError> {
    let mut s = state.borrow_mut();
    let inspector_state = s.borrow_mut::<WebInspectorState>();

    // Create or get existing session
    let session = inspector_state
        .sessions
        .entry(window_id.clone())
        .or_insert_with(|| InspectorSession::new(window_id.clone()));

    // Mark as connected
    session.connected = true;
    debug!("Connected inspector session for window: {}", window_id);

    Ok(session.to_info())
}

/// Disconnect from inspector for a window
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_disconnect(
    state: &mut OpState,
    #[string] window_id: String,
) -> Result<(), WebInspectorError> {
    let inspector_state = state.borrow_mut::<WebInspectorState>();

    if let Some(session) = inspector_state.sessions.get_mut(&window_id) {
        session.connected = false;
        session.enabled_domains.clear();
        debug!("Disconnected inspector session for window: {}", window_id);
        Ok(())
    } else {
        Err(WebInspectorError::session_not_found(format!(
            "No session for window: {}",
            window_id
        )))
    }
}

/// Check if inspector is connected for a window
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_is_connected(state: &OpState, #[string] window_id: String) -> bool {
    let inspector_state = state.borrow::<WebInspectorState>();
    inspector_state
        .sessions
        .get(&window_id)
        .map(|s| s.connected)
        .unwrap_or(false)
}

/// Get all active sessions
#[weld_op]
#[op2]
#[serde]
pub fn op_web_inspector_sessions(state: &OpState) -> Vec<InspectorSessionInfo> {
    let inspector_state = state.borrow::<WebInspectorState>();
    inspector_state
        .sessions
        .values()
        .map(|s| s.to_info())
        .collect()
}

// ============================================================================
// Operations - CDP Communication
// ============================================================================

/// Send a CDP message to a custom Forge domain
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_web_inspector_send_cdp(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
    #[string] method: String,
    #[serde] params: Option<serde_json::Value>,
) -> Result<serde_json::Value, WebInspectorError> {
    // Parse domain and method
    trace!(method = %method, window_id = %window_id, "Parsing CDP method");
    let parts: Vec<&str> = method.splitn(2, '.').collect();
    if parts.len() != 2 {
        warn!(method = %method, "Invalid CDP method format - expected Domain.method");
        return Err(WebInspectorError::cdp_error(format!(
            "Invalid method format: {}. Expected Domain.method",
            method
        )));
    }

    let domain_name = parts[0];
    let method_name = parts[1];

    // Check if this is a Forge domain
    let domain = CdpDomain::from_str(&format!(
        "Forge.{}",
        domain_name.strip_prefix("Forge.").unwrap_or(domain_name)
    ))
    .or_else(|| CdpDomain::from_str(domain_name));

    if domain.is_none() {
        warn!(domain = %domain_name, "Unknown Forge domain requested");
        return Err(WebInspectorError::cdp_error(format!(
            "Unknown Forge domain: {}",
            domain_name
        )));
    }

    let domain = domain.unwrap();
    trace!(domain = ?domain, method = %method_name, "Routing CDP command");

    // Check session exists and domain is enabled
    {
        let s = state.borrow();
        let inspector_state = s.borrow::<WebInspectorState>();

        let session = inspector_state.sessions.get(&window_id).ok_or_else(|| {
            WebInspectorError::session_not_found(format!("No session for window: {}", window_id))
        })?;

        if !session.connected {
            return Err(WebInspectorError::not_connected(format!(
                "Session not connected for window: {}",
                window_id
            )));
        }

        // Allow enable/disable without checking if domain is enabled
        if method_name != "enable"
            && method_name != "disable"
            && !session.enabled_domains.contains(&domain)
        {
            return Err(WebInspectorError::domain_not_enabled(format!(
                "Domain {} not enabled for window {}",
                domain.as_str(),
                window_id
            )));
        }
    }

    // Route to appropriate handler based on domain and method
    let result = cdp::router::route_cdp_command(&state, &domain, method_name, params).await?;

    Ok(result)
}

/// Enable a CDP domain for a session
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_enable_domain(
    state: &mut OpState,
    #[string] window_id: String,
    #[string] domain: String,
) -> Result<bool, WebInspectorError> {
    let inspector_state = state.borrow_mut::<WebInspectorState>();

    let session = inspector_state
        .sessions
        .get_mut(&window_id)
        .ok_or_else(|| {
            WebInspectorError::session_not_found(format!("No session for window: {}", window_id))
        })?;

    let cdp_domain = CdpDomain::from_str(&domain)
        .ok_or_else(|| WebInspectorError::cdp_error(format!("Unknown domain: {}", domain)))?;

    let was_new = session.enabled_domains.insert(cdp_domain);
    debug!(
        "Enabled domain {} for window {} (was_new: {})",
        domain, window_id, was_new
    );

    Ok(was_new)
}

/// Disable a CDP domain for a session
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_disable_domain(
    state: &mut OpState,
    #[string] window_id: String,
    #[string] domain: String,
) -> Result<bool, WebInspectorError> {
    let inspector_state = state.borrow_mut::<WebInspectorState>();

    let session = inspector_state
        .sessions
        .get_mut(&window_id)
        .ok_or_else(|| {
            WebInspectorError::session_not_found(format!("No session for window: {}", window_id))
        })?;

    let cdp_domain = CdpDomain::from_str(&domain)
        .ok_or_else(|| WebInspectorError::cdp_error(format!("Unknown domain: {}", domain)))?;

    let was_present = session.enabled_domains.remove(&cdp_domain);
    debug!(
        "Disabled domain {} for window {} (was_present: {})",
        domain, window_id, was_present
    );

    Ok(was_present)
}

// ============================================================================
// Operations - Panel Injection
// ============================================================================

/// Inject the Forge DevTools panel into the native inspector
#[weld_op(async)]
#[op2(async)]
pub async fn op_web_inspector_inject_panel(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, WebInspectorError> {
    let mut s = state.borrow_mut();
    let inspector_state = s.borrow_mut::<WebInspectorState>();

    let session = inspector_state
        .sessions
        .get_mut(&window_id)
        .ok_or_else(|| {
            WebInspectorError::session_not_found(format!("No session for window: {}", window_id))
        })?;

    if session.panel_injected {
        debug!("Panel already injected for window: {}", window_id);
        return Ok(false);
    }

    // Platform-specific injection would happen here
    // For now, just mark as injected
    session.panel_injected = true;
    debug!("Injected Forge panel for window: {}", window_id);

    Ok(true)
}

/// Check if panel is injected for a window
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_is_panel_injected(state: &OpState, #[string] window_id: String) -> bool {
    let inspector_state = state.borrow::<WebInspectorState>();
    inspector_state
        .sessions
        .get(&window_id)
        .map(|s| s.panel_injected)
        .unwrap_or(false)
}

// ============================================================================
// Operations - Metrics
// ============================================================================

/// Get aggregated metrics from all Forge extensions
#[weld_op]
#[op2]
#[serde]
pub fn op_web_inspector_get_metrics(state: &OpState) -> AggregatedMetrics {
    // This would aggregate metrics from ext_monitor, ext_trace, ext_signals, etc.
    // For now, return a placeholder
    let inspector_state = state.borrow::<WebInspectorState>();

    AggregatedMetrics {
        system_available: true,
        runtime_available: true,
        trace_available: true,
        active_span_count: 0,
        finished_span_count: 0,
        signal_subscriptions: 0,
        window_count: inspector_state.sessions.len() as u32,
        ipc_channel_count: 0,
    }
}

// ============================================================================
// Operations - Event Subscription
// ============================================================================

/// Subscribe to inspector events
#[weld_op]
#[op2]
#[string]
pub fn op_web_inspector_subscribe_events(state: &mut OpState) -> Result<String, WebInspectorError> {
    let inspector_state = state.borrow_mut::<WebInspectorState>();

    let id = format!("evt-{}", inspector_state.next_subscription_id);
    inspector_state.next_subscription_id += 1;

    let (tx, rx) = mpsc::channel(64);

    // Subscribe to broadcast
    let mut broadcast_rx = inspector_state.event_tx.subscribe();

    // Spawn task to forward broadcast events to the mpsc channel
    let event_count = Arc::new(AtomicU64::new(0));
    let count_clone = event_count.clone();

    tokio::spawn(async move {
        while let Ok(event) = broadcast_rx.recv().await {
            if tx.send(event).await.is_err() {
                break;
            }
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });

    inspector_state.subscriptions.insert(
        id.clone(),
        EventSubscription {
            id: id.clone(),
            receiver: Some(rx),
            event_count,
        },
    );

    Ok(id)
}

/// Get next event from subscription
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_web_inspector_next_event(
    state: Rc<RefCell<OpState>>,
    #[string] subscription_id: String,
) -> Result<Option<InspectorEvent>, WebInspectorError> {
    // Take receiver temporarily
    let maybe_receiver = {
        let mut s = state.borrow_mut();
        let inspector_state = s.borrow_mut::<WebInspectorState>();
        inspector_state
            .subscriptions
            .get_mut(&subscription_id)
            .and_then(|sub| sub.receiver.take())
    };

    let mut receiver = maybe_receiver.ok_or_else(|| {
        WebInspectorError::subscription_error(format!(
            "Subscription not found: {}",
            subscription_id
        ))
    })?;

    let result = receiver.recv().await;

    // Put receiver back
    {
        let mut s = state.borrow_mut();
        let inspector_state = s.borrow_mut::<WebInspectorState>();
        if let Some(sub) = inspector_state.subscriptions.get_mut(&subscription_id) {
            sub.receiver = Some(receiver);
        }
    }

    Ok(result)
}

/// Unsubscribe from inspector events
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_unsubscribe_events(
    state: &mut OpState,
    #[string] subscription_id: String,
) -> Result<(), WebInspectorError> {
    let inspector_state = state.borrow_mut::<WebInspectorState>();

    if let Some(subscription) = inspector_state.subscriptions.remove(&subscription_id) {
        // Log subscription stats using the id and event_count fields
        let total_events = subscription
            .event_count
            .load(std::sync::atomic::Ordering::Relaxed);
        debug!(
            subscription_id = %subscription.id,
            total_events = total_events,
            "Unsubscribed from events"
        );
        Ok(())
    } else {
        Err(WebInspectorError::subscription_error(format!(
            "Subscription not found: {}",
            subscription_id
        )))
    }
}

// ============================================================================
// Operations - Real-time Event Streaming
// ============================================================================

/// Emit a metric update event from current bridge data
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_emit_metrics_update(state: &mut OpState) -> Result<(), WebInspectorError> {
    use crate::bridge::{
        DebuggerBridge, ExtensionBridge, MonitorBridge, SignalsBridge, TraceBridge,
    };

    // Collect metrics from all bridges
    let monitor_bridge = MonitorBridge::new();
    let trace_bridge = TraceBridge::new();
    let signals_bridge = SignalsBridge::new();
    let debugger_bridge = DebuggerBridge::new();

    let monitor_summary = monitor_bridge.summary(state);
    let trace_summary = trace_bridge.summary(state);
    let signals_summary = signals_bridge.summary(state);
    let debugger_summary = debugger_bridge.summary(state);

    // Create combined metrics event
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let metrics_payload = serde_json::json!({
        "monitor": monitor_summary,
        "trace": trace_summary,
        "signals": signals_summary,
        "debugger": debugger_summary,
    });

    let event = InspectorEvent {
        event_type: "Forge.metricsUpdate".to_string(),
        domain: "Forge.Monitor".to_string(),
        payload: metrics_payload,
        timestamp_ms,
    };

    // Emit the event
    let inspector_state = state.borrow::<WebInspectorState>();
    inspector_state.emit_event(event);

    Ok(())
}

/// Emit a span event (for trace streaming)
#[weld_op]
#[op2(fast)]
pub fn op_web_inspector_emit_span_event(
    state: &mut OpState,
    #[string] event_type: String,
    #[string] span_id: String,
    #[string] span_name: String,
) -> Result<(), WebInspectorError> {
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let payload = serde_json::json!({
        "spanId": span_id,
        "spanName": span_name,
        "eventType": event_type,
    });

    let event = InspectorEvent {
        event_type: format!("Forge.Trace.{}", event_type),
        domain: "Forge.Trace".to_string(),
        payload,
        timestamp_ms,
    };

    let inspector_state = state.borrow::<WebInspectorState>();
    inspector_state.emit_event(event);

    Ok(())
}

/// Get aggregated summary from all bridges (for dashboard updates)
#[weld_op]
#[op2]
#[serde]
pub fn op_web_inspector_get_all_summaries(
    state: &mut OpState,
) -> Result<serde_json::Value, WebInspectorError> {
    use crate::bridge::{
        DebuggerBridge, ExtensionBridge, MonitorBridge, SignalsBridge, TraceBridge,
    };

    let monitor_bridge = MonitorBridge::new();
    let trace_bridge = TraceBridge::new();
    let signals_bridge = SignalsBridge::new();
    let debugger_bridge = DebuggerBridge::new();

    Ok(serde_json::json!({
        "monitor": {
            "available": monitor_bridge.is_available(state),
            "summary": monitor_bridge.summary(state)
        },
        "trace": {
            "available": trace_bridge.is_available(state),
            "summary": trace_bridge.summary(state)
        },
        "signals": {
            "available": signals_bridge.is_available(state),
            "summary": signals_bridge.summary(state)
        },
        "debugger": {
            "available": debugger_bridge.is_available(state),
            "summary": debugger_bridge.summary(state)
        }
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

impl WebInspectorState {
    /// Emit an event to all subscribers
    pub fn emit_event(&self, event: InspectorEvent) {
        if self.event_tx.receiver_count() > 0 {
            let _ = self.event_tx.send(event);
        }
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn web_inspector_extension() -> Extension {
    runtime_web_inspector::ext()
}
