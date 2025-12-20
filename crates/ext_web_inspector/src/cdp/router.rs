//! CDP message routing for Forge custom domains.
//!
//! Routes CDP commands to the appropriate domain handler based on
//! the method name (e.g., "Forge.Monitor.getMetrics" -> ForgeMonitor domain).
//! Uses extension bridges to access real data from ext_monitor, ext_trace,
//! ext_signals, and ext_debugger.

use std::cell::RefCell;
use std::rc::Rc;

use deno_core::OpState;
use serde_json::{json, Value};
use tracing::{debug, trace};

use crate::bridge::{DebuggerBridge, ExtensionBridge, MonitorBridge, SignalsBridge, TraceBridge};
use crate::{CdpDomain, WebInspectorError};

// ============================================================================
// CDP Router
// ============================================================================

/// Route a CDP command to the appropriate Forge domain handler.
///
/// # Arguments
/// * `state` - The operation state (provides access to extension states)
/// * `domain` - The Forge CDP domain
/// * `method` - The method name (without domain prefix)
/// * `params` - Optional parameters for the method
///
/// # Returns
/// The JSON result from the domain handler
pub async fn route_cdp_command(
    state: &Rc<RefCell<OpState>>,
    domain: &CdpDomain,
    method: &str,
    params: Option<Value>,
) -> Result<Value, WebInspectorError> {
    debug!("Routing CDP command: {}.{}", domain.as_str(), method);
    trace!("CDP params: {:?}", params);

    match domain {
        CdpDomain::ForgeMonitor => handle_monitor_command(state, method, params).await,
        CdpDomain::ForgeTrace => handle_trace_command(state, method, params).await,
        CdpDomain::ForgeSignals => handle_signals_command(state, method, params).await,
        CdpDomain::ForgeRuntime => handle_runtime_command(state, method, params).await,
    }
}

// ============================================================================
// Forge.Monitor Domain Handler
// ============================================================================

/// Handle commands for the Forge.Monitor domain
async fn handle_monitor_command(
    state: &Rc<RefCell<OpState>>,
    method: &str,
    params: Option<Value>,
) -> Result<Value, WebInspectorError> {
    match method {
        "enable" => {
            debug!("Forge.Monitor.enable called");
            Ok(json!({}))
        }
        "disable" => {
            debug!("Forge.Monitor.disable called");
            Ok(json!({}))
        }
        "getMetrics" => {
            // Aggregate metrics from ext_monitor if available
            let metrics = get_monitor_metrics(state)?;
            Ok(metrics)
        }
        "getCpuUsage" => {
            let cpu = get_cpu_usage(state)?;
            Ok(cpu)
        }
        "getMemoryUsage" => {
            let memory = get_memory_usage(state)?;
            Ok(memory)
        }
        "getRuntimeMetrics" => {
            let runtime = get_runtime_metrics(state)?;
            Ok(runtime)
        }
        "startProfiling" => {
            let profile_id = start_profiling(state, params)?;
            Ok(json!({ "profileId": profile_id }))
        }
        "stopProfiling" => {
            let profile_id = params
                .as_ref()
                .and_then(|p| p.get("profileId"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| WebInspectorError::cdp_error("Missing profileId parameter"))?;
            let profile = stop_profiling(state, profile_id)?;
            Ok(profile)
        }
        _ => Err(WebInspectorError::cdp_error(format!(
            "Unknown Forge.Monitor method: {}",
            method
        ))),
    }
}

// ============================================================================
// Forge.Trace Domain Handler
// ============================================================================

/// Handle commands for the Forge.Trace domain
async fn handle_trace_command(
    state: &Rc<RefCell<OpState>>,
    method: &str,
    params: Option<Value>,
) -> Result<Value, WebInspectorError> {
    match method {
        "enable" => {
            debug!("Forge.Trace.enable called");
            Ok(json!({}))
        }
        "disable" => {
            debug!("Forge.Trace.disable called");
            Ok(json!({}))
        }
        "getSpans" => {
            let options = params.unwrap_or(json!({}));
            let spans = get_trace_spans(state, &options)?;
            Ok(json!({ "spans": spans }))
        }
        "getActiveSpans" => {
            let spans = get_active_spans(state)?;
            Ok(json!({ "spans": spans }))
        }
        "clearSpans" => {
            let count = clear_spans(state)?;
            Ok(json!({ "clearedCount": count }))
        }
        _ => Err(WebInspectorError::cdp_error(format!(
            "Unknown Forge.Trace method: {}",
            method
        ))),
    }
}

// ============================================================================
// Forge.Signals Domain Handler
// ============================================================================

/// Handle commands for the Forge.Signals domain
async fn handle_signals_command(
    state: &Rc<RefCell<OpState>>,
    method: &str,
    _params: Option<Value>,
) -> Result<Value, WebInspectorError> {
    match method {
        "enable" => {
            debug!("Forge.Signals.enable called");
            Ok(json!({}))
        }
        "disable" => {
            debug!("Forge.Signals.disable called");
            Ok(json!({}))
        }
        "getSupportedSignals" => {
            let signals = get_supported_signals();
            Ok(json!({ "signals": signals }))
        }
        "getActiveSubscriptions" => {
            let subscriptions = get_signal_subscriptions(state)?;
            Ok(json!({ "subscriptions": subscriptions }))
        }
        "getStatus" => {
            let bridge = SignalsBridge::new();
            let op_state = state.borrow();
            let summary = bridge.summary(&op_state);
            Ok(summary)
        }
        _ => Err(WebInspectorError::cdp_error(format!(
            "Unknown Forge.Signals method: {}",
            method
        ))),
    }
}

// ============================================================================
// Forge.Runtime Domain Handler
// ============================================================================

/// Handle commands for the Forge.Runtime domain
async fn handle_runtime_command(
    state: &Rc<RefCell<OpState>>,
    method: &str,
    _params: Option<Value>,
) -> Result<Value, WebInspectorError> {
    match method {
        "enable" => {
            debug!("Forge.Runtime.enable called");
            Ok(json!({}))
        }
        "disable" => {
            debug!("Forge.Runtime.disable called");
            Ok(json!({}))
        }
        "getAppInfo" => {
            let app_info = get_app_info(state)?;
            Ok(json!({ "app": app_info }))
        }
        "getWindows" => {
            let windows = get_windows(state)?;
            Ok(json!({ "windows": windows }))
        }
        "getExtensions" => {
            let extensions = get_extensions(state)?;
            Ok(json!({ "extensions": extensions }))
        }
        "getIpcChannels" => {
            let channels = get_ipc_channels(state)?;
            Ok(json!({ "channels": channels }))
        }
        _ => Err(WebInspectorError::cdp_error(format!(
            "Unknown Forge.Runtime method: {}",
            method
        ))),
    }
}

// ============================================================================
// Monitor Domain Helpers
// ============================================================================

fn get_monitor_metrics(state: &Rc<RefCell<OpState>>) -> Result<Value, WebInspectorError> {
    let bridge = MonitorBridge::new();
    let op_state = state.borrow();

    let cpu = bridge.get_cpu(&op_state).unwrap_or_default();
    let memory = bridge.get_memory(&op_state).unwrap_or_default();
    let runtime = bridge.get_runtime(&op_state).unwrap_or_default();

    Ok(json!({
        "cpu": {
            "totalPercent": cpu.total_percent,
            "perCore": cpu.per_core,
            "coreCount": cpu.core_count
        },
        "memory": {
            "totalBytes": memory.total_bytes,
            "usedBytes": memory.used_bytes,
            "freeBytes": memory.free_bytes,
            "availableBytes": memory.available_bytes
        },
        "eventLoop": {
            "latencyUs": runtime.event_loop_latency_us
        },
        "runtime": {
            "uptimeSecs": runtime.uptime_secs
        },
        "subscriptions": bridge.get_subscription_count(&op_state),
        "timestamp": current_timestamp_ms()
    }))
}

fn get_cpu_usage(state: &Rc<RefCell<OpState>>) -> Result<Value, WebInspectorError> {
    let bridge = MonitorBridge::new();
    let op_state = state.borrow();
    let cpu = bridge.get_cpu(&op_state).unwrap_or_default();

    Ok(json!({
        "totalPercent": cpu.total_percent,
        "perCore": cpu.per_core,
        "coreCount": cpu.core_count
    }))
}

fn get_memory_usage(state: &Rc<RefCell<OpState>>) -> Result<Value, WebInspectorError> {
    let bridge = MonitorBridge::new();
    let op_state = state.borrow();
    let memory = bridge.get_memory(&op_state).unwrap_or_default();

    Ok(json!({
        "totalBytes": memory.total_bytes,
        "usedBytes": memory.used_bytes,
        "freeBytes": memory.free_bytes,
        "availableBytes": memory.available_bytes
    }))
}

fn get_runtime_metrics(state: &Rc<RefCell<OpState>>) -> Result<Value, WebInspectorError> {
    let bridge = MonitorBridge::new();
    let op_state = state.borrow();
    let runtime = bridge.get_runtime(&op_state).unwrap_or_default();

    Ok(json!({
        "eventLoopLatencyUs": runtime.event_loop_latency_us,
        "uptimeSecs": runtime.uptime_secs
    }))
}

fn start_profiling(
    _state: &Rc<RefCell<OpState>>,
    _params: Option<Value>,
) -> Result<String, WebInspectorError> {
    // Generate a profile ID
    let profile_id = format!("profile-{}", current_timestamp_ms());
    debug!("Started profiling: {}", profile_id);
    Ok(profile_id)
}

fn stop_profiling(
    _state: &Rc<RefCell<OpState>>,
    profile_id: &str,
) -> Result<Value, WebInspectorError> {
    debug!("Stopped profiling: {}", profile_id);
    Ok(json!({
        "profileId": profile_id,
        "samples": [],
        "durationMs": 0
    }))
}

// ============================================================================
// Trace Domain Helpers
// ============================================================================

fn get_trace_spans(
    state: &Rc<RefCell<OpState>>,
    options: &Value,
) -> Result<Vec<Value>, WebInspectorError> {
    let limit = options.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let bridge = TraceBridge::new();
    let op_state = state.borrow();

    let finished_spans = bridge.get_finished_spans(&op_state);
    let spans: Vec<Value> = finished_spans
        .iter()
        .take(limit)
        .map(|span| {
            json!({
                "id": span.id,
                "name": span.name,
                "startedAt": span.started_at,
                "durationMs": span.duration_ms,
                "attributes": span.attributes,
                "result": span.result
            })
        })
        .collect();

    Ok(spans)
}

fn get_active_spans(state: &Rc<RefCell<OpState>>) -> Result<Vec<Value>, WebInspectorError> {
    let bridge = TraceBridge::new();
    let op_state = state.borrow();

    let active = bridge.get_active_spans(&op_state);
    let spans: Vec<Value> = active
        .iter()
        .map(|(id, name)| {
            json!({
                "id": id,
                "name": name
            })
        })
        .collect();

    Ok(spans)
}

fn clear_spans(_state: &Rc<RefCell<OpState>>) -> Result<u32, WebInspectorError> {
    // Note: Clearing spans requires mutable access to TraceState
    // This would need an op_trace_clear or similar in ext_trace
    // For now, return 0 as a placeholder
    debug!("clear_spans called - requires mutable access to TraceState");
    Ok(0)
}

// ============================================================================
// Signals Domain Helpers
// ============================================================================

fn get_supported_signals() -> Vec<Value> {
    let bridge = SignalsBridge::new();
    bridge
        .get_supported()
        .iter()
        .map(|sig| {
            json!({
                "name": sig.name,
                "number": sig.number,
                "description": sig.description
            })
        })
        .collect()
}

fn get_signal_subscriptions(state: &Rc<RefCell<OpState>>) -> Result<Vec<Value>, WebInspectorError> {
    let bridge = SignalsBridge::new();
    let op_state = state.borrow();

    let ids = bridge.get_subscription_ids(&op_state);
    let subscriptions: Vec<Value> = ids
        .iter()
        .map(|id| {
            json!({
                "id": id,
                "active": true
            })
        })
        .collect();

    Ok(subscriptions)
}

// ============================================================================
// Runtime Domain Helpers
// ============================================================================

fn get_app_info(_state: &Rc<RefCell<OpState>>) -> Result<Value, WebInspectorError> {
    // Would query ext_app state
    Ok(json!({
        "name": "Forge App",
        "version": "0.1.0",
        "denoVersion": env!("CARGO_PKG_VERSION"),
        "platform": current_platform(),
        "arch": current_arch(),
        "cpuCount": num_cpus()
    }))
}

fn get_windows(state: &Rc<RefCell<OpState>>) -> Result<Vec<Value>, WebInspectorError> {
    // Get windows from inspector state
    let s = state.borrow();
    let inspector_state = s.borrow::<crate::WebInspectorState>();

    let windows: Vec<Value> = inspector_state
        .sessions
        .keys()
        .map(|window_id| {
            json!({
                "id": window_id,
                "title": "Window",
                "visible": true
            })
        })
        .collect();

    Ok(windows)
}

fn get_extensions(state: &Rc<RefCell<OpState>>) -> Result<Vec<Value>, WebInspectorError> {
    let op_state = state.borrow();

    // Create bridges and check their status
    let monitor_bridge = MonitorBridge::new();
    let trace_bridge = TraceBridge::new();
    let signals_bridge = SignalsBridge::new();
    let debugger_bridge = DebuggerBridge::new();

    let monitor_status = monitor_bridge.status(&op_state);
    let trace_status = trace_bridge.status(&op_state);
    let signals_status = signals_bridge.status(&op_state);
    let debugger_status = debugger_bridge.status(&op_state);

    Ok(vec![
        json!({ "name": "ext_fs", "status": "loaded" }),
        json!({ "name": "ext_window", "status": "loaded" }),
        json!({ "name": "ext_ipc", "status": "loaded" }),
        json!({
            "name": "ext_monitor",
            "status": if monitor_status.loaded { "loaded" } else { "not_loaded" },
            "active": monitor_status.active,
            "message": monitor_status.message
        }),
        json!({
            "name": "ext_trace",
            "status": if trace_status.loaded { "loaded" } else { "not_loaded" },
            "active": trace_status.active,
            "message": trace_status.message
        }),
        json!({
            "name": "ext_signals",
            "status": if signals_status.loaded { "loaded" } else { "not_loaded" },
            "active": signals_status.active,
            "message": signals_status.message
        }),
        json!({
            "name": "ext_debugger",
            "status": if debugger_status.loaded { "loaded" } else { "not_loaded" },
            "active": debugger_status.active,
            "message": debugger_status.message
        }),
        json!({ "name": "ext_web_inspector", "status": "loaded" }),
    ])
}

fn get_ipc_channels(_state: &Rc<RefCell<OpState>>) -> Result<Vec<Value>, WebInspectorError> {
    // Would query ext_ipc state
    Ok(vec![])
}

// ============================================================================
// Utility Functions
// ============================================================================

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(1)
}

fn current_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "darwin"
    }
    #[cfg(target_os = "windows")]
    {
        "win32"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "unknown"
    }
}

fn current_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "x64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "arm64"
    }
    #[cfg(target_arch = "x86")]
    {
        "x86"
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
    {
        "unknown"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_supported_signals() {
        let signals = get_supported_signals();
        assert!(!signals.is_empty());
    }

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        assert!(["darwin", "win32", "linux", "unknown"].contains(&platform));
    }

    #[test]
    fn test_current_arch() {
        let arch = current_arch();
        assert!(["x64", "arm64", "x86", "unknown"].contains(&arch));
    }
}
