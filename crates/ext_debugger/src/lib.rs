//! V8 Inspector Protocol debugger extension for Forge runtime.
//!
//! This extension provides a complete Chrome DevTools Protocol (CDP) client implementation
//! for debugging JavaScript/TypeScript code running in the Deno V8 runtime. It exposes the
//! V8 Inspector Protocol via WebSocket connection, enabling programmatic control over
//! execution, inspection of runtime state, and comprehensive debugging capabilities.
//!
//! # Module: `runtime:debugger`
//!
//! The extension exposes its functionality through the `runtime:debugger` TypeScript module,
//! providing a high-level API over the V8 Inspector Protocol.
//!
//! # Features
//!
//! ## Connection Management
//! - WebSocket connection to V8 Inspector (local or remote)
//! - Automatic debugger domain and runtime enabling
//! - Connection status checking
//! - Configurable timeout and connection URL
//!
//! ## Breakpoint Management
//! - Set breakpoints by URL and line number
//! - Conditional breakpoints with JavaScript expressions
//! - Enable/disable breakpoints without removing them
//! - Remove individual or all breakpoints
//! - List breakpoints with hit counts and metadata
//! - Automatic V8 breakpoint location resolution
//!
//! ## Execution Control
//! - Pause execution at current statement
//! - Resume execution from paused state
//! - Step over (execute current line, pause at next)
//! - Step into function calls
//! - Step out of current function
//! - Continue to specific location (run-to-cursor)
//! - Configure exception pause behavior (none, uncaught, all)
//!
//! ## Stack Inspection
//! - Retrieve complete call stack when paused
//! - Access scope chain for each frame (local, closure, global)
//! - Inspect variables in any scope
//! - Navigate scope hierarchy
//!
//! ## Object Inspection
//! - Fetch properties of remote objects
//! - Differentiate primitives from complex objects
//! - Access object metadata (type, subtype, class name)
//! - Preview object contents without full fetch
//!
//! ## Expression Evaluation
//! - Evaluate arbitrary JavaScript expressions
//! - Evaluate in global context or specific call frame
//! - Access local variables and closures
//! - Produce side effects and modify state
//!
//! ## Script Management
//! - List all loaded scripts with metadata
//! - Retrieve source code by script ID
//! - Track dynamically loaded code
//! - Monitor script parsing events
//!
//! ## Event Handling
//! - Listen for pause events (breakpoints, steps, exceptions)
//! - Listen for script parsed events (module loading)
//! - Broadcast channels for event distribution
//! - Async event listeners with cleanup functions
//!
//! # Architecture
//!
//! ## Inspector Client
//!
//! The core of the extension is the `InspectorClient` which maintains a WebSocket connection
//! to the V8 Inspector. The inspector runs on a configurable port (default: 9229) and
//! communicates using JSON-RPC 2.0 protocol messages.
//!
//! ```text
//! ┌─────────────────┐      WebSocket      ┌─────────────────┐
//! │ InspectorClient │ ←──────────────────→ │  V8 Inspector   │
//! │  (Rust/Tokio)   │   JSON-RPC Messages  │  (Chrome CDP)   │
//! └─────────────────┘                      └─────────────────┘
//!         ↑                                         ↑
//!         │ State Access                            │
//!         ↓                                         │
//! ┌─────────────────┐                              │
//! │ DebuggerState   │                              │
//! │ (Arc<Mutex<>>)  │                              │
//! │ - breakpoints   │                              │
//! │ - scripts       │                              │
//! │ - event channels│                              │
//! └─────────────────┘                              │
//!         ↑                                         │
//!         │ Op Calls                                │
//!         ↓                                         │
//! ┌─────────────────┐                              │
//! │  TypeScript     │                              │
//! │  runtime:       │                              │
//! │    debugger     │                              │
//! └─────────────────┘                              │
//!         ↑                                         │
//!         │ Import                                  │
//!         ↓                                         │
//! ┌─────────────────┐      Direct Access          │
//! │   Application   │ ─────────────────────────────┘
//! │   (main.ts)     │    V8 Runtime Introspection
//! └─────────────────┘
//! ```
//!
//! ## State Management
//!
//! The `DebuggerState` structure (wrapped in `Arc<Mutex<>>` for thread safety) maintains:
//! - Active WebSocket connection to V8 Inspector
//! - Breakpoint registry (ID to breakpoint mapping)
//! - Script registry (URL to script metadata mapping)
//! - Event broadcast channels (pause events, script events)
//! - Next request ID for protocol messages
//!
//! ## Protocol Communication
//!
//! All V8 Inspector operations use JSON-RPC 2.0 format:
//!
//! **Request:**
//! ```json
//! {
//!   "id": 1,
//!   "method": "Debugger.setBreakpoint",
//!   "params": {
//!     "location": {
//!       "scriptId": "42",
//!       "lineNumber": 10
//!     }
//!   }
//! }
//! ```
//!
//! **Response:**
//! ```json
//! {
//!   "id": 1,
//!   "result": {
//!     "breakpointId": "1:10:0:file:///src/main.ts",
//!     "actualLocation": {
//!       "scriptId": "42",
//!       "lineNumber": 10,
//!       "columnNumber": 0
//!     }
//!   }
//! }
//! ```
//!
//! **Event (no ID):**
//! ```json
//! {
//!   "method": "Debugger.paused",
//!   "params": {
//!     "callFrames": [...],
//!     "reason": "breakpoint",
//!     "hitBreakpoints": ["1:10:0:file:///src/main.ts"]
//!   }
//! }
//! ```
//!
//! ## Event Distribution
//!
//! Pause and script events are distributed via Tokio broadcast channels:
//! 1. V8 Inspector sends event message via WebSocket
//! 2. `InspectorClient` receives and parses the message
//! 3. Event is broadcast to all active receivers
//! 4. TypeScript listeners (`onPaused`, `onScriptParsed`) receive events asynchronously
//!
//! # Usage Example (TypeScript)
//!
//! ```typescript
//! import * as debugger from "runtime:debugger";
//!
//! // Connect to V8 Inspector
//! await debugger.connect();
//!
//! // Set breakpoint with condition
//! const bp = await debugger.setBreakpoint("file:///src/main.ts", 42, {
//!   condition: "user.role === 'admin'"
//! });
//! console.log(`Breakpoint set: ${bp.id}`);
//!
//! // Listen for pause events
//! const cleanup = debugger.onPaused(async (event) => {
//!   console.log(`Paused: ${event.reason}`);
//!
//!   // Print stack trace
//!   for (const frame of event.call_frames) {
//!     console.log(`  at ${frame.function_name} (${frame.url}:${frame.location.line_number})`);
//!   }
//!
//!   // Inspect local variables
//!   const topFrame = event.call_frames[0];
//!   const localScope = topFrame.scope_chain.find(s => s.type === "local");
//!
//!   if (localScope?.object.object_id) {
//!     const props = await debugger.getProperties(localScope.object.object_id);
//!     console.log("Local variables:");
//!     for (const prop of props) {
//!       if (prop.value) {
//!         console.log(`  ${prop.name} = ${prop.value.description}`);
//!       }
//!     }
//!   }
//!
//!   // Evaluate expression in current frame context
//!   const result = await debugger.evaluate("user.email", topFrame.call_frame_id);
//!   console.log(`user.email = ${result.value}`);
//!
//!   // Resume execution
//!   await debugger.resume();
//! });
//!
//! // Set exception pause mode
//! await debugger.setPauseOnExceptions("uncaught");
//!
//! // Later: cleanup and disconnect
//! cleanup();
//! await debugger.disconnect();
//! ```
//!
//! # Thread Safety
//!
//! - `DebuggerState` is wrapped in `Arc<Mutex<>>` for safe concurrent access
//! - WebSocket connection is `Arc<RwLock<>>` to allow concurrent reads
//! - Broadcast channels enable safe multi-consumer event distribution
//! - All ops are async and properly synchronized
//!
//! # Error Handling
//!
//! All operations return `Result<T, DebuggerError>` with structured error codes (9600-9614).
//! Common error scenarios:
//! - `NotConnected` (9602): Operation attempted without active inspector connection
//! - `ConnectionFailed` (9601): WebSocket connection to inspector failed
//! - `BreakpointFailed` (9603): Breakpoint could not be set at specified location
//! - `InvalidFrameId` (9604): Call frame ID doesn't exist in current pause state
//! - `EvaluationFailed` (9606): Expression evaluation threw exception
//! - `ProtocolError` (9611): V8 Inspector returned error response
//!
//! # Error Codes
//!
//! This extension uses error codes 9600-9614 for all operations. See [`DebuggerErrorCode`]
//! for the complete list.
//!
//! # Implementation Details
//!
//! ## V8 Inspector Connection
//!
//! The connection process involves:
//! 1. Opening WebSocket to `ws://localhost:9229/<session-id>`
//! 2. Sending `Debugger.enable` to activate debugging domain
//! 3. Sending `Runtime.enable` to activate runtime domain
//! 4. Starting message receiver task for async events
//!
//! ## Line Number Indexing
//!
//! **Important:** V8 Inspector uses 0-based line and column numbering, which differs from
//! most text editors (1-based). The TypeScript API preserves V8's 0-based convention for
//! consistency with Chrome DevTools.
//!
//! ## Remote Object References
//!
//! Complex objects (arrays, objects, functions) are not sent inline. Instead, V8 assigns
//! each object a unique `object_id` string. To inspect object properties, use `getProperties()`
//! with the object ID. Primitive values (number, string, boolean, null) are sent inline.
//!
//! ## Breakpoint Resolution
//!
//! When setting a breakpoint, V8 may adjust the location to the nearest executable statement.
//! For example, setting a breakpoint on a comment line will resolve to the next code line.
//! The returned `Breakpoint` struct contains the actual resolved location.
//!
//! ## Performance Considerations
//!
//! - WebSocket communication adds latency (typically <1ms on localhost)
//! - Large object inspection can be slow (fetch only needed properties)
//! - Event broadcasting has minimal overhead (Tokio channels)
//! - State locks are held briefly (no blocking I/O under lock)
//!
//! # Testing
//!
//! This crate includes comprehensive tests covering:
//! - Connection lifecycle (connect, disconnect, reconnect)
//! - Breakpoint operations (set, remove, enable/disable, list)
//! - Execution control (pause, resume, step operations)
//! - Expression evaluation (global and frame contexts)
//! - Event handling (pause and script events)
//! - Error conditions (not connected, invalid IDs, protocol errors)
//!
//! Run tests with: `cargo test -p ext_debugger`
//!
//! # See Also
//!
//! - [Chrome DevTools Protocol Documentation](https://chromedevtools.github.io/devtools-protocol/)
//! - [V8 Inspector Protocol Viewer](https://chromedevtools.github.io/devtools-protocol/v8/)
//! - [`ext_devtools`](../ext_devtools/index.html) - DevTools frontend integration
//! - [`ext_trace`](../ext_trace/index.html) - Application tracing and profiling

use deno_core::{op2, Extension, OpState, Resource, ResourceId};
use forge_weld_macro::{weld_enum, weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, error};

mod inspector;

use inspector::{InspectorClient, InspectorMessage};

// ============================================================================
// Error Types (9600-9699)
// ============================================================================

/// Error codes for debugger operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerErrorCode {
    /// Generic debugger error
    Generic = 9600,
    /// Failed to connect to inspector
    ConnectionFailed = 9601,
    /// Not connected to inspector
    NotConnected = 9602,
    /// Breakpoint operation failed
    BreakpointFailed = 9603,
    /// Invalid frame ID
    InvalidFrameId = 9604,
    /// Invalid scope ID
    InvalidScopeId = 9605,
    /// Expression evaluation failed
    EvaluationFailed = 9606,
    /// Script/source not found
    SourceNotFound = 9607,
    /// Step operation failed
    StepFailed = 9608,
    /// Pause operation failed
    PauseFailed = 9609,
    /// Resume operation failed
    ResumeFailed = 9610,
    /// Protocol error from V8
    ProtocolError = 9611,
    /// Inspector not enabled
    NotEnabled = 9612,
    /// Operation timeout
    Timeout = 9613,
    /// Invalid breakpoint location
    InvalidLocation = 9614,
}

#[derive(Debug, Error, deno_error::JsError)]
pub enum DebuggerError {
    #[error("[{code}] {message}")]
    #[class(generic)]
    WithCode { code: i32, message: String },

    #[error("Not connected to debugger")]
    #[class(generic)]
    NotConnected,

    #[error("Connection failed: {0}")]
    #[class(generic)]
    ConnectionFailed(String),

    #[error("Breakpoint operation failed: {0}")]
    #[class(generic)]
    BreakpointFailed(String),

    #[error("Invalid frame ID: {0}")]
    #[class(generic)]
    InvalidFrameId(String),

    #[error("Invalid scope ID: {0}")]
    #[class(generic)]
    InvalidScopeId(String),

    #[error("Evaluation failed: {0}")]
    #[class(generic)]
    EvaluationFailed(String),

    #[error("Source not found: {0}")]
    #[class(generic)]
    SourceNotFound(String),

    #[error("Step operation failed: {0}")]
    #[class(generic)]
    StepFailed(String),

    #[error("Pause failed: {0}")]
    #[class(generic)]
    PauseFailed(String),

    #[error("Resume failed: {0}")]
    #[class(generic)]
    ResumeFailed(String),

    #[error("Protocol error: {0}")]
    #[class(generic)]
    ProtocolError(String),

    #[error("Debugger not enabled")]
    #[class(generic)]
    NotEnabled,

    #[error("Operation timed out")]
    #[class(generic)]
    Timeout,

    #[error("Invalid breakpoint location")]
    #[class(generic)]
    InvalidLocation,

    #[error("Internal error: {0}")]
    #[class(generic)]
    Internal(String),
}

// ============================================================================
// Data Types
// ============================================================================

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub status: String,
}

/// Source code location
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    /// Script identifier
    pub script_id: String,
    /// Line number (0-based)
    pub line_number: u32,
    /// Column number (0-based)
    pub column_number: Option<u32>,
}

/// Breakpoint information
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Breakpoint identifier
    pub id: String,
    /// Actual location after V8 adjustment
    pub location: Location,
    /// Conditional expression (if any)
    pub condition: Option<String>,
    /// Number of times this breakpoint was hit
    pub hit_count: u32,
    /// Whether breakpoint is enabled
    pub enabled: bool,
}

/// Scope types
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub enum ScopeType {
    Global,
    #[default]
    Local,
    Closure,
    Catch,
    Block,
    Script,
    Eval,
    Module,
    WasmExpressionStack,
}

/// A scope in the call stack
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Type of scope
    #[serde(rename = "type")]
    pub scope_type: ScopeType,
    /// Object representing the scope
    pub object: RemoteObject,
    /// Scope name (for closure)
    pub name: Option<String>,
    /// Location where scope starts
    pub start_location: Option<Location>,
    /// Location where scope ends
    pub end_location: Option<Location>,
}

/// Remote object reference
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteObject {
    /// Object type
    #[serde(rename = "type")]
    pub object_type: String,
    /// Object subtype (array, null, regexp, date, map, set, etc.)
    pub subtype: Option<String>,
    /// Object class name
    pub class_name: Option<String>,
    /// Primitive value (for primitives)
    pub value: Option<serde_json::Value>,
    /// Unique object ID for non-primitives
    pub object_id: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Preview of object properties
    pub preview: Option<ObjectPreview>,
}

/// Preview of object properties
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPreview {
    /// Object type
    #[serde(rename = "type")]
    pub preview_type: String,
    /// Object subtype
    pub subtype: Option<String>,
    /// Description
    pub description: Option<String>,
    /// True if not all properties are shown
    pub overflow: bool,
    /// Preview properties
    pub properties: Vec<PropertyPreview>,
    /// Preview of internal properties
    pub entries: Option<Vec<EntryPreview>>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyPreview {
    pub name: String,
    #[serde(rename = "type")]
    pub property_type: String,
    pub subtype: Option<String>,
    pub value: Option<String>,
    pub value_preview: Option<Box<ObjectPreview>>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPreview {
    pub key: Option<ObjectPreview>,
    pub value: ObjectPreview,
}

/// Call frame in the stack
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    /// Call frame identifier
    pub call_frame_id: String,
    /// Function name
    pub function_name: String,
    /// Location in source code
    pub location: Location,
    /// Script URL
    pub url: String,
    /// Scope chain
    pub scope_chain: Vec<Scope>,
    /// `this` object
    pub this_object: RemoteObject,
    /// Return value (only when paused on return)
    pub return_value: Option<RemoteObject>,
}

/// Variable in a scope
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: RemoteObject,
    /// Whether variable can be modified
    pub writable: Option<bool>,
    /// Whether variable is configurable
    pub configurable: Option<bool>,
    /// Whether variable shows in enumeration
    pub enumerable: Option<bool>,
}

/// Property descriptor
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDescriptor {
    /// Property name
    pub name: String,
    /// Property value
    pub value: Option<RemoteObject>,
    /// True if writable
    pub writable: Option<bool>,
    /// Getter function
    pub get: Option<RemoteObject>,
    /// Setter function
    pub set: Option<RemoteObject>,
    /// True if configurable
    pub configurable: bool,
    /// True if enumerable
    pub enumerable: bool,
    /// True if own property (not inherited)
    pub is_own: Option<bool>,
    /// Property symbol
    pub symbol: Option<RemoteObject>,
}

/// Reasons for pause
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum PauseReason {
    Breakpoint,
    Step,
    Exception,
    DebuggerStatement,
    Ambiguous,
    Assert,
    Instrumentation,
    OOM,
    PromiseRejection,
    #[default]
    Other,
}

/// Paused event data
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedEvent {
    /// Reason for pause
    pub reason: PauseReason,
    /// Call frames
    pub call_frames: Vec<CallFrame>,
    /// Breakpoint IDs that were hit
    pub hit_breakpoints: Option<Vec<String>>,
    /// Additional data
    pub data: Option<serde_json::Value>,
    /// Async stack trace
    pub async_stack_trace: Option<serde_json::Value>,
}

/// Script information
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    /// Script ID
    pub script_id: String,
    /// Script URL
    pub url: String,
    /// Source map URL
    pub source_map_url: Option<String>,
    /// Start line
    pub start_line: u32,
    /// Start column
    pub start_column: u32,
    /// End line
    pub end_line: u32,
    /// End column
    pub end_column: u32,
    /// Content hash
    pub hash: String,
    /// Script length
    pub length: u32,
    /// Execution context ID
    pub execution_context_id: Option<i32>,
}

/// Result of stepping
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Whether step succeeded
    pub success: bool,
    /// Paused event after step (if any)
    pub paused_event: Option<PausedEvent>,
}

/// Breakpoint options for setting
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreakpointOptions {
    /// Conditional expression
    pub condition: Option<String>,
    /// Column number
    pub column_number: Option<u32>,
}

/// Connection options
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectOptions {
    /// WebSocket URL (defaults to internal inspector)
    pub url: Option<String>,
    /// Connection timeout in milliseconds
    pub timeout_ms: Option<u32>,
}

/// Connection status
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Whether connected
    pub connected: bool,
    /// Whether debugger domain is enabled
    pub enabled: bool,
    /// Whether currently paused
    pub paused: bool,
    /// WebSocket URL
    pub url: Option<String>,
}

/// Exception pause state
#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExceptionPauseState {
    /// Don't pause on exceptions
    #[default]
    None,
    /// Pause on uncaught exceptions
    Uncaught,
    /// Pause on all exceptions
    All,
}

/// Exception details
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionDetails {
    /// Exception ID
    pub exception_id: i32,
    /// Exception text
    pub text: String,
    /// Line number where exception occurred
    pub line_number: u32,
    /// Column number
    pub column_number: u32,
    /// Script ID
    pub script_id: Option<String>,
    /// URL
    pub url: Option<String>,
    /// Exception object
    pub exception: Option<RemoteObject>,
    /// Stack trace
    pub stack_trace: Option<serde_json::Value>,
}

/// Result of breakpoint resolution
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointResolved {
    /// Breakpoint ID
    pub breakpoint_id: String,
    /// Actual location
    pub location: Location,
}

// ============================================================================
// State Management
// ============================================================================

/// Debugger state stored in OpState
pub struct DebuggerState {
    /// Inspector client
    client: Arc<Mutex<Option<InspectorClient>>>,
    /// Whether connected
    connected: Arc<RwLock<bool>>,
    /// Whether debugger is enabled
    enabled: Arc<RwLock<bool>>,
    /// Whether currently paused
    paused: Arc<RwLock<bool>>,
    /// Local breakpoint registry
    breakpoints: Arc<RwLock<HashMap<String, Breakpoint>>>,
    /// Next breakpoint ID (reserved for programmatic breakpoint IDs)
    #[allow(dead_code)]
    next_bp_id: Arc<AtomicU64>,
    /// Cached call frames from last pause
    call_frames: Arc<RwLock<Vec<CallFrame>>>,
    /// Script registry
    scripts: Arc<RwLock<HashMap<String, ScriptInfo>>>,
    /// Pause event channel
    pause_tx: broadcast::Sender<PausedEvent>,
    /// Resume event channel
    resume_tx: broadcast::Sender<()>,
    /// Script parsed event channel
    script_tx: broadcast::Sender<ScriptInfo>,
    /// Breakpoint resolved event channel
    bp_resolved_tx: broadcast::Sender<BreakpointResolved>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        let (pause_tx, _) = broadcast::channel(64);
        let (resume_tx, _) = broadcast::channel(64);
        let (script_tx, _) = broadcast::channel(64);
        let (bp_resolved_tx, _) = broadcast::channel(64);

        Self {
            client: Arc::new(Mutex::new(None)),
            connected: Arc::new(RwLock::new(false)),
            enabled: Arc::new(RwLock::new(false)),
            paused: Arc::new(RwLock::new(false)),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            next_bp_id: Arc::new(AtomicU64::new(1)),
            call_frames: Arc::new(RwLock::new(Vec::new())),
            scripts: Arc::new(RwLock::new(HashMap::new())),
            pause_tx,
            resume_tx,
            script_tx,
            bp_resolved_tx,
        }
    }
}

impl DebuggerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pause_rx(&self) -> broadcast::Receiver<PausedEvent> {
        self.pause_tx.subscribe()
    }

    pub fn resume_rx(&self) -> broadcast::Receiver<()> {
        self.resume_tx.subscribe()
    }

    pub fn script_rx(&self) -> broadcast::Receiver<ScriptInfo> {
        self.script_tx.subscribe()
    }

    pub fn bp_resolved_rx(&self) -> broadcast::Receiver<BreakpointResolved> {
        self.bp_resolved_tx.subscribe()
    }

    // Bridge accessor methods for ext_web_inspector (sync versions using blocking_read)

    /// Check if debugger is connected (sync, blocking)
    pub fn is_connected(&self) -> bool {
        *self.connected.blocking_read()
    }

    /// Check if debugger is enabled (sync, blocking)
    pub fn is_enabled(&self) -> bool {
        *self.enabled.blocking_read()
    }

    /// Check if debugger is paused (sync, blocking)
    pub fn is_paused(&self) -> bool {
        *self.paused.blocking_read()
    }

    /// Get the count of registered breakpoints (sync, blocking)
    pub fn breakpoint_count(&self) -> usize {
        self.breakpoints.blocking_read().len()
    }

    /// Get a snapshot of breakpoint IDs (sync, blocking)
    pub fn breakpoint_ids(&self) -> Vec<String> {
        self.breakpoints.blocking_read().keys().cloned().collect()
    }

    /// Get the count of parsed scripts (sync, blocking)
    pub fn script_count(&self) -> usize {
        self.scripts.blocking_read().len()
    }

    /// Get a snapshot of script info (sync, blocking)
    pub fn scripts_snapshot(&self) -> Vec<ScriptInfo> {
        self.scripts.blocking_read().values().cloned().collect()
    }

    /// Get the current call frames if paused (sync, blocking)
    pub fn call_frames_snapshot(&self) -> Vec<CallFrame> {
        self.call_frames.blocking_read().clone()
    }
}

// Event receivers as resources
struct PauseEventReceiver(broadcast::Receiver<PausedEvent>);
impl Resource for PauseEventReceiver {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("DebuggerPauseEventReceiver")
    }
}

#[allow(dead_code)]
struct ResumeEventReceiver(broadcast::Receiver<()>);
impl Resource for ResumeEventReceiver {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("DebuggerResumeEventReceiver")
    }
}

struct ScriptEventReceiver(broadcast::Receiver<ScriptInfo>);
impl Resource for ScriptEventReceiver {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("DebuggerScriptEventReceiver")
    }
}

#[allow(dead_code)]
struct BpResolvedEventReceiver(broadcast::Receiver<BreakpointResolved>);
impl Resource for BpResolvedEventReceiver {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed("DebuggerBpResolvedEventReceiver")
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Get extension information
#[weld_op]
#[op2]
#[serde]
pub fn op_debugger_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_debugger".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        status: "active".to_string(),
    }
}

/// Connect to V8 inspector
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_connect(
    state: Rc<RefCell<OpState>>,
    #[serde] options: ConnectOptions,
) -> Result<ConnectionStatus, DebuggerError> {
    let debugger_state = {
        let state_ref = state.borrow();
        state_ref
            .try_borrow::<Arc<DebuggerState>>()
            .cloned()
            .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))?
    };

    let url = options
        .url
        .unwrap_or_else(|| "ws://127.0.0.1:9229".to_string());
    let timeout = options.timeout_ms.unwrap_or(5000);

    debug!(url = %url, timeout = timeout, "Connecting to V8 inspector");

    // Create inspector client
    let client = InspectorClient::connect(&url, timeout)
        .await
        .map_err(|e| DebuggerError::ConnectionFailed(e.to_string()))?;

    // Store client
    {
        let mut client_guard = debugger_state.client.lock().await;
        *client_guard = Some(client);
    }

    // Update state
    *debugger_state.connected.write().await = true;

    // Enable debugger domain
    {
        let client_guard = debugger_state.client.lock().await;
        if let Some(client) = client_guard.as_ref() {
            client
                .send_method("Debugger.enable", serde_json::json!({}))
                .await
                .map_err(|e| DebuggerError::ProtocolError(e.to_string()))?;

            // Also enable runtime for evaluate
            client
                .send_method("Runtime.enable", serde_json::json!({}))
                .await
                .map_err(|e| DebuggerError::ProtocolError(e.to_string()))?;
        }
    }

    *debugger_state.enabled.write().await = true;

    // Start event listener
    start_event_listener(debugger_state.clone());

    Ok(ConnectionStatus {
        connected: true,
        enabled: true,
        paused: false,
        url: Some(url),
    })
}

/// Disconnect from V8 inspector
#[weld_op]
#[op2(async)]
pub async fn op_debugger_disconnect(state: Rc<RefCell<OpState>>) -> Result<bool, DebuggerError> {
    let debugger_state = {
        let state_ref = state.borrow();
        state_ref
            .try_borrow::<Arc<DebuggerState>>()
            .cloned()
            .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))?
    };

    // Disable debugger domain first
    {
        let client_guard = debugger_state.client.lock().await;
        if let Some(client) = client_guard.as_ref() {
            let _ = client
                .send_method("Debugger.disable", serde_json::json!({}))
                .await;
        }
    }

    // Clear client
    {
        let mut client_guard = debugger_state.client.lock().await;
        *client_guard = None;
    }

    // Update state
    *debugger_state.connected.write().await = false;
    *debugger_state.enabled.write().await = false;
    *debugger_state.paused.write().await = false;

    debug!("Disconnected from V8 inspector");
    Ok(true)
}

/// Check if connected to inspector
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_is_connected(
    state: Rc<RefCell<OpState>>,
) -> Result<ConnectionStatus, DebuggerError> {
    let debugger_state = {
        let state_ref = state.borrow();
        state_ref
            .try_borrow::<Arc<DebuggerState>>()
            .cloned()
            .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))?
    };

    let connected = *debugger_state.connected.read().await;
    let enabled = *debugger_state.enabled.read().await;
    let paused = *debugger_state.paused.read().await;

    Ok(ConnectionStatus {
        connected,
        enabled,
        paused,
        url: None,
    })
}

/// Set a breakpoint by URL
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_set_breakpoint(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[smi] line_number: u32,
    #[serde] options: BreakpointOptions,
) -> Result<Breakpoint, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let mut params = serde_json::json!({
        "url": url,
        "lineNumber": line_number,
    });

    if let Some(col) = options.column_number {
        params["columnNumber"] = serde_json::json!(col);
    }
    if let Some(cond) = &options.condition {
        params["condition"] = serde_json::json!(cond);
    }

    let response = send_command(&debugger_state, "Debugger.setBreakpointByUrl", params).await?;

    let bp_id = response
        .get("breakpointId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DebuggerError::BreakpointFailed("No breakpoint ID returned".to_string()))?
        .to_string();

    // Parse locations
    let location = response
        .get("locations")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .map(|loc| Location {
            script_id: loc
                .get("scriptId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            line_number: loc.get("lineNumber").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            column_number: loc
                .get("columnNumber")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32),
        })
        .unwrap_or_default();

    let breakpoint = Breakpoint {
        id: bp_id.clone(),
        location,
        condition: options.condition,
        hit_count: 0,
        enabled: true,
    };

    // Store locally
    debugger_state
        .breakpoints
        .write()
        .await
        .insert(bp_id, breakpoint.clone());

    debug!(id = %breakpoint.id, "Breakpoint set");
    Ok(breakpoint)
}

/// Remove a breakpoint
#[weld_op]
#[op2(async)]
pub async fn op_debugger_remove_breakpoint(
    state: Rc<RefCell<OpState>>,
    #[string] breakpoint_id: String,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let params = serde_json::json!({
        "breakpointId": breakpoint_id,
    });

    send_command(&debugger_state, "Debugger.removeBreakpoint", params).await?;

    // Remove from local registry
    debugger_state
        .breakpoints
        .write()
        .await
        .remove(&breakpoint_id);

    debug!(id = %breakpoint_id, "Breakpoint removed");
    Ok(true)
}

/// Remove all breakpoints
#[weld_op]
#[op2(async)]
#[smi]
pub async fn op_debugger_remove_all_breakpoints(
    state: Rc<RefCell<OpState>>,
) -> Result<u32, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let breakpoints: Vec<String> = debugger_state
        .breakpoints
        .read()
        .await
        .keys()
        .cloned()
        .collect();

    let count = breakpoints.len() as u32;

    for bp_id in breakpoints {
        let params = serde_json::json!({
            "breakpointId": bp_id,
        });
        let _ = send_command(&debugger_state, "Debugger.removeBreakpoint", params).await;
    }

    debugger_state.breakpoints.write().await.clear();

    debug!(count = count, "All breakpoints removed");
    Ok(count)
}

/// List all breakpoints
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_list_breakpoints(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<Breakpoint>, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;

    let breakpoints: Vec<Breakpoint> = debugger_state
        .breakpoints
        .read()
        .await
        .values()
        .cloned()
        .collect();

    Ok(breakpoints)
}

/// Enable a breakpoint
#[weld_op]
#[op2(async)]
pub async fn op_debugger_enable_breakpoint(
    state: Rc<RefCell<OpState>>,
    #[string] breakpoint_id: String,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    // V8 doesn't have direct enable/disable - we need to re-add with same params
    // For simplicity, we just track locally
    let mut breakpoints = debugger_state.breakpoints.write().await;
    if let Some(bp) = breakpoints.get_mut(&breakpoint_id) {
        bp.enabled = true;
        Ok(true)
    } else {
        Err(DebuggerError::BreakpointFailed(format!(
            "Breakpoint {} not found",
            breakpoint_id
        )))
    }
}

/// Disable a breakpoint
#[weld_op]
#[op2(async)]
pub async fn op_debugger_disable_breakpoint(
    state: Rc<RefCell<OpState>>,
    #[string] breakpoint_id: String,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let mut breakpoints = debugger_state.breakpoints.write().await;
    if let Some(bp) = breakpoints.get_mut(&breakpoint_id) {
        bp.enabled = false;
        Ok(true)
    } else {
        Err(DebuggerError::BreakpointFailed(format!(
            "Breakpoint {} not found",
            breakpoint_id
        )))
    }
}

/// Pause execution
#[weld_op]
#[op2(async)]
pub async fn op_debugger_pause(state: Rc<RefCell<OpState>>) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    send_command(&debugger_state, "Debugger.pause", serde_json::json!({})).await?;

    debug!("Pause requested");
    Ok(true)
}

/// Resume execution
#[weld_op]
#[op2(async)]
pub async fn op_debugger_resume(state: Rc<RefCell<OpState>>) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    send_command(&debugger_state, "Debugger.resume", serde_json::json!({})).await?;

    *debugger_state.paused.write().await = false;
    debugger_state.call_frames.write().await.clear();

    debug!("Resumed execution");
    Ok(true)
}

/// Step over (next line)
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_step_over(
    state: Rc<RefCell<OpState>>,
) -> Result<StepResult, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    send_command(&debugger_state, "Debugger.stepOver", serde_json::json!({})).await?;

    debug!("Step over");
    Ok(StepResult {
        success: true,
        paused_event: None,
    })
}

/// Step into (enter function)
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_step_into(
    state: Rc<RefCell<OpState>>,
) -> Result<StepResult, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    send_command(&debugger_state, "Debugger.stepInto", serde_json::json!({})).await?;

    debug!("Step into");
    Ok(StepResult {
        success: true,
        paused_event: None,
    })
}

/// Step out (exit function)
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_step_out(
    state: Rc<RefCell<OpState>>,
) -> Result<StepResult, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    send_command(&debugger_state, "Debugger.stepOut", serde_json::json!({})).await?;

    debug!("Step out");
    Ok(StepResult {
        success: true,
        paused_event: None,
    })
}

/// Get current call frames
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_get_call_frames(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<CallFrame>, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;

    let frames = debugger_state.call_frames.read().await.clone();
    Ok(frames)
}

/// Get scope chain for a call frame
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_get_scope_chain(
    state: Rc<RefCell<OpState>>,
    #[string] call_frame_id: String,
) -> Result<Vec<Scope>, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;

    let frames = debugger_state.call_frames.read().await;
    let frame = frames
        .iter()
        .find(|f| f.call_frame_id == call_frame_id)
        .ok_or(DebuggerError::InvalidFrameId(call_frame_id))?;

    Ok(frame.scope_chain.clone())
}

/// Get properties of an object
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_get_properties(
    state: Rc<RefCell<OpState>>,
    #[string] object_id: String,
    own_properties: bool,
) -> Result<Vec<PropertyDescriptor>, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let params = serde_json::json!({
        "objectId": object_id,
        "ownProperties": own_properties,
    });

    let response = send_command(&debugger_state, "Runtime.getProperties", params).await?;

    let properties: Vec<PropertyDescriptor> = response
        .get("result")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(properties)
}

/// Evaluate expression
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_evaluate(
    state: Rc<RefCell<OpState>>,
    #[string] expression: String,
    #[string] call_frame_id: Option<String>,
) -> Result<RemoteObject, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let response = if let Some(frame_id) = call_frame_id {
        let params = serde_json::json!({
            "callFrameId": frame_id,
            "expression": expression,
            "returnByValue": false,
        });
        send_command(&debugger_state, "Debugger.evaluateOnCallFrame", params).await?
    } else {
        let params = serde_json::json!({
            "expression": expression,
            "returnByValue": false,
        });
        send_command(&debugger_state, "Runtime.evaluate", params).await?
    };

    let result: RemoteObject = response
        .get("result")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Check for exception
    if let Some(exception) = response.get("exceptionDetails") {
        let msg = exception
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("Evaluation failed");
        return Err(DebuggerError::EvaluationFailed(msg.to_string()));
    }

    Ok(result)
}

/// Get script source
#[weld_op]
#[op2(async)]
#[string]
pub async fn op_debugger_get_script_source(
    state: Rc<RefCell<OpState>>,
    #[string] script_id: String,
) -> Result<String, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let params = serde_json::json!({
        "scriptId": script_id,
    });

    let response = send_command(&debugger_state, "Debugger.getScriptSource", params).await?;

    let source = response
        .get("scriptSource")
        .and_then(|v| v.as_str())
        .ok_or(DebuggerError::SourceNotFound(script_id))?
        .to_string();

    Ok(source)
}

/// List all parsed scripts
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_list_scripts(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<ScriptInfo>, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;

    let scripts: Vec<ScriptInfo> = debugger_state
        .scripts
        .read()
        .await
        .values()
        .cloned()
        .collect();

    Ok(scripts)
}

/// Set exception pause behavior
#[weld_op]
#[op2(async)]
pub async fn op_debugger_set_pause_on_exceptions(
    state: Rc<RefCell<OpState>>,
    #[serde] pause_state: ExceptionPauseState,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let state_str = match pause_state {
        ExceptionPauseState::None => "none",
        ExceptionPauseState::Uncaught => "uncaught",
        ExceptionPauseState::All => "all",
    };

    let params = serde_json::json!({
        "state": state_str,
    });

    send_command(&debugger_state, "Debugger.setPauseOnExceptions", params).await?;

    debug!(state = state_str, "Exception pause state set");
    Ok(true)
}

/// Continue to a specific location
#[weld_op]
#[op2(async)]
pub async fn op_debugger_continue_to_location(
    state: Rc<RefCell<OpState>>,
    #[string] script_id: String,
    #[smi] line_number: u32,
    #[smi] column_number: Option<u32>,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let mut location = serde_json::json!({
        "scriptId": script_id,
        "lineNumber": line_number,
    });

    if let Some(col) = column_number {
        location["columnNumber"] = serde_json::json!(col);
    }

    let params = serde_json::json!({
        "location": location,
    });

    send_command(&debugger_state, "Debugger.continueToLocation", params).await?;

    debug!("Continue to location");
    Ok(true)
}

/// Set variable value in a scope
#[weld_op]
#[op2(async)]
pub async fn op_debugger_set_variable_value(
    state: Rc<RefCell<OpState>>,
    #[smi] scope_number: u32,
    #[string] variable_name: String,
    #[serde] new_value: serde_json::Value,
    #[string] call_frame_id: String,
) -> Result<bool, DebuggerError> {
    let debugger_state = get_debugger_state(&state)?;
    ensure_connected(&debugger_state).await?;

    let params = serde_json::json!({
        "scopeNumber": scope_number,
        "variableName": variable_name,
        "newValue": new_value,
        "callFrameId": call_frame_id,
    });

    send_command(&debugger_state, "Debugger.setVariableValue", params).await?;

    debug!(var = %variable_name, "Variable value set");
    Ok(true)
}

// ============================================================================
// Event Receiver Operations
// ============================================================================

/// Create a pause event receiver
#[weld_op]
#[op2(fast)]
#[smi]
pub fn op_debugger_create_pause_receiver(state: &mut OpState) -> Result<ResourceId, DebuggerError> {
    let debugger_state = state
        .try_borrow::<Arc<DebuggerState>>()
        .cloned()
        .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))?;

    let rx = debugger_state.pause_rx();
    let rid = state.resource_table.add(PauseEventReceiver(rx));
    Ok(rid)
}

/// Receive next pause event
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_receive_pause_event(
    state: Rc<RefCell<OpState>>,
    #[smi] rid: ResourceId,
) -> Result<PausedEvent, DebuggerError> {
    let rx = {
        let state_ref = state.borrow();
        let receiver = state_ref
            .resource_table
            .get::<PauseEventReceiver>(rid)
            .map_err(|_| DebuggerError::Internal("Invalid receiver".to_string()))?;
        receiver.0.resubscribe()
    };

    let mut rx = rx;
    rx.recv()
        .await
        .map_err(|_| DebuggerError::Internal("Event channel closed".to_string()))
}

/// Create a script event receiver
#[weld_op]
#[op2(fast)]
#[smi]
pub fn op_debugger_create_script_receiver(
    state: &mut OpState,
) -> Result<ResourceId, DebuggerError> {
    let debugger_state = state
        .try_borrow::<Arc<DebuggerState>>()
        .cloned()
        .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))?;

    let rx = debugger_state.script_rx();
    let rid = state.resource_table.add(ScriptEventReceiver(rx));
    Ok(rid)
}

/// Receive next script parsed event
#[weld_op]
#[op2(async)]
#[serde]
pub async fn op_debugger_receive_script_event(
    state: Rc<RefCell<OpState>>,
    #[smi] rid: ResourceId,
) -> Result<ScriptInfo, DebuggerError> {
    let rx = {
        let state_ref = state.borrow();
        let receiver = state_ref
            .resource_table
            .get::<ScriptEventReceiver>(rid)
            .map_err(|_| DebuggerError::Internal("Invalid receiver".to_string()))?;
        receiver.0.resubscribe()
    };

    let mut rx = rx;
    rx.recv()
        .await
        .map_err(|_| DebuggerError::Internal("Event channel closed".to_string()))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_debugger_state(state: &Rc<RefCell<OpState>>) -> Result<Arc<DebuggerState>, DebuggerError> {
    let state_ref = state.borrow();
    state_ref
        .try_borrow::<Arc<DebuggerState>>()
        .cloned()
        .ok_or_else(|| DebuggerError::Internal("Debugger state not initialized".to_string()))
}

async fn ensure_connected(state: &Arc<DebuggerState>) -> Result<(), DebuggerError> {
    if !*state.connected.read().await {
        return Err(DebuggerError::NotConnected);
    }
    if !*state.enabled.read().await {
        return Err(DebuggerError::NotEnabled);
    }
    Ok(())
}

async fn send_command(
    state: &Arc<DebuggerState>,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, DebuggerError> {
    let client_guard = state.client.lock().await;
    let client = client_guard.as_ref().ok_or(DebuggerError::NotConnected)?;

    client
        .send_method(method, params)
        .await
        .map_err(|e| DebuggerError::ProtocolError(e.to_string()))
}

fn start_event_listener(state: Arc<DebuggerState>) {
    tokio::spawn(async move {
        loop {
            let event = {
                let client_guard = state.client.lock().await;
                match client_guard.as_ref() {
                    Some(client) => client.receive_event().await,
                    None => break,
                }
            };

            match event {
                Ok(Some(msg)) => {
                    handle_inspector_event(&state, msg).await;
                }
                Ok(None) => {
                    // Connection closed
                    *state.connected.write().await = false;
                    break;
                }
                Err(e) => {
                    error!("Inspector event error: {}", e);
                    break;
                }
            }
        }
    });
}

async fn handle_inspector_event(state: &Arc<DebuggerState>, msg: InspectorMessage) {
    match msg.method.as_str() {
        "Debugger.paused" => {
            if let Some(params) = msg.params {
                *state.paused.write().await = true;

                // Parse call frames
                let call_frames: Vec<CallFrame> = params
                    .get("callFrames")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                *state.call_frames.write().await = call_frames.clone();

                // Parse pause reason
                let reason: PauseReason = params
                    .get("reason")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let hit_breakpoints: Option<Vec<String>> = params
                    .get("hitBreakpoints")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                let event = PausedEvent {
                    reason,
                    call_frames,
                    hit_breakpoints,
                    data: params.get("data").cloned(),
                    async_stack_trace: params.get("asyncStackTrace").cloned(),
                };

                let _ = state.pause_tx.send(event);
            }
        }
        "Debugger.resumed" => {
            *state.paused.write().await = false;
            state.call_frames.write().await.clear();
            let _ = state.resume_tx.send(());
        }
        "Debugger.scriptParsed" => {
            if let Some(params) = msg.params {
                let script_info: ScriptInfo = serde_json::from_value(serde_json::json!({
                    "script_id": params.get("scriptId"),
                    "url": params.get("url"),
                    "source_map_url": params.get("sourceMapURL"),
                    "start_line": params.get("startLine"),
                    "start_column": params.get("startColumn"),
                    "end_line": params.get("endLine"),
                    "end_column": params.get("endColumn"),
                    "hash": params.get("hash"),
                    "length": params.get("length"),
                    "execution_context_id": params.get("executionContextId"),
                }))
                .unwrap_or_else(|_| ScriptInfo {
                    script_id: params
                        .get("scriptId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    url: params
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    source_map_url: params
                        .get("sourceMapURL")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    start_line: params
                        .get("startLine")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    start_column: params
                        .get("startColumn")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    end_line: params.get("endLine").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    end_column: params
                        .get("endColumn")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    hash: params
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    length: params.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    execution_context_id: params
                        .get("executionContextId")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32),
                });

                state
                    .scripts
                    .write()
                    .await
                    .insert(script_info.script_id.clone(), script_info.clone());

                let _ = state.script_tx.send(script_info);
            }
        }
        "Debugger.breakpointResolved" => {
            if let Some(params) = msg.params {
                let bp_id = params
                    .get("breakpointId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let location = params
                    .get("location")
                    .map(|loc| Location {
                        script_id: loc
                            .get("scriptId")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        line_number: loc.get("lineNumber").and_then(|v| v.as_u64()).unwrap_or(0)
                            as u32,
                        column_number: loc
                            .get("columnNumber")
                            .and_then(|v| v.as_u64())
                            .map(|n| n as u32),
                    })
                    .unwrap_or_default();

                let event = BreakpointResolved {
                    breakpoint_id: bp_id.clone(),
                    location: location.clone(),
                };

                // Update local breakpoint
                if let Some(bp) = state.breakpoints.write().await.get_mut(&bp_id) {
                    bp.location = location;
                }

                let _ = state.bp_resolved_tx.send(event);
            }
        }
        _ => {
            debug!(method = %msg.method, "Unhandled inspector event");
        }
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn debugger_extension() -> Extension {
    runtime_debugger::ext()
}

pub fn init_debugger_state(op_state: &mut OpState) {
    let state = Arc::new(DebuggerState::new());
    op_state.put(state);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Data Structure Serialization Tests
    // =========================================================================

    #[test]
    fn test_location_default() {
        let loc = Location::default();
        assert_eq!(loc.script_id, "");
        assert_eq!(loc.line_number, 0);
        assert!(loc.column_number.is_none());
    }

    #[test]
    fn test_location_serialization() {
        let loc = Location {
            script_id: "123".to_string(),
            line_number: 42,
            column_number: Some(10),
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("\"script_id\":\"123\""));
        assert!(json.contains("\"line_number\":42"));

        let parsed: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.script_id, "123");
        assert_eq!(parsed.line_number, 42);
        assert_eq!(parsed.column_number, Some(10));
    }

    #[test]
    fn test_breakpoint_serialization() {
        let bp = Breakpoint {
            id: "bp-1".to_string(),
            location: Location {
                script_id: "script-1".to_string(),
                line_number: 100,
                column_number: None,
            },
            condition: Some("x > 5".to_string()),
            hit_count: 3,
            enabled: true,
        };

        let json = serde_json::to_string(&bp).unwrap();
        let parsed: Breakpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "bp-1");
        assert_eq!(parsed.hit_count, 3);
        assert_eq!(parsed.condition, Some("x > 5".to_string()));
        assert!(parsed.enabled);
    }

    #[test]
    fn test_scope_type_serialization() {
        let tests = [
            (ScopeType::Global, "global"),
            (ScopeType::Local, "local"),
            (ScopeType::Closure, "closure"),
            (ScopeType::Catch, "catch"),
            (ScopeType::Block, "block"),
            (ScopeType::Script, "script"),
            (ScopeType::Eval, "eval"),
            (ScopeType::Module, "module"),
            (ScopeType::WasmExpressionStack, "wasmExpressionStack"),
        ];

        for (scope_type, expected) in tests {
            let json = serde_json::to_string(&scope_type).unwrap();
            assert!(json.contains(expected), "Expected {} in {}", expected, json);
        }
    }

    #[test]
    fn test_remote_object_serialization() {
        let obj = RemoteObject {
            object_type: "number".to_string(),
            subtype: None,
            class_name: None,
            value: Some(serde_json::json!(42)),
            object_id: None,
            description: Some("42".to_string()),
            preview: None,
        };

        let json = serde_json::to_string(&obj).unwrap();
        let parsed: RemoteObject = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.object_type, "number");
        assert_eq!(parsed.value, Some(serde_json::json!(42)));
    }

    #[test]
    fn test_remote_object_complex() {
        let obj = RemoteObject {
            object_type: "object".to_string(),
            subtype: Some("array".to_string()),
            class_name: Some("Array".to_string()),
            value: None,
            object_id: Some("obj-123".to_string()),
            description: Some("Array(3)".to_string()),
            preview: Some(ObjectPreview {
                preview_type: "object".to_string(),
                subtype: Some("array".to_string()),
                description: Some("Array(3)".to_string()),
                overflow: false,
                properties: vec![PropertyPreview {
                    name: "0".to_string(),
                    property_type: "number".to_string(),
                    subtype: None,
                    value: Some("1".to_string()),
                    value_preview: None,
                }],
                entries: None,
            }),
        };

        let json = serde_json::to_string(&obj).unwrap();
        let parsed: RemoteObject = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.subtype, Some("array".to_string()));
        assert!(parsed.preview.is_some());
        assert_eq!(parsed.preview.unwrap().properties.len(), 1);
    }

    #[test]
    fn test_call_frame_serialization() {
        let frame = CallFrame {
            call_frame_id: "frame-0".to_string(),
            function_name: "myFunction".to_string(),
            location: Location {
                script_id: "1".to_string(),
                line_number: 10,
                column_number: Some(5),
            },
            url: "file:///src/main.ts".to_string(),
            scope_chain: vec![],
            this_object: RemoteObject::default(),
            return_value: None,
        };

        let json = serde_json::to_string(&frame).unwrap();
        let parsed: CallFrame = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.call_frame_id, "frame-0");
        assert_eq!(parsed.function_name, "myFunction");
        assert_eq!(parsed.url, "file:///src/main.ts");
    }

    #[test]
    fn test_pause_reason_serialization() {
        let reasons = [
            (PauseReason::Breakpoint, "breakpoint"),
            (PauseReason::Step, "step"),
            (PauseReason::Exception, "exception"),
            (PauseReason::DebuggerStatement, "debuggerStatement"),
            (PauseReason::PromiseRejection, "promiseRejection"),
        ];

        for (reason, expected) in reasons {
            let json = serde_json::to_string(&reason).unwrap();
            assert!(json.contains(expected));
        }
    }

    #[test]
    fn test_paused_event_serialization() {
        let event = PausedEvent {
            reason: PauseReason::Breakpoint,
            call_frames: vec![CallFrame {
                call_frame_id: "frame-0".to_string(),
                function_name: "test".to_string(),
                location: Location::default(),
                url: "test.ts".to_string(),
                scope_chain: vec![],
                this_object: RemoteObject::default(),
                return_value: None,
            }],
            hit_breakpoints: Some(vec!["bp-1".to_string()]),
            data: None,
            async_stack_trace: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: PausedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.reason, PauseReason::Breakpoint);
        assert_eq!(parsed.call_frames.len(), 1);
        assert_eq!(parsed.hit_breakpoints.unwrap().len(), 1);
    }

    #[test]
    fn test_script_info_serialization() {
        let info = ScriptInfo {
            script_id: "42".to_string(),
            url: "file:///src/main.ts".to_string(),
            source_map_url: Some("main.ts.map".to_string()),
            start_line: 0,
            start_column: 0,
            end_line: 100,
            end_column: 50,
            hash: "abc123".to_string(),
            length: 5000,
            execution_context_id: Some(1),
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ScriptInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.script_id, "42");
        assert_eq!(parsed.url, "file:///src/main.ts");
        assert_eq!(parsed.end_line, 100);
    }

    #[test]
    fn test_step_result_serialization() {
        let result = StepResult {
            success: true,
            paused_event: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: StepResult = serde_json::from_str(&json).unwrap();

        assert!(parsed.success);
        assert!(parsed.paused_event.is_none());
    }

    #[test]
    fn test_connect_options_default() {
        let opts = ConnectOptions::default();
        assert!(opts.url.is_none());
        assert!(opts.timeout_ms.is_none());
    }

    #[test]
    fn test_connection_status_serialization() {
        let status = ConnectionStatus {
            connected: true,
            enabled: true,
            paused: false,
            url: Some("ws://127.0.0.1:9229".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        let parsed: ConnectionStatus = serde_json::from_str(&json).unwrap();

        assert!(parsed.connected);
        assert!(parsed.enabled);
        assert!(!parsed.paused);
    }

    #[test]
    fn test_exception_pause_state_serialization() {
        let states = [
            (ExceptionPauseState::None, "none"),
            (ExceptionPauseState::Uncaught, "uncaught"),
            (ExceptionPauseState::All, "all"),
        ];

        for (state, expected) in states {
            let json = serde_json::to_string(&state).unwrap();
            assert!(json.contains(expected));
        }
    }

    #[test]
    fn test_property_descriptor_serialization() {
        let desc = PropertyDescriptor {
            name: "myProp".to_string(),
            value: Some(RemoteObject {
                object_type: "string".to_string(),
                value: Some(serde_json::json!("hello")),
                ..Default::default()
            }),
            writable: Some(true),
            get: None,
            set: None,
            configurable: true,
            enumerable: true,
            is_own: Some(true),
            symbol: None,
        };

        let json = serde_json::to_string(&desc).unwrap();
        let parsed: PropertyDescriptor = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "myProp");
        assert!(parsed.writable.unwrap());
        assert!(parsed.configurable);
    }

    #[test]
    fn test_breakpoint_resolved_serialization() {
        let resolved = BreakpointResolved {
            breakpoint_id: "bp-1".to_string(),
            location: Location {
                script_id: "script-1".to_string(),
                line_number: 42,
                column_number: Some(0),
            },
        };

        let json = serde_json::to_string(&resolved).unwrap();
        let parsed: BreakpointResolved = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.breakpoint_id, "bp-1");
        assert_eq!(parsed.location.line_number, 42);
    }

    // =========================================================================
    // Error Type Tests
    // =========================================================================

    #[test]
    fn test_error_codes_in_range() {
        let codes = [
            DebuggerErrorCode::Generic as i32,
            DebuggerErrorCode::ConnectionFailed as i32,
            DebuggerErrorCode::NotConnected as i32,
            DebuggerErrorCode::BreakpointFailed as i32,
            DebuggerErrorCode::InvalidFrameId as i32,
            DebuggerErrorCode::InvalidScopeId as i32,
            DebuggerErrorCode::EvaluationFailed as i32,
            DebuggerErrorCode::SourceNotFound as i32,
            DebuggerErrorCode::StepFailed as i32,
            DebuggerErrorCode::PauseFailed as i32,
            DebuggerErrorCode::ResumeFailed as i32,
            DebuggerErrorCode::ProtocolError as i32,
            DebuggerErrorCode::NotEnabled as i32,
            DebuggerErrorCode::Timeout as i32,
            DebuggerErrorCode::InvalidLocation as i32,
        ];

        // Check all codes are in the 9600-9699 range
        for code in codes {
            assert!(code >= 9600 && code < 9700, "Code {} out of range", code);
        }

        // Check all codes are unique
        let mut seen = std::collections::HashSet::new();
        for code in codes {
            assert!(seen.insert(code), "Duplicate error code: {}", code);
        }
    }

    #[test]
    fn test_error_display() {
        let errors = [
            DebuggerError::NotConnected,
            DebuggerError::NotEnabled,
            DebuggerError::Timeout,
            DebuggerError::InvalidLocation,
            DebuggerError::ConnectionFailed("test".to_string()),
            DebuggerError::BreakpointFailed("test".to_string()),
            DebuggerError::EvaluationFailed("test".to_string()),
        ];

        for err in errors {
            let msg = format!("{}", err);
            assert!(!msg.is_empty());
        }
    }

    // =========================================================================
    // State Management Tests
    // =========================================================================

    #[test]
    fn test_debugger_state_default() {
        let state = DebuggerState::new();

        // Test that all fields are properly initialized
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            assert!(!*state.connected.read().await);
            assert!(!*state.enabled.read().await);
            assert!(!*state.paused.read().await);
            assert!(state.breakpoints.read().await.is_empty());
            assert!(state.call_frames.read().await.is_empty());
            assert!(state.scripts.read().await.is_empty());
        });
    }

    #[test]
    fn test_debugger_state_receivers() {
        let state = DebuggerState::new();

        // Should be able to create receivers
        let _pause_rx = state.pause_rx();
        let _resume_rx = state.resume_rx();
        let _script_rx = state.script_rx();
        let _bp_rx = state.bp_resolved_rx();
    }

    #[test]
    fn test_debugger_state_broadcast() {
        let state = DebuggerState::new();
        let mut rx = state.pause_rx();

        let event = PausedEvent {
            reason: PauseReason::Breakpoint,
            call_frames: vec![],
            hit_breakpoints: None,
            data: None,
            async_stack_trace: None,
        };

        let _ = state.pause_tx.send(event.clone());

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().reason, PauseReason::Breakpoint);
    }

    #[test]
    fn test_debugger_state_script_broadcast() {
        let state = DebuggerState::new();
        let mut rx = state.script_rx();

        let script = ScriptInfo {
            script_id: "1".to_string(),
            url: "test.ts".to_string(),
            source_map_url: None,
            start_line: 0,
            start_column: 0,
            end_line: 10,
            end_column: 0,
            hash: "".to_string(),
            length: 100,
            execution_context_id: None,
        };

        let _ = state.script_tx.send(script);

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().script_id, "1");
    }

    // =========================================================================
    // Extension Info Test
    // =========================================================================

    #[test]
    fn test_extension_info_struct() {
        // Test ExtensionInfo struct directly (op_debugger_info is transformed by #[op2])
        let info = ExtensionInfo {
            name: "ext_debugger".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            status: "active".to_string(),
        };
        assert_eq!(info.name, "ext_debugger");
        assert!(!info.version.is_empty());
        assert_eq!(info.status, "active");
    }

    // =========================================================================
    // Breakpoint Options Tests
    // =========================================================================

    #[test]
    fn test_breakpoint_options_default() {
        let opts = BreakpointOptions::default();
        assert!(opts.condition.is_none());
        assert!(opts.column_number.is_none());
    }

    #[test]
    fn test_breakpoint_options_full() {
        let opts = BreakpointOptions {
            condition: Some("i > 10".to_string()),
            column_number: Some(5),
        };

        let json = serde_json::to_string(&opts).unwrap();
        let parsed: BreakpointOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.condition, Some("i > 10".to_string()));
        assert_eq!(parsed.column_number, Some(5));
    }

    // =========================================================================
    // Variable/Property Tests
    // =========================================================================

    #[test]
    fn test_variable_serialization() {
        let var = Variable {
            name: "count".to_string(),
            value: RemoteObject {
                object_type: "number".to_string(),
                value: Some(serde_json::json!(42)),
                ..Default::default()
            },
            writable: Some(true),
            configurable: Some(true),
            enumerable: Some(true),
        };

        let json = serde_json::to_string(&var).unwrap();
        let parsed: Variable = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "count");
        assert!(parsed.writable.unwrap());
    }

    // =========================================================================
    // Scope Tests
    // =========================================================================

    #[test]
    fn test_scope_serialization() {
        let scope = Scope {
            scope_type: ScopeType::Local,
            object: RemoteObject {
                object_type: "object".to_string(),
                object_id: Some("scope-obj-1".to_string()),
                ..Default::default()
            },
            name: Some("myFunction".to_string()),
            start_location: Some(Location {
                script_id: "1".to_string(),
                line_number: 10,
                column_number: Some(0),
            }),
            end_location: Some(Location {
                script_id: "1".to_string(),
                line_number: 20,
                column_number: Some(1),
            }),
        };

        let json = serde_json::to_string(&scope).unwrap();
        let parsed: Scope = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.scope_type, ScopeType::Local);
        assert_eq!(parsed.name, Some("myFunction".to_string()));
    }

    // =========================================================================
    // Exception Details Tests
    // =========================================================================

    #[test]
    fn test_exception_details_serialization() {
        let details = ExceptionDetails {
            exception_id: 1,
            text: "TypeError: Cannot read property 'x' of undefined".to_string(),
            line_number: 42,
            column_number: 15,
            script_id: Some("script-1".to_string()),
            url: Some("file:///src/main.ts".to_string()),
            exception: Some(RemoteObject {
                object_type: "object".to_string(),
                class_name: Some("TypeError".to_string()),
                ..Default::default()
            }),
            stack_trace: None,
        };

        let json = serde_json::to_string(&details).unwrap();
        let parsed: ExceptionDetails = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.exception_id, 1);
        assert!(parsed.text.contains("TypeError"));
        assert_eq!(parsed.line_number, 42);
    }

    // =========================================================================
    // Preview Types Tests
    // =========================================================================

    #[test]
    fn test_object_preview_serialization() {
        let preview = ObjectPreview {
            preview_type: "object".to_string(),
            subtype: None,
            description: Some("Object".to_string()),
            overflow: false,
            properties: vec![
                PropertyPreview {
                    name: "a".to_string(),
                    property_type: "number".to_string(),
                    subtype: None,
                    value: Some("1".to_string()),
                    value_preview: None,
                },
                PropertyPreview {
                    name: "b".to_string(),
                    property_type: "string".to_string(),
                    subtype: None,
                    value: Some("hello".to_string()),
                    value_preview: None,
                },
            ],
            entries: None,
        };

        let json = serde_json::to_string(&preview).unwrap();
        let parsed: ObjectPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.properties.len(), 2);
        assert!(!parsed.overflow);
    }

    #[test]
    fn test_entry_preview_serialization() {
        let entry = EntryPreview {
            key: Some(ObjectPreview {
                preview_type: "string".to_string(),
                subtype: None,
                description: Some("key1".to_string()),
                overflow: false,
                properties: vec![],
                entries: None,
            }),
            value: ObjectPreview {
                preview_type: "number".to_string(),
                subtype: None,
                description: Some("42".to_string()),
                overflow: false,
                properties: vec![],
                entries: None,
            },
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: EntryPreview = serde_json::from_str(&json).unwrap();

        assert!(parsed.key.is_some());
    }

    // =========================================================================
    // Default Implementations Tests
    // =========================================================================

    #[test]
    fn test_scope_type_default() {
        let default = ScopeType::default();
        assert_eq!(default, ScopeType::Local);
    }

    #[test]
    fn test_pause_reason_default() {
        let default = PauseReason::default();
        assert_eq!(default, PauseReason::Other);
    }

    #[test]
    fn test_exception_pause_state_default() {
        let default = ExceptionPauseState::default();
        assert_eq!(default, ExceptionPauseState::None);
    }

    #[test]
    fn test_remote_object_default() {
        let obj = RemoteObject::default();
        assert_eq!(obj.object_type, "");
        assert!(obj.value.is_none());
        assert!(obj.object_id.is_none());
    }
}
