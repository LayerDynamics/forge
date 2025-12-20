//! Extension bridges for aggregating data from other Forge extensions.
//!
//! Each bridge provides a type-safe interface to query data from a specific
//! extension (monitor, trace, signals, debugger) and convert it to CDP-compatible
//! formats for the web inspector.
//!
//! ## State Access Pattern
//! Extensions store state in `OpState` using `state.put<T>()`. Bridges access
//! this state using `state.try_borrow::<T>()` which returns `Option<&T>`,
//! allowing graceful handling when an extension isn't loaded.

use std::sync::Arc;

use async_trait::async_trait;
use deno_core::OpState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, trace, warn};

use crate::{CdpDomain, WebInspectorError};

// Re-export extension types we need
pub use ext_monitor::{
    CpuUsage as MonitorCpuUsage, DiskUsage as MonitorDiskUsage, HeapStats as MonitorHeapStats,
    MemoryUsage as MonitorMemoryUsage, MetricSnapshot, MonitorState,
    NetworkStats as MonitorNetworkStats, ProcessInfo as MonitorProcessInfo,
    RuntimeMetrics as MonitorRuntimeMetrics, SubscriptionInfo as MonitorSubscriptionInfo,
    WebViewStats as MonitorWebViewStats,
};

pub use ext_debugger::{
    Breakpoint as DebuggerBreakpoint, CallFrame as DebuggerCallFrame, DebuggerState,
    ScriptInfo as DebuggerScriptInfo,
};

pub use ext_signals::SignalsState;
pub use ext_trace::{SpanRecord as TraceSpanRecord, TraceState};

// ============================================================================
// Bridge Trait
// ============================================================================

/// Trait for extension bridges that provide data to the web inspector.
///
/// Each bridge encapsulates access to a specific extension's state and
/// converts the data to CDP-compatible JSON format.
#[async_trait(?Send)]
pub trait ExtensionBridge {
    /// The CDP domain this bridge serves
    fn domain(&self) -> CdpDomain;

    /// Human-readable name of this bridge
    fn name(&self) -> &'static str;

    /// Check if the underlying extension is available
    fn is_available(&self, state: &OpState) -> bool;

    /// Get the current status of the extension
    fn status(&self, state: &OpState) -> BridgeStatus;

    /// Get a summary of the bridge's data (for overview panels)
    fn summary(&self, state: &OpState) -> Value;
}

/// Status of an extension bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    /// Whether the extension is loaded
    pub loaded: bool,
    /// Whether the extension is active
    pub active: bool,
    /// Optional status message
    pub message: Option<String>,
    /// Extension version
    pub version: Option<String>,
}

impl Default for BridgeStatus {
    fn default() -> Self {
        Self {
            loaded: false,
            active: false,
            message: None,
            version: None,
        }
    }
}

// ============================================================================
// Bridge Collection
// ============================================================================

/// Collection of all extension bridges
pub struct ExtensionBridges {
    pub monitor: MonitorBridge,
    pub trace: TraceBridge,
    pub signals: SignalsBridge,
    pub debugger: DebuggerBridge,
}

impl Default for ExtensionBridges {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionBridges {
    pub fn new() -> Self {
        debug!("Creating ExtensionBridges collection");
        Self {
            monitor: MonitorBridge::new(),
            trace: TraceBridge::new(),
            signals: SignalsBridge::new(),
            debugger: DebuggerBridge::new(),
        }
    }

    /// Get all bridges as a slice
    pub fn all(&self) -> Vec<&dyn ExtensionBridge> {
        vec![&self.monitor, &self.trace, &self.signals, &self.debugger]
    }

    /// Get bridge by domain
    pub fn get(&self, domain: &CdpDomain) -> Option<&dyn ExtensionBridge> {
        match domain {
            CdpDomain::ForgeMonitor => Some(&self.monitor),
            CdpDomain::ForgeTrace => Some(&self.trace),
            CdpDomain::ForgeSignals => Some(&self.signals),
            CdpDomain::ForgeRuntime => None, // Runtime uses multiple bridges
        }
    }

    /// Get summary of all bridges
    pub fn summary(&self, state: &OpState) -> Value {
        trace!("Collecting summary from all extension bridges");
        serde_json::json!({
            "monitor": self.monitor.summary(state),
            "trace": self.trace.summary(state),
            "signals": self.signals.summary(state),
            "debugger": self.debugger.summary(state),
        })
    }
}

// ============================================================================
// Monitor Bridge (ext_monitor) - REAL IMPLEMENTATION
// ============================================================================

/// Bridge to ext_monitor for system and runtime metrics.
///
/// This bridge directly accesses the `MonitorState` stored in `OpState`
/// to provide real-time CPU, memory, disk, network, and runtime metrics.
pub struct MonitorBridge {
    // Configuration
    poll_interval_ms: u64,
}

impl MonitorBridge {
    pub fn new() -> Self {
        Self {
            poll_interval_ms: 1000,
        }
    }

    /// Get CPU metrics from ext_monitor's MonitorState
    pub fn get_cpu(&self, state: &OpState) -> Result<CpuMetrics, WebInspectorError> {
        trace!("Fetching CPU metrics from MonitorState");
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            let cpus = monitor_state.system.cpus();
            let per_core: Vec<f64> = cpus.iter().map(|cpu| cpu.cpu_usage() as f64).collect();
            let total_percent = if per_core.is_empty() {
                0.0
            } else {
                per_core.iter().sum::<f64>() / per_core.len() as f64
            };

            Ok(CpuMetrics {
                total_percent,
                per_core,
                core_count: cpus.len() as u32,
            })
        } else {
            warn!("MonitorState not available in OpState");
            Ok(CpuMetrics::default())
        }
    }

    /// Get memory metrics from ext_monitor's MonitorState
    pub fn get_memory(&self, state: &OpState) -> Result<MemoryMetrics, WebInspectorError> {
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            Ok(MemoryMetrics {
                total_bytes: monitor_state.system.total_memory(),
                used_bytes: monitor_state.system.used_memory(),
                free_bytes: monitor_state.system.free_memory(),
                available_bytes: monitor_state.system.available_memory(),
            })
        } else {
            warn!("MonitorState not available in OpState");
            Ok(MemoryMetrics::default())
        }
    }

    /// Get runtime metrics from ext_monitor
    pub fn get_runtime(&self, state: &OpState) -> Result<RuntimeMetrics, WebInspectorError> {
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            Ok(RuntimeMetrics {
                pending_ops_count: 0, // Would need JsRuntime access
                event_loop_latency_us: monitor_state.latency_measurer.get_latency_us(),
                uptime_secs: monitor_state.latency_measurer.get_uptime_secs(),
            })
        } else {
            Ok(RuntimeMetrics::default())
        }
    }

    /// Get disk usage from ext_monitor
    pub fn get_disks(&self, state: &OpState) -> Result<Vec<DiskInfo>, WebInspectorError> {
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            let disks: Vec<DiskInfo> = monitor_state
                .disks
                .iter()
                .map(|disk| DiskInfo {
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    device: disk.name().to_string_lossy().to_string(),
                    filesystem: disk.file_system().to_string_lossy().to_string(),
                    total_bytes: disk.total_space(),
                    used_bytes: disk.total_space().saturating_sub(disk.available_space()),
                    free_bytes: disk.available_space(),
                })
                .collect();
            Ok(disks)
        } else {
            Ok(vec![])
        }
    }

    /// Get network stats from ext_monitor
    pub fn get_network(&self, state: &OpState) -> Result<Vec<NetworkInfo>, WebInspectorError> {
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            let networks: Vec<NetworkInfo> = monitor_state
                .networks
                .iter()
                .map(|(name, network)| NetworkInfo {
                    interface: name.clone(),
                    bytes_sent: network.total_transmitted(),
                    bytes_recv: network.total_received(),
                    packets_sent: network.total_packets_transmitted(),
                    packets_recv: network.total_packets_received(),
                })
                .collect();
            Ok(networks)
        } else {
            Ok(vec![])
        }
    }

    /// Get subscription count from ext_monitor
    pub fn get_subscription_count(&self, state: &OpState) -> usize {
        if let Some(monitor_state) = state.try_borrow::<MonitorState>() {
            monitor_state.subscription_count()
        } else {
            0
        }
    }
}

impl Default for MonitorBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl ExtensionBridge for MonitorBridge {
    fn domain(&self) -> CdpDomain {
        CdpDomain::ForgeMonitor
    }

    fn name(&self) -> &'static str {
        "Monitor"
    }

    fn is_available(&self, state: &OpState) -> bool {
        state.try_borrow::<MonitorState>().is_some()
    }

    fn status(&self, state: &OpState) -> BridgeStatus {
        let loaded = self.is_available(state);
        BridgeStatus {
            loaded,
            active: loaded,
            message: if loaded {
                None
            } else {
                Some("MonitorState not initialized".to_string())
            },
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    fn summary(&self, state: &OpState) -> Value {
        let available = self.is_available(state);
        let cpu = self.get_cpu(state).unwrap_or_default();
        let memory = self.get_memory(state).unwrap_or_default();
        let runtime = self.get_runtime(state).unwrap_or_default();
        let subscription_count = self.get_subscription_count(state);

        serde_json::json!({
            "available": available,
            "cpuPercent": cpu.total_percent,
            "coreCount": cpu.core_count,
            "memoryUsed": memory.used_bytes,
            "memoryTotal": memory.total_bytes,
            "memoryPercent": if memory.total_bytes > 0 {
                (memory.used_bytes as f64 / memory.total_bytes as f64) * 100.0
            } else {
                0.0
            },
            "eventLoopLatencyUs": runtime.event_loop_latency_us,
            "uptimeSecs": runtime.uptime_secs,
            "activeSubscriptions": subscription_count,
            "pollIntervalMs": self.poll_interval_ms,
        })
    }
}

// ============================================================================
// Trace Bridge (ext_trace)
// ============================================================================

/// Bridge to ext_trace for span and trace data.
///
/// Connects to TraceState to provide real-time span tracking information
/// for the web inspector. Uses the public accessor methods on TraceState.
pub struct TraceBridge;

impl TraceBridge {
    pub fn new() -> Self {
        Self
    }

    /// Get active spans count from TraceState
    pub fn get_active_span_count(&self, state: &OpState) -> usize {
        if let Some(trace_state) = state.try_borrow::<TraceState>() {
            trace_state.active_count()
        } else {
            0
        }
    }

    /// Get finished spans count from TraceState
    pub fn get_finished_span_count(&self, state: &OpState) -> usize {
        if let Some(trace_state) = state.try_borrow::<TraceState>() {
            trace_state.finished_count()
        } else {
            0
        }
    }

    /// Get both active and finished span counts
    pub fn span_count(&self, state: &OpState) -> (usize, usize) {
        (
            self.get_active_span_count(state),
            self.get_finished_span_count(state),
        )
    }

    /// Get active span info (id, name pairs)
    pub fn get_active_spans(&self, state: &OpState) -> Vec<(u64, String)> {
        if let Some(trace_state) = state.try_borrow::<TraceState>() {
            trace_state.active_spans()
        } else {
            vec![]
        }
    }

    /// Get finished span records (without clearing buffer)
    pub fn get_finished_spans(&self, state: &OpState) -> Vec<TraceSpanRecord> {
        if let Some(trace_state) = state.try_borrow::<TraceState>() {
            trace_state.finished_spans().to_vec()
        } else {
            vec![]
        }
    }
}

impl Default for TraceBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl ExtensionBridge for TraceBridge {
    fn domain(&self) -> CdpDomain {
        CdpDomain::ForgeTrace
    }

    fn name(&self) -> &'static str {
        "Trace"
    }

    fn is_available(&self, state: &OpState) -> bool {
        state.try_borrow::<TraceState>().is_some()
    }

    fn status(&self, state: &OpState) -> BridgeStatus {
        let available = self.is_available(state);
        BridgeStatus {
            loaded: available,
            active: available,
            message: if available {
                None
            } else {
                Some("TraceState not initialized".to_string())
            },
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    fn summary(&self, state: &OpState) -> Value {
        let (active, finished) = self.span_count(state);
        let active_spans = self.get_active_spans(state);
        serde_json::json!({
            "available": self.is_available(state),
            "activeSpans": active,
            "finishedSpans": finished,
            "activeSpanNames": active_spans.iter().map(|(_, name)| name.clone()).collect::<Vec<_>>(),
        })
    }
}

// ============================================================================
// Signals Bridge (ext_signals)
// ============================================================================

/// Bridge to ext_signals for OS signal handling.
///
/// Connects to SignalsState to provide real-time signal subscription info
/// and platform-specific signal support for the web inspector.
pub struct SignalsBridge;

impl SignalsBridge {
    pub fn new() -> Self {
        Self
    }

    /// Get supported signals for current platform
    pub fn get_supported(&self) -> Vec<SignalInfo> {
        #[cfg(unix)]
        {
            vec![
                SignalInfo {
                    name: "SIGINT".to_string(),
                    number: 2,
                    description: "Interrupt".to_string(),
                },
                SignalInfo {
                    name: "SIGTERM".to_string(),
                    number: 15,
                    description: "Terminate".to_string(),
                },
                SignalInfo {
                    name: "SIGHUP".to_string(),
                    number: 1,
                    description: "Hangup".to_string(),
                },
                SignalInfo {
                    name: "SIGQUIT".to_string(),
                    number: 3,
                    description: "Quit".to_string(),
                },
                SignalInfo {
                    name: "SIGUSR1".to_string(),
                    number: 10,
                    description: "User signal 1".to_string(),
                },
                SignalInfo {
                    name: "SIGUSR2".to_string(),
                    number: 12,
                    description: "User signal 2".to_string(),
                },
                SignalInfo {
                    name: "SIGALRM".to_string(),
                    number: 14,
                    description: "Alarm".to_string(),
                },
                SignalInfo {
                    name: "SIGCHLD".to_string(),
                    number: 17,
                    description: "Child".to_string(),
                },
                SignalInfo {
                    name: "SIGPIPE".to_string(),
                    number: 13,
                    description: "Pipe".to_string(),
                },
            ]
        }
        #[cfg(windows)]
        {
            vec![
                SignalInfo {
                    name: "CTRL_C".to_string(),
                    number: 0,
                    description: "Ctrl+C".to_string(),
                },
                SignalInfo {
                    name: "CTRL_BREAK".to_string(),
                    number: 1,
                    description: "Ctrl+Break".to_string(),
                },
            ]
        }
        #[cfg(not(any(unix, windows)))]
        {
            vec![]
        }
    }

    /// Get active signal subscription count from SignalsState
    pub fn get_subscription_count(&self, state: &OpState) -> usize {
        if let Some(signals_state) = state.try_borrow::<SignalsState>() {
            signals_state.subscription_count()
        } else {
            0
        }
    }

    /// Get subscription IDs
    pub fn get_subscription_ids(&self, state: &OpState) -> Vec<u64> {
        if let Some(signals_state) = state.try_borrow::<SignalsState>() {
            signals_state.subscription_ids()
        } else {
            vec![]
        }
    }
}

impl Default for SignalsBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl ExtensionBridge for SignalsBridge {
    fn domain(&self) -> CdpDomain {
        CdpDomain::ForgeSignals
    }

    fn name(&self) -> &'static str {
        "Signals"
    }

    fn is_available(&self, state: &OpState) -> bool {
        // Signals extension is available on Unix platforms and if SignalsState is initialized
        cfg!(unix) && state.try_borrow::<SignalsState>().is_some()
    }

    fn status(&self, state: &OpState) -> BridgeStatus {
        let state_available = state.try_borrow::<SignalsState>().is_some();
        let platform_supported = cfg!(unix);
        BridgeStatus {
            loaded: state_available,
            active: platform_supported && state_available,
            message: if !platform_supported {
                Some("Signals not supported on this platform".to_string())
            } else if !state_available {
                Some("SignalsState not initialized".to_string())
            } else {
                None
            },
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    fn summary(&self, state: &OpState) -> Value {
        let supported = self.get_supported();
        let subscription_count = self.get_subscription_count(state);
        let subscription_ids = self.get_subscription_ids(state);

        serde_json::json!({
            "available": self.is_available(state),
            "supportedSignals": supported.len(),
            "signals": supported,
            "activeSubscriptions": subscription_count,
            "subscriptionIds": subscription_ids,
        })
    }
}

// ============================================================================
// Debugger Bridge (ext_debugger) - REAL IMPLEMENTATION
// ============================================================================

/// Bridge to ext_debugger for debugging features.
///
/// This bridge accesses the `Arc<DebuggerState>` stored in `OpState` to
/// provide real-time debugging information including connection status,
/// breakpoints, scripts, and call frames using the public accessor methods.
pub struct DebuggerBridge;

impl DebuggerBridge {
    pub fn new() -> Self {
        Self
    }

    /// Get debugger state if available
    fn get_state(&self, state: &OpState) -> Option<Arc<DebuggerState>> {
        state.try_borrow::<Arc<DebuggerState>>().cloned()
    }

    /// Check if debugger is connected
    pub fn is_connected(&self, state: &OpState) -> bool {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.is_connected()
        } else {
            false
        }
    }

    /// Check if debugger is enabled
    pub fn is_enabled(&self, state: &OpState) -> bool {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.is_enabled()
        } else {
            false
        }
    }

    /// Check if currently paused
    pub fn is_paused(&self, state: &OpState) -> bool {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.is_paused()
        } else {
            false
        }
    }

    /// Get breakpoint count
    pub fn get_breakpoint_count(&self, state: &OpState) -> usize {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.breakpoint_count()
        } else {
            0
        }
    }

    /// Get breakpoint IDs
    pub fn get_breakpoint_ids(&self, state: &OpState) -> Vec<String> {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.breakpoint_ids()
        } else {
            vec![]
        }
    }

    /// Get script count
    pub fn get_script_count(&self, state: &OpState) -> usize {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.script_count()
        } else {
            0
        }
    }

    /// Get script info snapshot
    pub fn get_scripts(&self, state: &OpState) -> Vec<DebuggerScriptInfo> {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.scripts_snapshot()
        } else {
            vec![]
        }
    }

    /// Get current call frames (if paused)
    pub fn get_call_frames(&self, state: &OpState) -> Vec<DebuggerCallFrame> {
        if let Some(debugger_state) = self.get_state(state) {
            debugger_state.call_frames_snapshot()
        } else {
            vec![]
        }
    }
}

impl Default for DebuggerBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl ExtensionBridge for DebuggerBridge {
    fn domain(&self) -> CdpDomain {
        CdpDomain::ForgeRuntime // Debugger reports through Runtime domain
    }

    fn name(&self) -> &'static str {
        "Debugger"
    }

    fn is_available(&self, state: &OpState) -> bool {
        self.get_state(state).is_some()
    }

    fn status(&self, state: &OpState) -> BridgeStatus {
        let loaded = self.is_available(state);
        let connected = self.is_connected(state);
        let enabled = self.is_enabled(state);

        BridgeStatus {
            loaded,
            active: connected && enabled,
            message: if !loaded {
                Some("DebuggerState not initialized".to_string())
            } else if !connected {
                Some("Debugger not connected".to_string())
            } else if !enabled {
                Some("Debugger not enabled".to_string())
            } else {
                None
            },
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    fn summary(&self, state: &OpState) -> Value {
        let available = self.is_available(state);
        let connected = self.is_connected(state);
        let enabled = self.is_enabled(state);
        let paused = self.is_paused(state);
        let breakpoint_count = self.get_breakpoint_count(state);
        let breakpoint_ids = self.get_breakpoint_ids(state);
        let script_count = self.get_script_count(state);
        let call_frame_count = if paused {
            self.get_call_frames(state).len()
        } else {
            0
        };

        serde_json::json!({
            "available": available,
            "connected": connected,
            "enabled": enabled,
            "paused": paused,
            "breakpoints": breakpoint_count,
            "breakpointIds": breakpoint_ids,
            "scripts": script_count,
            "callFrames": call_frame_count,
        })
    }
}

// ============================================================================
// Data Types
// ============================================================================

/// CPU metrics for web inspector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub total_percent: f64,
    pub per_core: Vec<f64>,
    pub core_count: u32,
}

/// Memory metrics for web inspector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
}

/// Runtime metrics for web inspector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub pending_ops_count: u32,
    pub event_loop_latency_us: u64,
    pub uptime_secs: u64,
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packets_sent: u64,
    pub packets_recv: u64,
}

/// Signal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInfo {
    pub name: String,
    pub number: i32,
    pub description: String,
}

/// Span information from trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    pub id: String,
    pub name: String,
    pub target: String,
    pub level: String,
    pub start_time_us: u64,
    pub end_time_us: Option<u64>,
    pub parent_id: Option<String>,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    pub id: String,
    pub url: String,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
    pub enabled: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // BridgeStatus Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bridge_status_default() {
        let status = BridgeStatus::default();
        assert!(!status.loaded);
        assert!(!status.active);
        assert!(status.message.is_none());
        assert!(status.version.is_none());
    }

    #[test]
    fn test_bridge_status_with_values() {
        let status = BridgeStatus {
            loaded: true,
            active: true,
            message: Some("Ready".to_string()),
            version: Some("1.0.0".to_string()),
        };
        assert!(status.loaded);
        assert!(status.active);
        assert_eq!(status.message, Some("Ready".to_string()));
        assert_eq!(status.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_bridge_status_serialization() {
        let status = BridgeStatus {
            loaded: true,
            active: false,
            message: Some("Test message".to_string()),
            version: Some("0.1.0".to_string()),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"loaded\":true"));
        assert!(json.contains("\"active\":false"));
        assert!(json.contains("Test message"));
        assert!(json.contains("0.1.0"));

        // Deserialize back
        let deserialized: BridgeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.loaded, status.loaded);
        assert_eq!(deserialized.active, status.active);
        assert_eq!(deserialized.message, status.message);
        assert_eq!(deserialized.version, status.version);
    }

    // ------------------------------------------------------------------------
    // MonitorBridge Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_monitor_bridge_new() {
        let bridge = MonitorBridge::new();
        assert_eq!(bridge.poll_interval_ms, 1000);
    }

    #[test]
    fn test_monitor_bridge_default() {
        let bridge = MonitorBridge::default();
        assert_eq!(bridge.poll_interval_ms, 1000);
    }

    #[test]
    fn test_monitor_bridge_domain() {
        let bridge = MonitorBridge::new();
        assert!(matches!(bridge.domain(), CdpDomain::ForgeMonitor));
    }

    #[test]
    fn test_monitor_bridge_name() {
        let bridge = MonitorBridge::new();
        assert_eq!(bridge.name(), "Monitor");
    }

    // ------------------------------------------------------------------------
    // TraceBridge Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_trace_bridge_new() {
        let _bridge = TraceBridge::new();
        // TraceBridge is a unit struct, just verify construction
    }

    #[test]
    fn test_trace_bridge_default() {
        let _bridge = TraceBridge::default();
    }

    #[test]
    fn test_trace_bridge_domain() {
        let bridge = TraceBridge::new();
        assert!(matches!(bridge.domain(), CdpDomain::ForgeTrace));
    }

    #[test]
    fn test_trace_bridge_name() {
        let bridge = TraceBridge::new();
        assert_eq!(bridge.name(), "Trace");
    }

    // ------------------------------------------------------------------------
    // SignalsBridge Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_signals_bridge_new() {
        let _bridge = SignalsBridge::new();
    }

    #[test]
    fn test_signals_bridge_default() {
        let _bridge = SignalsBridge::default();
    }

    #[test]
    fn test_signals_bridge_domain() {
        let bridge = SignalsBridge::new();
        assert!(matches!(bridge.domain(), CdpDomain::ForgeSignals));
    }

    #[test]
    fn test_signals_bridge_name() {
        let bridge = SignalsBridge::new();
        assert_eq!(bridge.name(), "Signals");
    }

    #[test]
    fn test_signals_bridge_supported() {
        let bridge = SignalsBridge::new();
        let signals = bridge.get_supported();
        #[cfg(unix)]
        {
            assert!(signals.len() >= 5, "Unix should have at least 5 signals");
            let signal_names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
            assert!(signal_names.contains(&"SIGINT"));
            assert!(signal_names.contains(&"SIGTERM"));
            assert!(signal_names.contains(&"SIGHUP"));
        }
        #[cfg(windows)]
        {
            assert!(signals.len() >= 2, "Windows should have at least 2 signals");
            let signal_names: Vec<&str> = signals.iter().map(|s| s.name.as_str()).collect();
            assert!(signal_names.contains(&"CTRL_C"));
        }
    }

    #[test]
    fn test_signals_bridge_signal_info_complete() {
        let bridge = SignalsBridge::new();
        let signals = bridge.get_supported();

        for signal in signals {
            assert!(!signal.name.is_empty(), "Signal name should not be empty");
            assert!(
                !signal.description.is_empty(),
                "Signal description should not be empty"
            );
        }
    }

    // ------------------------------------------------------------------------
    // DebuggerBridge Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_debugger_bridge_new() {
        let _bridge = DebuggerBridge::new();
    }

    #[test]
    fn test_debugger_bridge_default() {
        let _bridge = DebuggerBridge::default();
    }

    #[test]
    fn test_debugger_bridge_domain() {
        let bridge = DebuggerBridge::new();
        // Debugger reports through Runtime domain
        assert!(matches!(bridge.domain(), CdpDomain::ForgeRuntime));
    }

    #[test]
    fn test_debugger_bridge_name() {
        let bridge = DebuggerBridge::new();
        assert_eq!(bridge.name(), "Debugger");
    }

    // ------------------------------------------------------------------------
    // ExtensionBridges Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_extension_bridges_new() {
        let bridges = ExtensionBridges::new();
        assert_eq!(bridges.all().len(), 4);
    }

    #[test]
    fn test_extension_bridges_default() {
        let bridges = ExtensionBridges::default();
        assert_eq!(bridges.all().len(), 4);
    }

    #[test]
    fn test_extension_bridges_get_by_domain() {
        let bridges = ExtensionBridges::new();

        // ForgeMonitor should return the monitor bridge
        let monitor = bridges.get(&CdpDomain::ForgeMonitor);
        assert!(monitor.is_some());
        assert_eq!(monitor.unwrap().name(), "Monitor");

        // ForgeTrace should return the trace bridge
        let trace = bridges.get(&CdpDomain::ForgeTrace);
        assert!(trace.is_some());
        assert_eq!(trace.unwrap().name(), "Trace");

        // ForgeSignals should return the signals bridge
        let signals = bridges.get(&CdpDomain::ForgeSignals);
        assert!(signals.is_some());
        assert_eq!(signals.unwrap().name(), "Signals");

        // ForgeRuntime returns None (uses multiple bridges)
        let runtime = bridges.get(&CdpDomain::ForgeRuntime);
        assert!(runtime.is_none());
    }

    #[test]
    fn test_extension_bridges_all() {
        let bridges = ExtensionBridges::new();
        let all = bridges.all();

        let names: Vec<&str> = all.iter().map(|b| b.name()).collect();
        assert!(names.contains(&"Monitor"));
        assert!(names.contains(&"Trace"));
        assert!(names.contains(&"Signals"));
        assert!(names.contains(&"Debugger"));
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - CpuMetrics
    // ------------------------------------------------------------------------

    #[test]
    fn test_cpu_metrics_default() {
        let metrics = CpuMetrics::default();
        assert_eq!(metrics.total_percent, 0.0);
        assert!(metrics.per_core.is_empty());
        assert_eq!(metrics.core_count, 0);
    }

    #[test]
    fn test_cpu_metrics_serialization() {
        let metrics = CpuMetrics {
            total_percent: 45.5,
            per_core: vec![40.0, 50.0, 45.0, 47.0],
            core_count: 4,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("45.5"));
        assert!(json.contains("4"));

        let deserialized: CpuMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_percent, 45.5);
        assert_eq!(deserialized.core_count, 4);
        assert_eq!(deserialized.per_core.len(), 4);
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - MemoryMetrics
    // ------------------------------------------------------------------------

    #[test]
    fn test_memory_metrics_default() {
        let metrics = MemoryMetrics::default();
        assert_eq!(metrics.total_bytes, 0);
        assert_eq!(metrics.used_bytes, 0);
        assert_eq!(metrics.free_bytes, 0);
        assert_eq!(metrics.available_bytes, 0);
    }

    #[test]
    fn test_memory_metrics_serialization() {
        let metrics = MemoryMetrics {
            total_bytes: 16_000_000_000,
            used_bytes: 8_000_000_000,
            free_bytes: 4_000_000_000,
            available_bytes: 6_000_000_000,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: MemoryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_bytes, metrics.total_bytes);
        assert_eq!(deserialized.used_bytes, metrics.used_bytes);
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - RuntimeMetrics
    // ------------------------------------------------------------------------

    #[test]
    fn test_runtime_metrics_default() {
        let metrics = RuntimeMetrics::default();
        assert_eq!(metrics.pending_ops_count, 0);
        assert_eq!(metrics.event_loop_latency_us, 0);
        assert_eq!(metrics.uptime_secs, 0);
    }

    #[test]
    fn test_runtime_metrics_serialization() {
        let metrics = RuntimeMetrics {
            pending_ops_count: 5,
            event_loop_latency_us: 1500,
            uptime_secs: 3600,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("1500"));
        assert!(json.contains("3600"));

        let deserialized: RuntimeMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pending_ops_count, 5);
        assert_eq!(deserialized.event_loop_latency_us, 1500);
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - DiskInfo
    // ------------------------------------------------------------------------

    #[test]
    fn test_disk_info_serialization() {
        let disk = DiskInfo {
            mount_point: "/".to_string(),
            device: "/dev/sda1".to_string(),
            filesystem: "ext4".to_string(),
            total_bytes: 500_000_000_000,
            used_bytes: 250_000_000_000,
            free_bytes: 250_000_000_000,
        };

        let json = serde_json::to_string(&disk).unwrap();
        assert!(json.contains("/dev/sda1"));
        assert!(json.contains("ext4"));

        let deserialized: DiskInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mount_point, "/");
        assert_eq!(deserialized.filesystem, "ext4");
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - NetworkInfo
    // ------------------------------------------------------------------------

    #[test]
    fn test_network_info_serialization() {
        let network = NetworkInfo {
            interface: "eth0".to_string(),
            bytes_sent: 1_000_000,
            bytes_recv: 2_000_000,
            packets_sent: 1000,
            packets_recv: 2000,
        };

        let json = serde_json::to_string(&network).unwrap();
        assert!(json.contains("eth0"));
        assert!(json.contains("1000000"));

        let deserialized: NetworkInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.interface, "eth0");
        assert_eq!(deserialized.bytes_sent, 1_000_000);
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - SignalInfo
    // ------------------------------------------------------------------------

    #[test]
    fn test_signal_info_serialization() {
        let signal = SignalInfo {
            name: "SIGINT".to_string(),
            number: 2,
            description: "Interrupt".to_string(),
        };

        let json = serde_json::to_string(&signal).unwrap();
        assert!(json.contains("SIGINT"));
        assert!(json.contains("2"));
        assert!(json.contains("Interrupt"));

        let deserialized: SignalInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "SIGINT");
        assert_eq!(deserialized.number, 2);
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - SpanInfo
    // ------------------------------------------------------------------------

    #[test]
    fn test_span_info_serialization() {
        let span = SpanInfo {
            id: "span-123".to_string(),
            name: "fetch_data".to_string(),
            target: "app::http".to_string(),
            level: "INFO".to_string(),
            start_time_us: 1000000,
            end_time_us: Some(1500000),
            parent_id: Some("span-100".to_string()),
        };

        let json = serde_json::to_string(&span).unwrap();
        assert!(json.contains("span-123"));
        assert!(json.contains("fetch_data"));
        assert!(json.contains("span-100"));

        let deserialized: SpanInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "span-123");
        assert_eq!(deserialized.name, "fetch_data");
        assert_eq!(deserialized.end_time_us, Some(1500000));
    }

    #[test]
    fn test_span_info_no_parent_no_end() {
        let span = SpanInfo {
            id: "span-456".to_string(),
            name: "active_span".to_string(),
            target: "app".to_string(),
            level: "DEBUG".to_string(),
            start_time_us: 2000000,
            end_time_us: None,
            parent_id: None,
        };

        let json = serde_json::to_string(&span).unwrap();
        let deserialized: SpanInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "span-456");
        assert!(deserialized.end_time_us.is_none());
        assert!(deserialized.parent_id.is_none());
    }

    // ------------------------------------------------------------------------
    // Data Type Tests - BreakpointInfo
    // ------------------------------------------------------------------------

    #[test]
    fn test_breakpoint_info_serialization() {
        let bp = BreakpointInfo {
            id: "bp-1".to_string(),
            url: "file:///src/main.ts".to_string(),
            line: 42,
            column: Some(10),
            condition: Some("x > 5".to_string()),
            enabled: true,
        };

        let json = serde_json::to_string(&bp).unwrap();
        assert!(json.contains("bp-1"));
        assert!(json.contains("main.ts"));
        assert!(json.contains("42"));
        assert!(json.contains("x > 5"));

        let deserialized: BreakpointInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "bp-1");
        assert_eq!(deserialized.line, 42);
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_breakpoint_info_minimal() {
        let bp = BreakpointInfo {
            id: "bp-2".to_string(),
            url: "file:///app.js".to_string(),
            line: 100,
            column: None,
            condition: None,
            enabled: false,
        };

        let json = serde_json::to_string(&bp).unwrap();
        let deserialized: BreakpointInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "bp-2");
        assert!(deserialized.column.is_none());
        assert!(deserialized.condition.is_none());
        assert!(!deserialized.enabled);
    }

    // ------------------------------------------------------------------------
    // Clone Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bridge_status_clone() {
        let status = BridgeStatus {
            loaded: true,
            active: true,
            message: Some("Test".to_string()),
            version: Some("1.0".to_string()),
        };
        let cloned = status.clone();
        assert_eq!(cloned.loaded, status.loaded);
        assert_eq!(cloned.message, status.message);
    }

    #[test]
    fn test_cpu_metrics_clone() {
        let metrics = CpuMetrics {
            total_percent: 50.0,
            per_core: vec![45.0, 55.0],
            core_count: 2,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.total_percent, metrics.total_percent);
        assert_eq!(cloned.per_core, metrics.per_core);
    }

    #[test]
    fn test_memory_metrics_clone() {
        let metrics = MemoryMetrics {
            total_bytes: 1000,
            used_bytes: 500,
            free_bytes: 300,
            available_bytes: 400,
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.total_bytes, metrics.total_bytes);
    }

    // ------------------------------------------------------------------------
    // Debug Trait Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bridge_status_debug() {
        let status = BridgeStatus::default();
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("BridgeStatus"));
        assert!(debug_str.contains("loaded"));
    }

    #[test]
    fn test_cpu_metrics_debug() {
        let metrics = CpuMetrics::default();
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("CpuMetrics"));
    }

    #[test]
    fn test_signal_info_debug() {
        let signal = SignalInfo {
            name: "SIGTERM".to_string(),
            number: 15,
            description: "Terminate".to_string(),
        };
        let debug_str = format!("{:?}", signal);
        assert!(debug_str.contains("SignalInfo"));
        assert!(debug_str.contains("SIGTERM"));
    }
}
