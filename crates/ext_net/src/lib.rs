//! host:net extension - Network operations for Forge apps
//!
//! Provides HTTP fetch with capability-based security.

use deno_core::{op2, Extension, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tracing::debug;
use url::Url;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for network operations (for machine-readable errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NetErrorCode {
    /// Generic IO error
    Io = 1000,
    /// Permission denied by capability system
    PermissionDenied = 1001,
    /// Invalid URL format
    InvalidUrl = 1002,
    /// Request timeout
    Timeout = 1003,
    /// Connection failed
    ConnectionFailed = 1004,
    /// HTTP error response
    HttpError = 1005,
    /// Request building failed
    RequestBuildError = 1006,
}

/// Custom error type for Net operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum NetError {
    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    Io { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Invalid URL: {message}")]
    #[class(generic)]
    InvalidUrl { code: u32, message: String },

    #[error("[{code}] Request timeout: {message}")]
    #[class(generic)]
    Timeout { code: u32, message: String },

    #[error("[{code}] Connection failed: {message}")]
    #[class(generic)]
    ConnectionFailed { code: u32, message: String },

    #[error("[{code}] HTTP error: {message}")]
    #[class(generic)]
    HttpError { code: u32, message: String },

    #[error("[{code}] Request build error: {message}")]
    #[class(generic)]
    RequestBuildError { code: u32, message: String },
}

impl NetError {
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            code: NetErrorCode::Io as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: NetErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn invalid_url(message: impl Into<String>) -> Self {
        Self::InvalidUrl {
            code: NetErrorCode::InvalidUrl as u32,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            code: NetErrorCode::Timeout as u32,
            message: message.into(),
        }
    }

    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            code: NetErrorCode::ConnectionFailed as u32,
            message: message.into(),
        }
    }

    pub fn http_error(message: impl Into<String>) -> Self {
        Self::HttpError {
            code: NetErrorCode::HttpError as u32,
            message: message.into(),
        }
    }

    pub fn request_build_error(message: impl Into<String>) -> Self {
        Self::RequestBuildError {
            code: NetErrorCode::RequestBuildError as u32,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for NetError {
    fn from(e: std::io::Error) -> Self {
        Self::io(e.to_string())
    }
}

impl From<reqwest::Error> for NetError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::timeout(e.to_string())
        } else if e.is_connect() {
            Self::connection_failed(e.to_string())
        } else if e.is_request() {
            Self::request_build_error(e.to_string())
        } else if e.is_status() {
            Self::http_error(e.to_string())
        } else {
            Self::io(e.to_string())
        }
    }
}

impl From<url::ParseError> for NetError {
    fn from(e: url::ParseError) -> Self {
        Self::invalid_url(e.to_string())
    }
}

// ============================================================================
// Types
// ============================================================================

/// Options for HTTP fetch
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FetchOpts {
    pub method: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub timeout_ms: Option<u64>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String,
    pub ok: bool,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for network operations
pub trait NetCapabilityChecker: Send + Sync {
    fn check_connect(&self, host: &str) -> Result<(), String>;
    fn check_listen(&self, port: u16) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveNetChecker;

impl NetCapabilityChecker for PermissiveNetChecker {
    fn check_connect(&self, _host: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_listen(&self, _port: u16) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store in OpState
pub struct NetCapabilities {
    pub checker: Arc<dyn NetCapabilityChecker>,
}

impl Default for NetCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveNetChecker),
        }
    }
}

/// Shared HTTP client (reused for connection pooling)
pub struct NetHttpClient {
    pub client: reqwest::Client,
}

impl Default for NetHttpClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Forge/0.1")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract host from URL for capability checking
fn extract_host(url: &str) -> Result<String, NetError> {
    let parsed = Url::parse(url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| NetError::invalid_url("URL has no host"))?;

    // Include port if non-default
    let port = parsed.port();
    let default_port = match parsed.scheme() {
        "http" => Some(80),
        "https" => Some(443),
        _ => None,
    };

    if let Some(p) = port {
        if port != default_port {
            return Ok(format!("{}:{}", host, p));
        }
    }
    Ok(host.to_string())
}

/// Check net connect capability
fn check_net_connect(state: &OpState, host: &str) -> Result<(), NetError> {
    if let Some(caps) = state.try_borrow::<NetCapabilities>() {
        caps.checker
            .check_connect(host)
            .map_err(NetError::permission_denied)
    } else {
        Ok(())
    }
}

/// Get or create HTTP client
fn get_http_client(state: &mut OpState) -> reqwest::Client {
    if let Some(client_state) = state.try_borrow::<NetHttpClient>() {
        client_state.client.clone()
    } else {
        let client_state = NetHttpClient::default();
        let client = client_state.client.clone();
        state.put(client_state);
        client
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Fetch a URL with full HTTP support
#[op2(async)]
#[serde]
async fn op_net_fetch(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<FetchOpts>,
) -> Result<FetchResponse, NetError> {
    let opts = opts.unwrap_or_default();

    // Extract host and check capabilities
    let host = extract_host(&url)?;
    {
        let s = state.borrow();
        check_net_connect(&s, &host)?;
    }

    debug!(url = %url, method = ?opts.method, "net.fetch");

    // Get HTTP client
    let client = {
        let mut s = state.borrow_mut();
        get_http_client(&mut s)
    };

    // Build request
    let method = opts.method.as_deref().unwrap_or("GET").to_uppercase();
    let mut request_builder = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => {
            return Err(NetError::request_build_error(format!(
                "Unsupported method: {}",
                method
            )));
        }
    };

    // Add headers
    if let Some(headers) = opts.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }
    }

    // Add body
    if let Some(body) = opts.body {
        request_builder = request_builder.body(body);
    }

    // Set timeout
    if let Some(timeout_ms) = opts.timeout_ms {
        request_builder = request_builder.timeout(std::time::Duration::from_millis(timeout_ms));
    }

    // Execute request
    let response = request_builder.send().await?;

    // Extract response data
    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();
    let ok = response.status().is_success();
    let final_url = response.url().to_string();

    // Convert headers
    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    // Read body as text
    let body = response.text().await?;

    debug!(status = status, body_len = body.len(), "net.fetch complete");

    Ok(FetchResponse {
        status,
        status_text,
        headers,
        body,
        url: final_url,
        ok,
    })
}

/// Fetch and return raw bytes (for binary data)
#[op2(async)]
#[serde]
async fn op_net_fetch_bytes(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<FetchOpts>,
) -> Result<FetchBytesResponse, NetError> {
    let opts = opts.unwrap_or_default();

    // Extract host and check capabilities
    let host = extract_host(&url)?;
    {
        let s = state.borrow();
        check_net_connect(&s, &host)?;
    }

    debug!(url = %url, "net.fetch_bytes");

    // Get HTTP client
    let client = {
        let mut s = state.borrow_mut();
        get_http_client(&mut s)
    };

    // Build request
    let method = opts.method.as_deref().unwrap_or("GET").to_uppercase();
    let mut request_builder = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => {
            return Err(NetError::request_build_error(format!(
                "Unsupported method: {}",
                method
            )));
        }
    };

    // Add headers
    if let Some(headers) = opts.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(&key, &value);
        }
    }

    // Add body
    if let Some(body) = opts.body {
        request_builder = request_builder.body(body);
    }

    // Set timeout
    if let Some(timeout_ms) = opts.timeout_ms {
        request_builder = request_builder.timeout(std::time::Duration::from_millis(timeout_ms));
    }

    // Execute request
    let response = request_builder.send().await?;

    // Extract response data
    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();
    let ok = response.status().is_success();
    let final_url = response.url().to_string();

    // Convert headers
    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(key.to_string(), v.to_string());
        }
    }

    // Read body as bytes
    let body = response.bytes().await?.to_vec();

    debug!(
        status = status,
        body_len = body.len(),
        "net.fetch_bytes complete"
    );

    Ok(FetchBytesResponse {
        status,
        status_text,
        headers,
        body,
        url: final_url,
        ok,
    })
}

/// Response with raw bytes body
#[derive(Debug, Clone, Serialize)]
pub struct FetchBytesResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
    pub ok: bool,
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize net state in OpState
pub fn init_net_state(op_state: &mut OpState, capabilities: Option<Arc<dyn NetCapabilityChecker>>) {
    // Initialize HTTP client
    op_state.put(NetHttpClient::default());

    // Set capabilities
    if let Some(caps) = capabilities {
        op_state.put(NetCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

deno_core::extension!(
    host_net,
    ops = [op_net_fetch, op_net_fetch_bytes,],
    esm_entry_point = "ext:host_net/init.js",
    esm = ["ext:host_net/init.js" = "js/init.js"]
);

pub fn net_extension() -> Extension {
    host_net::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(
            extract_host("https://example.com/path").unwrap(),
            "example.com"
        );
        assert_eq!(
            extract_host("https://example.com:8443/path").unwrap(),
            "example.com:8443"
        );
        assert_eq!(
            extract_host("http://localhost:3000").unwrap(),
            "localhost:3000"
        );
        assert_eq!(
            extract_host("https://api.example.com:443/v1").unwrap(),
            "api.example.com"
        );
    }

    #[test]
    fn test_error_codes() {
        let err = NetError::permission_denied("test");
        match err {
            NetError::PermissionDenied { code, .. } => {
                assert_eq!(code, NetErrorCode::PermissionDenied as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }
}
