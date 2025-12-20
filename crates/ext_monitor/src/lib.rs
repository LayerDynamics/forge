//! runtime:monitor extension - System and runtime monitoring for Forge
//!
//! Provides real-time system metrics (CPU, memory, disk, network), Deno runtime
//! metrics (event loop latency, uptime), process information, and subscription-based
//! continuous monitoring. Built on the [`sysinfo`](https://docs.rs/sysinfo) crate
//! for cross-platform system information access.
//!
//! **Runtime Module:** `runtime:monitor`
//!
//! ## Overview
//!
//! `ext_monitor` is a comprehensive monitoring extension that provides both one-time
//! metric snapshots and continuous monitoring via subscriptions. It uses the `sysinfo`
//! crate to access platform-specific system information APIs (proc fs on Linux,
//! sysctl on macOS, WMI on Windows).
//!
//! Key design features:
//! - **Cached State**: `MonitorState` maintains `System`, `Disks`, and `Networks`
//!   instances for efficient repeated queries
//! - **Async CPU Measurement**: CPU usage requires ~200ms measurement window for accuracy
//! - **Subscription Isolation**: Each subscription gets a dedicated `System` instance
//!   to avoid `Rc<RefCell<>>` borrow conflicts in tokio tasks
//! - **Resource Limits**: Maximum 10 concurrent subscriptions, processes limited to top 50
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ TypeScript Application (runtime:monitor)                     │
//! │  - getCpu(), getMemory(), getDisks()                         │
//! │  - subscribe(), nextSnapshot()                               │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ Deno Ops (op_monitor_*)
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ ext_monitor Operations                                       │
//! │  - MonitorState: cached System, Disks, Networks              │
//! │  - EventLoopLatencyMeasurer: background latency tracking     │
//! │  - Subscriptions: HashMap<id, Subscription>                  │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ sysinfo API calls
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ sysinfo crate                                                │
//! │  - System::new_with_specifics()                              │
//! │  - refresh_cpu_usage(), refresh_memory(), etc.               │
//! └────────────────┬─────────────────────────────────────────────┘
//!                  │ Platform-specific system APIs
//!                  ↓
//! ┌──────────────────────────────────────────────────────────────┐
//! │ OS System Information APIs                                   │
//! │  - Linux: /proc filesystem, sysfs                            │
//! │  - macOS: sysctl, host_statistics, vm_stat                   │
//! │  - Windows: WMI, Performance Counters, PSAPI                 │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Operations
//!
//! The extension provides 17 operations across 5 categories:
//!
//! ### System Metrics (6 operations)
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_monitor_cpu` | `CpuUsage` | CPU usage statistics (async, 200ms) |
//! | `op_monitor_memory` | `MemoryUsage` | RAM and swap statistics |
//! | `op_monitor_disk` | `Vec<DiskUsage>` | All mounted filesystem usage |
//! | `op_monitor_network` | `Vec<NetworkStats>` | Network interface traffic |
//! | `op_monitor_process_self` | `ProcessInfo` | Current process information |
//! | `op_monitor_processes` | `Vec<ProcessInfo>` | Top 50 processes by CPU |
//!
//! ### Runtime Metrics (2 operations)
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_monitor_runtime` | `RuntimeMetrics` | Event loop latency, uptime |
//! | `op_monitor_heap` | `HeapStats` | V8 heap stats (placeholder) |
//!
//! ### WebView Metrics (1 operation)
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_monitor_webview` | `WebViewStats` | Window metrics (placeholder) |
//!
//! ### Subscription API (4 operations)
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_monitor_subscribe` | `String` | Create metric subscription (returns ID) |
//! | `op_monitor_next` | `Option<MetricSnapshot>` | Get next snapshot (async) |
//! | `op_monitor_unsubscribe` | `()` | Cancel subscription |
//! | `op_monitor_subscriptions` | `Vec<SubscriptionInfo>` | List active subscriptions |
//!
//! ### Legacy Operations (2 operations, backward compatibility)
//!
//! | Operation | Return Type | Purpose |
//! |-----------|-------------|---------|
//! | `op_monitor_info` | `ExtensionInfo` | Extension metadata |
//! | `op_monitor_echo` | `String` | Echo test operation |
//!
//! ## Error Handling
//!
//! All operations use [`MonitorError`] with structured error codes:
//!
//! | Code | Error | Description |
//! |------|-------|-------------|
//! | 9800 | Generic | General monitoring operation error |
//! | 9801 | QueryFailed | Failed to query system metrics |
//! | 9802 | ProcessNotFound | Process ID not found |
//! | 9803 | PermissionDenied | Insufficient permissions for operation |
//! | 9804 | InvalidSubscription | Subscription ID invalid or expired |
//! | 9805 | SubscriptionLimitExceeded | Maximum 10 subscriptions exceeded |
//! | 9806 | WebViewMetricsUnavailable | WebView metrics not yet implemented |
//! | 9807 | PlatformNotSupported | Operation not supported on this platform |
//! | 9808 | InvalidInterval | Subscription interval < 100ms minimum |
//!
//! Errors are automatically converted to JavaScript exceptions via `#[derive(JsError)]`.
//!
//! ## TypeScript Usage
//!
//! ```typescript
//! import { getCpu, getMemory, subscribe, nextSnapshot } from "runtime:monitor";
//!
//! // One-time CPU snapshot (takes ~200ms)
//! const cpu = await getCpu();
//! console.log(`CPU: ${cpu.total_percent.toFixed(1)}%`);
//! cpu.per_core.forEach((usage, i) => {
//!   console.log(`  Core ${i}: ${usage.toFixed(1)}%`);
//! });
//!
//! // Memory snapshot (synchronous)
//! const mem = getMemory();
//! const usedGB = mem.used_bytes / (1024 ** 3);
//! const totalGB = mem.total_bytes / (1024 ** 3);
//! console.log(`Memory: ${usedGB.toFixed(1)} / ${totalGB.toFixed(1)} GB`);
//!
//! // Continuous monitoring via subscription
//! const subId = await subscribe({
//!   intervalMs: 1000,
//!   includeCpu: true,
//!   includeMemory: true,
//!   includeRuntime: true,
//! });
//!
//! // Receive 10 snapshots
//! for (let i = 0; i < 10; i++) {
//!   const snapshot = await nextSnapshot(subId);
//!   if (snapshot) {
//!     console.log(`CPU: ${snapshot.cpu?.total_percent.toFixed(1)}%`);
//!     console.log(`Memory: ${(snapshot.memory?.used_bytes / 1024**3).toFixed(1)} GB`);
//!     console.log(`Latency: ${snapshot.runtime?.event_loop_latency_us}μs`);
//!   }
//! }
//!
//! // Clean up
//! unsubscribe(subId);
//! ```
//!
//! ## Implementation Details
//!
//! ### CPU Measurement
//!
//! CPU usage calculation requires two measurements with a time interval because
//! it's calculated as `(cpu_time_delta / wall_time_delta)`. The `op_monitor_cpu`
//! operation:
//!
//! 1. Calls `system.refresh_cpu_usage()` to establish baseline
//! 2. Sleeps for 200ms to allow CPU time to accumulate
//! 3. Calls `system.refresh_cpu_usage()` again to measure delta
//! 4. Returns per-core and averaged total CPU percentages
//!
//! ### State Management
//!
//! `MonitorState` is stored in Deno's `OpState` and contains:
//! - `System`: Main system information cache (CPU, memory, processes)
//! - `Disks`: Cached disk/filesystem information
//! - `Networks`: Cached network interface information
//! - `EventLoopLatencyMeasurer`: Background latency tracking (Arc<AtomicU64>)
//! - `subscriptions`: HashMap of active metric subscriptions
//!
//! The cache is refreshed on each operation call (`refresh_cpu_usage()`,
//! `refresh_memory()`, etc.) to get current values.
//!
//! ### Subscription Architecture
//!
//! Each subscription spawns a tokio background task with:
//! - Dedicated `System` instance (to avoid `Rc<RefCell<>>` borrow conflicts)
//! - `tokio::time::interval` ticker for periodic collection
//! - `mpsc::channel` for sending snapshots to subscriber
//! - `CancellationToken` for graceful shutdown
//! - `Arc<AtomicU64>` snapshot counter
//!
//! The background task collects metrics at the specified interval and sends
//! `MetricSnapshot` messages through the channel. The subscriber receives them
//! via `op_monitor_next`, which temporarily borrows the receiver from the
//! subscription HashMap.
//!
//! ### Event Loop Latency Measurement
//!
//! The `EventLoopLatencyMeasurer` spawns a background task that:
//! 1. Schedules a 10ms sleep via `tokio::time::sleep`
//! 2. Measures actual wake-up time deviation from expected 10ms
//! 3. Stores latency in `Arc<AtomicU64>` for lock-free access
//! 4. Repeats every 500ms
//!
//! High latency indicates the event loop is blocked by long-running operations.
//!
//! ### Process Limits
//!
//! `op_monitor_processes` returns only the top 50 processes sorted by CPU usage
//! to prevent overwhelming the runtime. Full process list access would require
//! querying thousands of processes on typical systems, which is expensive.
//!
//! ## Platform Support
//!
//! | Platform | System Info Source | Status |
//! |----------|-------------------|--------|
//! | macOS (x64) | sysctl, host_statistics | ✅ Full support |
//! | macOS (ARM) | sysctl, host_statistics | ✅ Full support |
//! | Windows (x64) | WMI, Performance Counters | ✅ Full support |
//! | Windows (ARM) | WMI, Performance Counters | ✅ Full support |
//! | Linux (x64) | /proc, sysfs | ✅ Full support |
//! | Linux (ARM) | /proc, sysfs | ✅ Full support |
//!
//! Platform-specific behavior is handled by the `sysinfo` crate, which provides
//! a unified API across all platforms.
//!
//! ## Dependencies
//!
//! | Dependency | Version | Purpose |
//! |-----------|---------|---------  |
//! | `deno_core` | 0.373 | Op definitions and runtime integration |
//! | `sysinfo` | Latest | Cross-platform system information |
//! | `tokio` | 1.x | Async runtime for subscriptions |
//! | `tokio_util` | 0.7 | CancellationToken for subscription cleanup |
//! | `thiserror` | 2.x | Error type definitions |
//! | `deno_error` | 0.x | JavaScript error conversion |
//! | `serde` | 1.x | Serialization for metrics |
//! | `tracing` | 0.1 | Logging and diagnostics |
//! | `forge-weld-macro` | 0.1 | TypeScript binding generation |
//!
//! ## Testing
//!
//! ```bash
//! # Run all tests
//! cargo test -p ext_monitor
//!
//! # Run with output
//! cargo test -p ext_monitor -- --nocapture
//!
//! # With debug logging
//! RUST_LOG=ext_monitor=debug cargo test -p ext_monitor -- --nocapture
//! ```
//!
//! ## See Also
//!
//! - [`sysinfo`](https://docs.rs/sysinfo) - System information library
//! - [`ext_sys`] - System information extension (app info, clipboard, notifications)
//! - [`ext_process`] - Process spawning and management extension

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use deno_core::{op2, Extension, OpState};
use deno_error::JsError;
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate,
    RefreshKind, System,
};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace};

// ============================================================================
// Error Types (Error codes 9800-9899)
// ============================================================================

/// Error codes for monitor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MonitorErrorCode {
    /// Generic monitoring error
    Generic = 9800,
    /// Failed to query metrics
    QueryFailed = 9801,
    /// Process not found
    ProcessNotFound = 9802,
    /// Permission denied
    PermissionDenied = 9803,
    /// Invalid subscription ID
    InvalidSubscription = 9804,
    /// Subscription limit exceeded
    SubscriptionLimitExceeded = 9805,
    /// WebView metrics unavailable
    WebViewMetricsUnavailable = 9806,
    /// Platform not supported
    PlatformNotSupported = 9807,
    /// Invalid interval
    InvalidInterval = 9808,
}

/// Monitor extension errors
#[derive(Debug, Error, JsError)]
pub enum MonitorError {
    #[error("[{code}] Monitoring error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Query failed: {message}")]
    #[class(generic)]
    QueryFailed { code: u32, message: String },

    #[error("[{code}] Process not found: {message}")]
    #[class(generic)]
    ProcessNotFound { code: u32, message: String },

    #[error("[{code}] Invalid subscription: {message}")]
    #[class(generic)]
    InvalidSubscription { code: u32, message: String },

    #[error("[{code}] Subscription limit exceeded: {message}")]
    #[class(generic)]
    SubscriptionLimitExceeded { code: u32, message: String },

    #[error("[{code}] Invalid interval: {message}")]
    #[class(generic)]
    InvalidInterval { code: u32, message: String },
}

impl MonitorError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: MonitorErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn query_failed(message: impl Into<String>) -> Self {
        Self::QueryFailed {
            code: MonitorErrorCode::QueryFailed as u32,
            message: message.into(),
        }
    }

    pub fn process_not_found(message: impl Into<String>) -> Self {
        Self::ProcessNotFound {
            code: MonitorErrorCode::ProcessNotFound as u32,
            message: message.into(),
        }
    }

    pub fn invalid_subscription(message: impl Into<String>) -> Self {
        Self::InvalidSubscription {
            code: MonitorErrorCode::InvalidSubscription as u32,
            message: message.into(),
        }
    }

    pub fn subscription_limit_exceeded(message: impl Into<String>) -> Self {
        Self::SubscriptionLimitExceeded {
            code: MonitorErrorCode::SubscriptionLimitExceeded as u32,
            message: message.into(),
        }
    }

    pub fn invalid_interval(message: impl Into<String>) -> Self {
        Self::InvalidInterval {
            code: MonitorErrorCode::InvalidInterval as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Metric Types - System
// ============================================================================

/// CPU usage metrics
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct CpuUsage {
    /// Total CPU usage percentage (0-100)
    pub total_percent: f64,
    /// Per-core CPU usage percentages
    pub per_core: Vec<f64>,
    /// Number of CPU cores
    pub core_count: u32,
    /// CPU frequency in MHz (if available)
    pub frequency_mhz: Option<u64>,
}

/// Memory usage metrics
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct MemoryUsage {
    /// Total physical memory in bytes
    pub total_bytes: u64,
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Free memory in bytes
    pub free_bytes: u64,
    /// Available memory in bytes (free + reclaimable)
    pub available_bytes: u64,
    /// Total swap in bytes
    pub swap_total_bytes: u64,
    /// Used swap in bytes
    pub swap_used_bytes: u64,
}

/// Disk usage for a mount point
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct DiskUsage {
    /// Mount point path
    pub mount_point: String,
    /// Device name
    pub device: String,
    /// Filesystem type
    pub filesystem: String,
    /// Total capacity in bytes
    pub total_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Free space in bytes
    pub free_bytes: u64,
}

/// Network interface statistics
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct NetworkStats {
    /// Interface name
    pub interface: String,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_recv: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_recv: u64,
}

/// Process information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Resident memory in bytes
    pub memory_rss_bytes: u64,
    /// Virtual memory in bytes
    pub memory_virtual_bytes: u64,
    /// Process status
    pub status: String,
    /// Process start time (Unix timestamp)
    pub start_time_secs: u64,
    /// Parent process ID
    pub parent_pid: Option<u32>,
}

// ============================================================================
// Metric Types - Runtime
// ============================================================================

/// Deno runtime metrics
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct RuntimeMetrics {
    /// Number of pending async operations (placeholder)
    pub pending_ops_count: u32,
    /// Number of loaded modules (placeholder)
    pub module_count: u32,
    /// Event loop latency in microseconds
    pub event_loop_latency_us: u64,
    /// Process uptime in seconds
    pub uptime_secs: u64,
}

/// Heap statistics (placeholder for V8 heap stats)
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct HeapStats {
    /// Total heap size in bytes
    pub total_heap_size: u64,
    /// Used heap size in bytes
    pub used_heap_size: u64,
    /// Heap size limit in bytes
    pub heap_size_limit: u64,
    /// External memory in bytes
    pub external_memory: u64,
    /// Number of native contexts
    pub number_of_native_contexts: u32,
}

// ============================================================================
// Metric Types - WebView
// ============================================================================

/// WebView metrics for a single window
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct WebViewMetrics {
    /// Window ID
    pub window_id: String,
    /// Whether window is visible
    pub is_visible: bool,
    /// DOM node count (if available)
    pub dom_node_count: Option<u32>,
    /// JavaScript heap size in bytes
    pub js_heap_size_bytes: Option<u64>,
    /// JavaScript heap limit in bytes
    pub js_heap_size_limit: Option<u64>,
}

/// Aggregated WebView statistics
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct WebViewStats {
    /// Total window count
    pub window_count: u32,
    /// Number of visible windows
    pub visible_count: u32,
    /// Per-window metrics
    pub windows: Vec<WebViewMetrics>,
}

// ============================================================================
// Subscription Types
// ============================================================================

/// Subscription options
#[weld_struct]
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeOptions {
    /// Interval between metric updates in milliseconds
    pub interval_ms: u64,
    /// Whether to include CPU metrics
    #[serde(default = "default_true")]
    pub include_cpu: bool,
    /// Whether to include memory metrics
    #[serde(default = "default_true")]
    pub include_memory: bool,
    /// Whether to include runtime metrics
    #[serde(default)]
    pub include_runtime: bool,
    /// Whether to include process info for current process
    #[serde(default)]
    pub include_process: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            include_cpu: true,
            include_memory: true,
            include_runtime: false,
            include_process: false,
        }
    }
}

/// Complete metric snapshot
#[weld_struct]
#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricSnapshot {
    /// Timestamp when metrics were collected (Unix millis)
    pub timestamp_ms: u64,
    /// CPU metrics (if requested)
    pub cpu: Option<CpuUsage>,
    /// Memory metrics (if requested)
    pub memory: Option<MemoryUsage>,
    /// Runtime metrics (if requested)
    pub runtime: Option<RuntimeMetrics>,
    /// Current process info (if requested)
    pub process: Option<ProcessInfo>,
}

/// Subscription information
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionInfo {
    /// Unique subscription ID
    pub id: String,
    /// Interval in milliseconds
    pub interval_ms: u64,
    /// Whether subscription is active
    pub is_active: bool,
    /// Number of snapshots delivered
    pub snapshot_count: u64,
}

// ============================================================================
// Legacy Types (backward compatibility)
// ============================================================================

#[weld_struct]
#[derive(Serialize)]
pub struct ExtensionInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

// ============================================================================
// State Management
// ============================================================================

/// Internal subscription state
struct Subscription {
    id: String,
    options: SubscribeOptions,
    sender: mpsc::Sender<MetricSnapshot>,
    receiver: Option<mpsc::Receiver<MetricSnapshot>>,
    snapshot_count: Arc<AtomicU64>,
    cancel_token: CancellationToken,
}

/// Event loop latency measurer
pub struct EventLoopLatencyMeasurer {
    last_latency_us: Arc<AtomicU64>,
    start_time: Instant,
}

impl EventLoopLatencyMeasurer {
    pub fn new() -> Self {
        Self {
            last_latency_us: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    pub fn get_latency_us(&self) -> u64 {
        self.last_latency_us.load(Ordering::Relaxed)
    }

    pub fn get_uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Start measuring event loop latency (background task)
    pub fn start_measurement(&self) {
        let latency = self.last_latency_us.clone();
        tokio::spawn(async move {
            loop {
                let start = Instant::now();
                tokio::time::sleep(Duration::from_millis(10)).await;
                let elapsed = start.elapsed();
                // Expected ~10ms, measure deviation
                let latency_us = elapsed.as_micros().saturating_sub(10000) as u64;
                latency.store(latency_us, Ordering::Relaxed);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }
}

impl Default for EventLoopLatencyMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitor state stored in OpState
pub struct MonitorState {
    /// Active subscriptions (private - use subscription_count() for access)
    subscriptions: HashMap<String, Subscription>,
    /// Next subscription ID
    next_subscription_id: u64,
    /// Maximum allowed subscriptions
    max_subscriptions: usize,
    /// System info (for caching CPU measurements) (pub for bridge access)
    pub system: System,
    /// Disks info (pub for bridge access)
    pub disks: Disks,
    /// Networks info (pub for bridge access)
    pub networks: Networks,
    /// Event loop latency measurer (pub for bridge access)
    pub latency_measurer: EventLoopLatencyMeasurer,
    /// Whether latency measurement has started
    latency_started: bool,
}

impl MonitorState {
    /// Get the number of active metric subscriptions
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            subscriptions: HashMap::new(),
            next_subscription_id: 1,
            max_subscriptions: 10,
            system: System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            latency_measurer: EventLoopLatencyMeasurer::new(),
            latency_started: false,
        }
    }
}

/// Initialize monitor state in OpState
pub fn init_monitor_state(op_state: &mut OpState) {
    debug!("Initializing monitor state");
    op_state.put(MonitorState::default());
}

// ============================================================================
// Legacy Operations (backward compatibility)
// ============================================================================

#[weld_op]
#[op2]
#[serde]
fn op_monitor_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_monitor",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
    }
}

#[weld_op]
#[op2]
#[string]
fn op_monitor_echo(#[string] message: String) -> String {
    message
}

// ============================================================================
// System Metric Operations
// ============================================================================

/// Get CPU usage statistics
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_monitor_cpu(state: Rc<RefCell<OpState>>) -> Result<CpuUsage, MonitorError> {
    // First refresh to establish baseline
    {
        let mut s = state.borrow_mut();
        let monitor_state = s.borrow_mut::<MonitorState>();
        monitor_state.system.refresh_cpu_usage();
    }

    // Wait for CPU usage calculation (sysinfo needs time between measurements)
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second refresh to get actual usage
    let mut s = state.borrow_mut();
    let monitor_state = s.borrow_mut::<MonitorState>();
    monitor_state.system.refresh_cpu_usage();

    let cpus = monitor_state.system.cpus();
    let per_core: Vec<f64> = cpus.iter().map(|cpu| cpu.cpu_usage() as f64).collect();
    let total_percent = if per_core.is_empty() {
        0.0
    } else {
        per_core.iter().sum::<f64>() / per_core.len() as f64
    };

    let frequency_mhz = cpus.first().map(|cpu| cpu.frequency());

    Ok(CpuUsage {
        total_percent,
        per_core,
        core_count: cpus.len() as u32,
        frequency_mhz,
    })
}

/// Get memory usage statistics
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_memory(state: &mut OpState) -> Result<MemoryUsage, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();
    monitor_state.system.refresh_memory();

    Ok(MemoryUsage {
        total_bytes: monitor_state.system.total_memory(),
        used_bytes: monitor_state.system.used_memory(),
        free_bytes: monitor_state.system.free_memory(),
        available_bytes: monitor_state.system.available_memory(),
        swap_total_bytes: monitor_state.system.total_swap(),
        swap_used_bytes: monitor_state.system.used_swap(),
    })
}

/// Get disk usage for all mounts
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_disk(state: &mut OpState) -> Result<Vec<DiskUsage>, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();
    monitor_state.disks.refresh();

    let disks: Vec<DiskUsage> = monitor_state
        .disks
        .iter()
        .map(|disk| DiskUsage {
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            device: disk.name().to_string_lossy().to_string(),
            filesystem: disk.file_system().to_string_lossy().to_string(),
            total_bytes: disk.total_space(),
            used_bytes: disk.total_space().saturating_sub(disk.available_space()),
            free_bytes: disk.available_space(),
        })
        .collect();

    Ok(disks)
}

/// Get network interface statistics
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_network(state: &mut OpState) -> Result<Vec<NetworkStats>, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();
    monitor_state.networks.refresh();

    let networks: Vec<NetworkStats> = monitor_state
        .networks
        .iter()
        .map(|(name, network)| NetworkStats {
            interface: name.clone(),
            bytes_sent: network.total_transmitted(),
            bytes_recv: network.total_received(),
            packets_sent: network.total_packets_transmitted(),
            packets_recv: network.total_packets_received(),
        })
        .collect();

    Ok(networks)
}

/// Get current process information
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_process_self(state: &mut OpState) -> Result<ProcessInfo, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();

    let pid = Pid::from_u32(std::process::id());
    monitor_state.system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::everything(),
    );

    let process = monitor_state
        .system
        .process(pid)
        .ok_or_else(|| MonitorError::process_not_found("current process"))?;

    Ok(ProcessInfo {
        pid: pid.as_u32(),
        name: process.name().to_string_lossy().to_string(),
        cpu_percent: process.cpu_usage() as f64,
        memory_rss_bytes: process.memory(),
        memory_virtual_bytes: process.virtual_memory(),
        status: format!("{:?}", process.status()),
        start_time_secs: process.start_time(),
        parent_pid: process.parent().map(|p| p.as_u32()),
    })
}

/// Get list of all processes (top 50 by CPU usage)
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_processes(state: &mut OpState) -> Result<Vec<ProcessInfo>, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();
    monitor_state.system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        false,
        ProcessRefreshKind::new().with_cpu().with_memory(),
    );

    let mut processes: Vec<ProcessInfo> = monitor_state
        .system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            cpu_percent: process.cpu_usage() as f64,
            memory_rss_bytes: process.memory(),
            memory_virtual_bytes: process.virtual_memory(),
            status: format!("{:?}", process.status()),
            start_time_secs: process.start_time(),
            parent_pid: process.parent().map(|p| p.as_u32()),
        })
        .collect();

    // Sort by CPU usage descending and limit to 50
    processes.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(50);

    Ok(processes)
}

// ============================================================================
// Runtime Metric Operations
// ============================================================================

/// Get Deno runtime metrics
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_runtime(state: &mut OpState) -> Result<RuntimeMetrics, MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();

    // Start latency measurement if not already started
    if !monitor_state.latency_started {
        monitor_state.latency_measurer.start_measurement();
        monitor_state.latency_started = true;
    }

    Ok(RuntimeMetrics {
        pending_ops_count: 0, // Would need JsRuntime access
        module_count: 0,      // Would need JsRuntime access
        event_loop_latency_us: monitor_state.latency_measurer.get_latency_us(),
        uptime_secs: monitor_state.latency_measurer.get_uptime_secs(),
    })
}

/// Get V8 heap statistics (placeholder)
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_heap() -> Result<HeapStats, MonitorError> {
    // Note: Would need access to V8 isolate for real heap stats
    // This is a placeholder that returns default values
    Ok(HeapStats::default())
}

// ============================================================================
// WebView Metric Operations
// ============================================================================

/// Get WebView statistics (requires window manager coordination)
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_webview() -> Result<WebViewStats, MonitorError> {
    // Note: Would need coordination with ext_window/WindowManager
    // This is a placeholder that returns empty stats
    Ok(WebViewStats::default())
}

// ============================================================================
// Subscription Operations
// ============================================================================

/// Subscribe to continuous metric updates
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_monitor_subscribe(
    state: Rc<RefCell<OpState>>,
    #[serde] options: SubscribeOptions,
) -> Result<String, MonitorError> {
    // Validate interval (minimum 100ms to prevent excessive load)
    if options.interval_ms < 100 {
        return Err(MonitorError::invalid_interval(
            "Minimum interval is 100ms".to_string(),
        ));
    }

    let (subscription_id, latency_measurer) = {
        let mut s = state.borrow_mut();
        let monitor_state = s.borrow_mut::<MonitorState>();

        // Check subscription limit
        if monitor_state.subscriptions.len() >= monitor_state.max_subscriptions {
            return Err(MonitorError::subscription_limit_exceeded(format!(
                "Maximum {} subscriptions allowed",
                monitor_state.max_subscriptions
            )));
        }

        // Start latency measurement if including runtime and not started
        if options.include_runtime && !monitor_state.latency_started {
            monitor_state.latency_measurer.start_measurement();
            monitor_state.latency_started = true;
        }

        let id = format!("sub-{}", monitor_state.next_subscription_id);
        monitor_state.next_subscription_id += 1;

        let (tx, rx) = mpsc::channel(32);
        let cancel_token = CancellationToken::new();
        let snapshot_count = Arc::new(AtomicU64::new(0));

        let subscription = Subscription {
            id: id.clone(),
            options: options.clone(),
            sender: tx.clone(),
            receiver: Some(rx),
            snapshot_count: snapshot_count.clone(),
            cancel_token: cancel_token.clone(),
        };

        monitor_state.subscriptions.insert(id.clone(), subscription);

        // Spawn background collection task with its own System instance
        let interval = options.interval_ms;
        let id_clone = id.clone();
        // Clone the latency measurer's atomic for runtime metrics
        let latency_measurer_clone = Arc::clone(&monitor_state.latency_measurer.last_latency_us);
        let start_time = monitor_state.latency_measurer.start_time;

        tokio::spawn(async move {
            // Create dedicated System for this subscription task
            let mut system = System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            );

            let mut ticker = tokio::time::interval(Duration::from_millis(interval));

            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        trace!("Subscription {} cancelled", id_clone);
                        break;
                    }
                    _ = ticker.tick() => {
                        // Collect metrics snapshot using dedicated System
                        let snapshot = collect_snapshot_send_safe(
                            &mut system,
                            &options,
                            &latency_measurer_clone,
                            start_time,
                        );

                        // Send to subscriber
                        if tx.send(snapshot).await.is_err() {
                            debug!("Subscription {} receiver dropped", id_clone);
                            break;
                        }

                        snapshot_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        (id, monitor_state.latency_measurer.last_latency_us.clone())
    };

    // Suppress unused variable warning
    let _ = latency_measurer;

    Ok(subscription_id)
}

/// Get next metric snapshot from subscription
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_monitor_next(
    state: Rc<RefCell<OpState>>,
    #[string] subscription_id: String,
) -> Result<Option<MetricSnapshot>, MonitorError> {
    // Take receiver temporarily
    let maybe_receiver = {
        let mut s = state.borrow_mut();
        let monitor_state = s.borrow_mut::<MonitorState>();
        monitor_state
            .subscriptions
            .get_mut(&subscription_id)
            .and_then(|sub| sub.receiver.take())
    };

    let mut receiver = maybe_receiver
        .ok_or_else(|| MonitorError::invalid_subscription(subscription_id.clone()))?;

    let result = receiver.recv().await;

    // Put receiver back
    {
        let mut s = state.borrow_mut();
        let monitor_state = s.borrow_mut::<MonitorState>();
        if let Some(sub) = monitor_state.subscriptions.get_mut(&subscription_id) {
            sub.receiver = Some(receiver);
        }
    }

    Ok(result)
}

/// Unsubscribe from metric updates
#[weld_op]
#[op2(fast)]
pub fn op_monitor_unsubscribe(
    state: &mut OpState,
    #[string] subscription_id: String,
) -> Result<(), MonitorError> {
    let monitor_state = state.borrow_mut::<MonitorState>();

    if let Some(subscription) = monitor_state.subscriptions.remove(&subscription_id) {
        subscription.cancel_token.cancel();
        debug!("Unsubscribed from {}", subscription_id);
        Ok(())
    } else {
        Err(MonitorError::invalid_subscription(subscription_id))
    }
}

/// List active subscriptions
#[weld_op]
#[op2]
#[serde]
pub fn op_monitor_subscriptions(state: &OpState) -> Vec<SubscriptionInfo> {
    let monitor_state = state.borrow::<MonitorState>();

    monitor_state
        .subscriptions
        .values()
        .map(|sub| {
            // Use sender to check if the channel is still healthy (not closed)
            let channel_healthy = !sub.sender.is_closed();
            trace!(
                subscription_id = %sub.id,
                channel_healthy = channel_healthy,
                "Collecting subscription info"
            );
            SubscriptionInfo {
                id: sub.id.clone(),
                interval_ms: sub.options.interval_ms,
                is_active: !sub.cancel_token.is_cancelled() && channel_healthy,
                snapshot_count: sub.snapshot_count.load(Ordering::Relaxed),
            }
        })
        .collect()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Send-safe version of snapshot collection for use in spawned tasks.
/// Uses a dedicated System instance instead of borrowing from OpState.
fn collect_snapshot_send_safe(
    system: &mut System,
    options: &SubscribeOptions,
    latency_us: &Arc<AtomicU64>,
    start_time: Instant,
) -> MetricSnapshot {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let cpu = if options.include_cpu {
        system.refresh_cpu_usage();

        let cpus = system.cpus();
        let per_core: Vec<f64> = cpus.iter().map(|cpu| cpu.cpu_usage() as f64).collect();
        let total_percent = if per_core.is_empty() {
            0.0
        } else {
            per_core.iter().sum::<f64>() / per_core.len() as f64
        };

        Some(CpuUsage {
            total_percent,
            per_core,
            core_count: cpus.len() as u32,
            frequency_mhz: cpus.first().map(|cpu| cpu.frequency()),
        })
    } else {
        None
    };

    let memory = if options.include_memory {
        system.refresh_memory();

        Some(MemoryUsage {
            total_bytes: system.total_memory(),
            used_bytes: system.used_memory(),
            free_bytes: system.free_memory(),
            available_bytes: system.available_memory(),
            swap_total_bytes: system.total_swap(),
            swap_used_bytes: system.used_swap(),
        })
    } else {
        None
    };

    let runtime = if options.include_runtime {
        Some(RuntimeMetrics {
            pending_ops_count: 0,
            module_count: 0,
            event_loop_latency_us: latency_us.load(Ordering::Relaxed),
            uptime_secs: start_time.elapsed().as_secs(),
        })
    } else {
        None
    };

    let process = if options.include_process {
        let pid = Pid::from_u32(std::process::id());
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            false,
            ProcessRefreshKind::new().with_cpu().with_memory(),
        );

        system.process(pid).map(|p| ProcessInfo {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_percent: p.cpu_usage() as f64,
            memory_rss_bytes: p.memory(),
            memory_virtual_bytes: p.virtual_memory(),
            status: format!("{:?}", p.status()),
            start_time_secs: p.start_time(),
            parent_pid: p.parent().map(|pp| pp.as_u32()),
        })
    } else {
        None
    };

    MetricSnapshot {
        timestamp_ms,
        cpu,
        memory,
        runtime,
        process,
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn monitor_extension() -> Extension {
    runtime_monitor::ext()
}
