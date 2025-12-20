//! Chrome DevTools Protocol types and routing for custom Forge domains.
//!
//! Custom domains:
//! - `Forge.Monitor` - System and runtime metrics
//! - `Forge.Trace` - Span and trace data
//! - `Forge.Signals` - OS signal handling
//! - `Forge.Runtime` - App info, windows, IPC

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod router;

// ============================================================================
// CDP Message Types
// ============================================================================

/// CDP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpMessage {
    /// Request ID
    pub id: u64,
    /// Method name (e.g., "Forge.Monitor.getMetrics")
    pub method: String,
    /// Optional parameters
    #[serde(default)]
    pub params: Option<Value>,
}

/// CDP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpResponse {
    /// Request ID (matches the request)
    pub id: u64,
    /// Result on success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CdpError>,
}

/// CDP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl CdpResponse {
    /// Create a success response
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(CdpError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// CDP event (server -> client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpEvent {
    /// Event method (e.g., "Forge.Monitor.metricsUpdated")
    pub method: String,
    /// Event parameters
    pub params: Value,
}

impl CdpEvent {
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

// ============================================================================
// CDP Error Codes (using Chrome DevTools Protocol conventions)
// ============================================================================

pub mod error_codes {
    /// Parse error - Invalid JSON
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Server error (reserved for implementation-defined errors)
    pub const SERVER_ERROR: i32 = -32000;
}
