//! runtime:net extension - Network operations for Forge apps
//!
//! Provides HTTP fetch and WebSocket support with capability-based security.

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
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
    /// WebSocket connection error
    WebSocketConnect = 1007,
    /// WebSocket send error
    WebSocketSend = 1008,
    /// WebSocket receive error
    WebSocketRecv = 1009,
    /// WebSocket close error
    WebSocketClose = 1010,
    /// WebSocket not found
    WebSocketNotFound = 1011,
    /// Streaming error
    StreamError = 1012,
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

    #[error("[{code}] WebSocket connect error: {message}")]
    #[class(generic)]
    WebSocketConnect { code: u32, message: String },

    #[error("[{code}] WebSocket send error: {message}")]
    #[class(generic)]
    WebSocketSend { code: u32, message: String },

    #[error("[{code}] WebSocket receive error: {message}")]
    #[class(generic)]
    WebSocketRecv { code: u32, message: String },

    #[error("[{code}] WebSocket close error: {message}")]
    #[class(generic)]
    WebSocketClose { code: u32, message: String },

    #[error("[{code}] WebSocket not found: {message}")]
    #[class(generic)]
    WebSocketNotFound { code: u32, message: String },

    #[error("[{code}] Stream error: {message}")]
    #[class(generic)]
    StreamError { code: u32, message: String },
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

    pub fn websocket_connect(message: impl Into<String>) -> Self {
        Self::WebSocketConnect {
            code: NetErrorCode::WebSocketConnect as u32,
            message: message.into(),
        }
    }

    pub fn websocket_send(message: impl Into<String>) -> Self {
        Self::WebSocketSend {
            code: NetErrorCode::WebSocketSend as u32,
            message: message.into(),
        }
    }

    pub fn websocket_recv(message: impl Into<String>) -> Self {
        Self::WebSocketRecv {
            code: NetErrorCode::WebSocketRecv as u32,
            message: message.into(),
        }
    }

    pub fn websocket_close(message: impl Into<String>) -> Self {
        Self::WebSocketClose {
            code: NetErrorCode::WebSocketClose as u32,
            message: message.into(),
        }
    }

    pub fn websocket_not_found(message: impl Into<String>) -> Self {
        Self::WebSocketNotFound {
            code: NetErrorCode::WebSocketNotFound as u32,
            message: message.into(),
        }
    }

    pub fn stream_error(message: impl Into<String>) -> Self {
        Self::StreamError {
            code: NetErrorCode::StreamError as u32,
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

impl From<tokio_tungstenite::tungstenite::Error> for NetError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::websocket_connect(e.to_string())
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
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String,
    pub ok: bool,
}

/// WebSocket connection options
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebSocketOpts {
    pub headers: Option<HashMap<String, String>>,
    pub protocols: Option<Vec<String>>,
}

/// WebSocket connection result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketConnection {
    pub id: u64,
    pub url: String,
    pub protocol: Option<String>,
}

/// WebSocket message
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub msg_type: String, // "text", "binary", "ping", "pong", "close"
    pub data: Option<String>,
    pub binary: Option<Vec<u8>>,
}

/// Streaming fetch chunk
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct StreamChunk {
    pub done: bool,
    pub data: Option<Vec<u8>>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for network operations
pub trait NetCapabilityChecker: Send + Sync {
    fn check_connect(&self, runtime: &str) -> Result<(), String>;
    fn check_listen(&self, port: u16) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveNetChecker;

impl NetCapabilityChecker for PermissiveNetChecker {
    fn check_connect(&self, _runtime: &str) -> Result<(), String> {
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

/// Type alias for WebSocket stream
type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// WebSocket connection entry
pub struct WebSocketEntry {
    pub url: String,
    pub protocol: Option<String>,
    pub stream: Arc<Mutex<WsStream>>,
}

/// WebSocket state manager
#[derive(Clone)]
pub struct WebSocketState {
    pub connections: Arc<Mutex<HashMap<u64, WebSocketEntry>>>,
    pub next_id: Arc<Mutex<u64>>,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
}

/// Streaming fetch state
#[derive(Clone)]
pub struct StreamState {
    pub streams: Arc<Mutex<HashMap<u64, StreamEntry>>>,
    pub next_id: Arc<Mutex<u64>>,
}

impl Default for StreamState {
    fn default() -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
}

/// Stream entry for fetch streaming
pub struct StreamEntry {
    pub response: reqwest::Response,
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
#[weld_op(async)]
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
#[weld_op(async)]
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
#[weld_struct]
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
// WebSocket Operations
// ============================================================================

/// Connect to a WebSocket server
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_net_ws_connect(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<WebSocketOpts>,
) -> Result<WebSocketConnection, NetError> {
    let _opts = opts.unwrap_or_default();

    // Parse URL and check capabilities
    let host = extract_host(&url)?;
    {
        let s = state.borrow();
        check_net_connect(&s, &host)?;
    }

    debug!(url = %url, "ws.connect");

    // Connect to WebSocket
    let (ws_stream, response) = connect_async(&url)
        .await
        .map_err(|e| NetError::websocket_connect(e.to_string()))?;

    // Get negotiated protocol
    let protocol = response
        .headers()
        .get("sec-websocket-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get or create WebSocket state
    let ws_state = {
        let mut s = state.borrow_mut();
        if s.try_borrow::<WebSocketState>().is_none() {
            s.put(WebSocketState::default());
        }
        s.borrow::<WebSocketState>().clone()
    };

    // Generate connection ID
    let id = {
        let mut next_id = ws_state.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        id
    };

    // Store connection
    {
        let mut connections = ws_state.connections.lock().await;
        connections.insert(
            id,
            WebSocketEntry {
                url: url.clone(),
                protocol: protocol.clone(),
                stream: Arc::new(Mutex::new(ws_stream)),
            },
        );
    }

    debug!(id = id, "ws.connect complete");

    Ok(WebSocketConnection { id, url, protocol })
}

/// Send a message over WebSocket
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_net_ws_send(
    state: Rc<RefCell<OpState>>,
    #[bigint] id: u64,
    #[serde] message: WebSocketMessage,
) -> Result<(), NetError> {
    // Get WebSocket state
    let ws_state = {
        let s = state.borrow();
        s.try_borrow::<WebSocketState>()
            .ok_or_else(|| NetError::websocket_not_found("No WebSocket state"))?
            .clone()
    };

    // Get the connection
    let stream = {
        let connections = ws_state.connections.lock().await;
        connections
            .get(&id)
            .ok_or_else(|| NetError::websocket_not_found(format!("WebSocket {} not found", id)))?
            .stream
            .clone()
    };

    // Convert message to tungstenite Message
    let msg = match message.msg_type.as_str() {
        "text" => Message::Text(message.data.unwrap_or_default()),
        "binary" => Message::Binary(message.binary.unwrap_or_default()),
        "ping" => Message::Ping(message.binary.unwrap_or_default()),
        "pong" => Message::Pong(message.binary.unwrap_or_default()),
        "close" => Message::Close(None),
        _ => return Err(NetError::websocket_send("Unknown message type")),
    };

    // Send the message
    let mut stream_guard = stream.lock().await;
    stream_guard
        .send(msg)
        .await
        .map_err(|e| NetError::websocket_send(e.to_string()))?;

    debug!(id = id, "ws.send complete");

    Ok(())
}

/// Receive a message from WebSocket
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_net_ws_recv(
    state: Rc<RefCell<OpState>>,
    #[bigint] id: u64,
) -> Result<Option<WebSocketMessage>, NetError> {
    // Get WebSocket state
    let ws_state = {
        let s = state.borrow();
        s.try_borrow::<WebSocketState>()
            .ok_or_else(|| NetError::websocket_not_found("No WebSocket state"))?
            .clone()
    };

    // Get the connection
    let stream = {
        let connections = ws_state.connections.lock().await;
        connections
            .get(&id)
            .ok_or_else(|| NetError::websocket_not_found(format!("WebSocket {} not found", id)))?
            .stream
            .clone()
    };

    // Receive message
    let mut stream_guard = stream.lock().await;
    match stream_guard.next().await {
        Some(Ok(msg)) => {
            let ws_msg = match msg {
                Message::Text(text) => WebSocketMessage {
                    msg_type: "text".to_string(),
                    data: Some(text),
                    binary: None,
                },
                Message::Binary(data) => WebSocketMessage {
                    msg_type: "binary".to_string(),
                    data: None,
                    binary: Some(data),
                },
                Message::Ping(data) => WebSocketMessage {
                    msg_type: "ping".to_string(),
                    data: None,
                    binary: Some(data),
                },
                Message::Pong(data) => WebSocketMessage {
                    msg_type: "pong".to_string(),
                    data: None,
                    binary: Some(data),
                },
                Message::Close(_) => WebSocketMessage {
                    msg_type: "close".to_string(),
                    data: None,
                    binary: None,
                },
                Message::Frame(_) => {
                    // Raw frames are not typically exposed to applications
                    return Ok(None);
                }
            };
            debug!(id = id, msg_type = %ws_msg.msg_type, "ws.recv");
            Ok(Some(ws_msg))
        }
        Some(Err(e)) => Err(NetError::websocket_recv(e.to_string())),
        None => {
            debug!(id = id, "ws.recv: connection closed");
            Ok(None)
        }
    }
}

/// Close a WebSocket connection
#[weld_op(async)]
#[op2(async)]
async fn op_net_ws_close(state: Rc<RefCell<OpState>>, #[bigint] id: u64) -> Result<(), NetError> {
    // Get WebSocket state
    let ws_state = {
        let s = state.borrow();
        s.try_borrow::<WebSocketState>()
            .ok_or_else(|| NetError::websocket_not_found("No WebSocket state"))?
            .clone()
    };

    // Remove and close the connection
    let stream = {
        let mut connections = ws_state.connections.lock().await;
        connections
            .remove(&id)
            .ok_or_else(|| NetError::websocket_not_found(format!("WebSocket {} not found", id)))?
            .stream
    };

    // Close the WebSocket
    let mut stream_guard = stream.lock().await;
    stream_guard
        .close(None)
        .await
        .map_err(|e| NetError::websocket_close(e.to_string()))?;

    debug!(id = id, "ws.close complete");

    Ok(())
}

// ============================================================================
// Streaming Fetch Operations
// ============================================================================

/// Streaming fetch response info
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct StreamResponse {
    pub id: u64,
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub url: String,
    pub ok: bool,
}

/// Start a streaming fetch request
#[weld_op(async)]
#[op2(async)]
#[serde]
async fn op_net_fetch_stream(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<FetchOpts>,
) -> Result<StreamResponse, NetError> {
    let opts = opts.unwrap_or_default();

    // Extract host and check capabilities
    let host = extract_host(&url)?;
    {
        let s = state.borrow();
        check_net_connect(&s, &host)?;
    }

    debug!(url = %url, "net.fetch_stream");

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

    // Extract response info
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

    // Get or create stream state
    let stream_state = {
        let mut s = state.borrow_mut();
        if s.try_borrow::<StreamState>().is_none() {
            s.put(StreamState::default());
        }
        s.borrow::<StreamState>().clone()
    };

    // Generate stream ID
    let id = {
        let mut next_id = stream_state.next_id.lock().await;
        let stream_id = *next_id;
        *next_id += 1;
        stream_id
    };

    // Store the response for streaming
    {
        let mut streams = stream_state.streams.lock().await;
        streams.insert(id, StreamEntry { response });
    }

    debug!(id = id, status = status, "net.fetch_stream started");

    Ok(StreamResponse {
        id,
        status,
        status_text,
        headers,
        url: final_url,
        ok,
    })
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize net state in OpState
pub fn init_net_state(op_state: &mut OpState, capabilities: Option<Arc<dyn NetCapabilityChecker>>) {
    // Initialize HTTP client
    op_state.put(NetHttpClient::default());

    // Initialize WebSocket state
    op_state.put(WebSocketState::default());

    // Initialize stream state
    op_state.put(StreamState::default());

    // Set capabilities
    if let Some(caps) = capabilities {
        op_state.put(NetCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs (contains transpiled TypeScript)
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn net_extension() -> Extension {
    runtime_net::ext()
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
            extract_host("http://localruntime:3000").unwrap(),
            "localruntime:3000"
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
