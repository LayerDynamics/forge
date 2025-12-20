//! runtime:protocol extension - Custom URL protocol handler registration
//!
//! Provides cross-platform custom URL protocol handling:
//! - Register custom URL schemes (e.g., myapp://)
//! - Handle protocol invocations when URLs are opened
//! - Query registration status
//! - Support for deep linking and app activation
//!
//! Platform implementations:
//! - macOS: CFBundleURLTypes in Info.plist, Launch Services
//! - Windows: Registry HKEY_CLASSES_ROOT entries
//! - Linux: .desktop files with xdg-mime

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

// Platform-specific implementations
#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "macos")]
mod os_mac;
#[cfg(target_os = "windows")]
mod os_windows;

// Re-export platform implementation
#[cfg(target_os = "linux")]
use os_linux as platform;
#[cfg(target_os = "macos")]
use os_mac as platform;
#[cfg(target_os = "windows")]
use os_windows as platform;

// Include generated extension code from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

// ============================================================================
// Error Types with Structured Codes (9400-9499)
// ============================================================================

/// Error codes for protocol operations (9400-9499)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProtocolErrorCode {
    /// Generic protocol error
    Generic = 9400,
    /// Registration failed
    RegistrationFailed = 9401,
    /// Unregistration failed
    UnregistrationFailed = 9402,
    /// Already registered by another application
    AlreadyRegistered = 9403,
    /// Not registered
    NotRegistered = 9404,
    /// Invalid scheme format
    InvalidScheme = 9405,
    /// Invalid URL format
    InvalidUrl = 9406,
    /// Platform unsupported for this operation
    PlatformUnsupported = 9407,
    /// Permission denied
    PermissionDenied = 9408,
    /// System tool not found (xdg-mime, reg.exe)
    ToolNotFound = 9409,
    /// Desktop file error (Linux)
    DesktopFileError = 9410,
    /// Registry access error (Windows)
    RegistryError = 9411,
    /// Info.plist error (macOS)
    InfoPlistError = 9412,
    /// Invocation dispatch failed
    InvocationFailed = 9413,
    /// State not initialized
    NotInitialized = 9414,
}

impl std::fmt::Display for ProtocolErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}

/// Custom error type for protocol operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum ProtocolError {
    #[error("[{code}] Protocol error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Registration failed: {message}")]
    #[class(generic)]
    RegistrationFailed { code: u32, message: String },

    #[error("[{code}] Unregistration failed: {message}")]
    #[class(generic)]
    UnregistrationFailed { code: u32, message: String },

    #[error("[{code}] Already registered: {message}")]
    #[class(generic)]
    AlreadyRegistered { code: u32, message: String },

    #[error("[{code}] Not registered: {message}")]
    #[class(generic)]
    NotRegistered { code: u32, message: String },

    #[error("[{code}] Invalid scheme: {message}")]
    #[class(generic)]
    InvalidScheme { code: u32, message: String },

    #[error("[{code}] Invalid URL: {message}")]
    #[class(generic)]
    InvalidUrl { code: u32, message: String },

    #[error("[{code}] Platform unsupported: {message}")]
    #[class(generic)]
    PlatformUnsupported { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Tool not found: {message}")]
    #[class(generic)]
    ToolNotFound { code: u32, message: String },

    #[error("[{code}] Desktop file error: {message}")]
    #[class(generic)]
    DesktopFileError { code: u32, message: String },

    #[error("[{code}] Registry error: {message}")]
    #[class(generic)]
    RegistryError { code: u32, message: String },

    #[error("[{code}] Info.plist error: {message}")]
    #[class(generic)]
    InfoPlistError { code: u32, message: String },

    #[error("[{code}] Invocation failed: {message}")]
    #[class(generic)]
    InvocationFailed { code: u32, message: String },

    #[error("[{code}] Not initialized: {message}")]
    #[class(generic)]
    NotInitialized { code: u32, message: String },
}

impl ProtocolError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: ProtocolErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn registration_failed(message: impl Into<String>) -> Self {
        Self::RegistrationFailed {
            code: ProtocolErrorCode::RegistrationFailed as u32,
            message: message.into(),
        }
    }

    pub fn unregistration_failed(message: impl Into<String>) -> Self {
        Self::UnregistrationFailed {
            code: ProtocolErrorCode::UnregistrationFailed as u32,
            message: message.into(),
        }
    }

    pub fn already_registered(message: impl Into<String>) -> Self {
        Self::AlreadyRegistered {
            code: ProtocolErrorCode::AlreadyRegistered as u32,
            message: message.into(),
        }
    }

    pub fn not_registered(message: impl Into<String>) -> Self {
        Self::NotRegistered {
            code: ProtocolErrorCode::NotRegistered as u32,
            message: message.into(),
        }
    }

    pub fn invalid_scheme(message: impl Into<String>) -> Self {
        Self::InvalidScheme {
            code: ProtocolErrorCode::InvalidScheme as u32,
            message: message.into(),
        }
    }

    pub fn invalid_url(message: impl Into<String>) -> Self {
        Self::InvalidUrl {
            code: ProtocolErrorCode::InvalidUrl as u32,
            message: message.into(),
        }
    }

    pub fn platform_unsupported(message: impl Into<String>) -> Self {
        Self::PlatformUnsupported {
            code: ProtocolErrorCode::PlatformUnsupported as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: ProtocolErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn tool_not_found(message: impl Into<String>) -> Self {
        Self::ToolNotFound {
            code: ProtocolErrorCode::ToolNotFound as u32,
            message: message.into(),
        }
    }

    pub fn desktop_file_error(message: impl Into<String>) -> Self {
        Self::DesktopFileError {
            code: ProtocolErrorCode::DesktopFileError as u32,
            message: message.into(),
        }
    }

    pub fn registry_error(message: impl Into<String>) -> Self {
        Self::RegistryError {
            code: ProtocolErrorCode::RegistryError as u32,
            message: message.into(),
        }
    }

    pub fn info_plist_error(message: impl Into<String>) -> Self {
        Self::InfoPlistError {
            code: ProtocolErrorCode::InfoPlistError as u32,
            message: message.into(),
        }
    }

    pub fn invocation_failed(message: impl Into<String>) -> Self {
        Self::InvocationFailed {
            code: ProtocolErrorCode::InvocationFailed as u32,
            message: message.into(),
        }
    }

    pub fn not_initialized(message: impl Into<String>) -> Self {
        Self::NotInitialized {
            code: ProtocolErrorCode::NotInitialized as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Extension info
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub status: &'static str,
}

/// Options for registering a protocol handler
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationOptions {
    /// Human-readable description of what this protocol does
    pub description: Option<String>,
    /// Path to an icon file for the protocol
    pub icon_path: Option<String>,
    /// Whether to set this app as the default handler (default: true)
    pub set_as_default: Option<bool>,
}

impl Default for RegistrationOptions {
    fn default() -> Self {
        Self {
            description: None,
            icon_path: None,
            set_as_default: Some(true),
        }
    }
}

/// Result of a protocol registration attempt
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// Whether registration succeeded
    pub success: bool,
    /// The scheme that was registered
    pub scheme: String,
    /// Whether the scheme was already registered (by this or another app)
    pub was_already_registered: bool,
    /// The previous handler app identifier, if known
    pub previous_handler: Option<String>,
}

/// Status of a protocol registration
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationStatus {
    /// Whether this scheme is registered on the system
    pub is_registered: bool,
    /// Whether this app is the default handler
    pub is_default: bool,
    /// The app identifier that handles this scheme, if known
    pub registered_by: Option<String>,
}

/// Information about a registered protocol
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    /// The URL scheme (e.g., "myapp")
    pub scheme: String,
    /// Description of the protocol
    pub description: Option<String>,
    /// Path to the protocol icon
    pub icon_path: Option<String>,
    /// Whether this app is the default handler
    pub is_default: bool,
    /// The app that registered this protocol
    pub registered_by: Option<String>,
}

/// A protocol invocation event when a URL is opened
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInvocation {
    /// Unique identifier for this invocation
    pub id: String,
    /// The full URL that was opened
    pub url: String,
    /// The scheme part (e.g., "myapp")
    pub scheme: String,
    /// The path component after the scheme
    pub path: String,
    /// Query parameters as key-value pairs
    pub query: HashMap<String, String>,
    /// The URL fragment (after #)
    pub fragment: Option<String>,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Whether this invocation launched the app (vs app already running)
    pub is_launch: bool,
}

/// Result of parsing a protocol URL
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedProtocolUrl {
    /// The scheme part
    pub scheme: String,
    /// The path component
    pub path: String,
    /// The host component if present
    pub host: Option<String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// The fragment
    pub fragment: Option<String>,
    /// Whether the URL is valid
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Platform capabilities for protocol handling
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    /// Whether protocol registration is supported
    pub can_register: bool,
    /// Whether the app can query registered handlers
    pub can_query: bool,
    /// Whether deep linking is supported
    pub can_deep_link: bool,
    /// Current platform
    pub platform: String,
    /// Platform-specific notes
    pub notes: Option<String>,
}

// ============================================================================
// State Management
// ============================================================================

/// State for the protocol extension
pub struct ProtocolState {
    /// Schemes registered by this app instance
    pub registered_schemes: HashSet<String>,
    /// URL that launched the app (if applicable)
    pub launch_url: Option<String>,
    /// Channel for broadcasting protocol invocations
    pub invocation_tx: broadcast::Sender<ProtocolInvocation>,
    /// Counter for generating invocation IDs
    pub next_invocation_id: u64,
    /// App identifier (bundle ID on macOS, exe path on Windows)
    pub app_identifier: String,
    /// App name for display
    pub app_name: String,
    /// Path to the app executable
    pub exe_path: String,
}

impl ProtocolState {
    pub fn new(app_identifier: String, app_name: String, exe_path: String) -> Self {
        let (invocation_tx, _) = broadcast::channel(64);
        Self {
            registered_schemes: HashSet::new(),
            launch_url: None,
            invocation_tx,
            next_invocation_id: 1,
            app_identifier,
            app_name,
            exe_path,
        }
    }

    pub fn generate_invocation_id(&mut self) -> String {
        let id = format!("inv-{}", self.next_invocation_id);
        self.next_invocation_id += 1;
        id
    }
}

/// Capability checker for protocol operations
pub trait ProtocolCapabilityChecker: Send + Sync + 'static {
    /// Check if protocol registration is allowed
    fn check_register(&self, scheme: &str) -> Result<(), String>;
    /// Check if unregistration is allowed
    fn check_unregister(&self, scheme: &str) -> Result<(), String>;
}

/// Default capability checker that allows all operations
pub struct DefaultProtocolCapabilityChecker;

impl ProtocolCapabilityChecker for DefaultProtocolCapabilityChecker {
    fn check_register(&self, _scheme: &str) -> Result<(), String> {
        Ok(())
    }

    fn check_unregister(&self, _scheme: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store capability checker in OpState
pub struct ProtocolCapabilities_ {
    pub checker: Arc<dyn ProtocolCapabilityChecker>,
}

impl Default for ProtocolCapabilities_ {
    fn default() -> Self {
        Self {
            checker: Arc::new(DefaultProtocolCapabilityChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Validate a URL scheme
fn validate_scheme(scheme: &str) -> Result<(), ProtocolError> {
    if scheme.is_empty() {
        return Err(ProtocolError::invalid_scheme("Scheme cannot be empty"));
    }

    // Scheme must start with a letter
    if !scheme
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        return Err(ProtocolError::invalid_scheme(
            "Scheme must start with a letter",
        ));
    }

    // Scheme can only contain letters, digits, +, -, .
    for c in scheme.chars() {
        if !c.is_ascii_alphanumeric() && c != '+' && c != '-' && c != '.' {
            return Err(ProtocolError::invalid_scheme(format!(
                "Invalid character in scheme: '{}'",
                c
            )));
        }
    }

    // Reject reserved schemes
    let reserved = [
        "http",
        "https",
        "file",
        "ftp",
        "mailto",
        "tel",
        "data",
        "javascript",
    ];
    if reserved.contains(&scheme.to_lowercase().as_str()) {
        return Err(ProtocolError::invalid_scheme(format!(
            "Scheme '{}' is reserved",
            scheme
        )));
    }

    Ok(())
}

/// Parse a protocol URL into components
fn parse_protocol_url(url: &str) -> ParsedProtocolUrl {
    match url::Url::parse(url) {
        Ok(parsed) => {
            let query: HashMap<String, String> = parsed
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            ParsedProtocolUrl {
                scheme: parsed.scheme().to_string(),
                path: parsed.path().to_string(),
                host: parsed.host_str().map(|s| s.to_string()),
                query,
                fragment: parsed.fragment().map(|s| s.to_string()),
                is_valid: true,
                error: None,
            }
        }
        Err(e) => ParsedProtocolUrl {
            scheme: String::new(),
            path: String::new(),
            host: None,
            query: HashMap::new(),
            fragment: None,
            is_valid: false,
            error: Some(e.to_string()),
        },
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Get extension info
#[weld_op]
#[op2]
#[serde]
pub fn op_protocol_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_protocol",
        version: env!("CARGO_PKG_VERSION"),
        status: "active",
    }
}

/// Register a custom URL protocol handler
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_protocol_register(
    state: Rc<RefCell<OpState>>,
    #[string] scheme: String,
    #[serde] options: Option<RegistrationOptions>,
) -> Result<RegistrationResult, ProtocolError> {
    debug!(scheme = %scheme, "protocol.register");

    // Validate scheme
    validate_scheme(&scheme)?;

    let options = options.unwrap_or_default();

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<ProtocolCapabilities_>() {
            caps.checker
                .check_register(&scheme)
                .map_err(ProtocolError::permission_denied)?;
        }
    }

    // Get app info from state
    let (app_id, app_name, exe_path) = {
        let state_ref = state.borrow();
        let proto_state = state_ref
            .try_borrow::<ProtocolState>()
            .ok_or_else(|| ProtocolError::not_initialized("Protocol state not initialized"))?;
        (
            proto_state.app_identifier.clone(),
            proto_state.app_name.clone(),
            proto_state.exe_path.clone(),
        )
    };

    // Call platform-specific registration
    let result =
        platform::register_protocol(&scheme, &app_id, &app_name, &exe_path, &options).await?;

    // Update state
    if result.success {
        let mut state_ref = state.borrow_mut();
        if let Some(proto_state) = state_ref.try_borrow_mut::<ProtocolState>() {
            proto_state.registered_schemes.insert(scheme);
        }
    }

    Ok(result)
}

/// Unregister a custom URL protocol handler
#[weld_op(async)]
#[op2(async)]
pub async fn op_protocol_unregister(
    state: Rc<RefCell<OpState>>,
    #[string] scheme: String,
) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "protocol.unregister");

    // Validate scheme
    validate_scheme(&scheme)?;

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<ProtocolCapabilities_>() {
            caps.checker
                .check_unregister(&scheme)
                .map_err(ProtocolError::permission_denied)?;
        }
    }

    // Call platform-specific unregistration
    let success = platform::unregister_protocol(&scheme).await?;

    // Update state
    if success {
        let mut state_ref = state.borrow_mut();
        if let Some(proto_state) = state_ref.try_borrow_mut::<ProtocolState>() {
            proto_state.registered_schemes.remove(&scheme);
        }
    }

    Ok(success)
}

/// Check if a protocol scheme is registered
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_protocol_is_registered(
    #[string] scheme: String,
) -> Result<RegistrationStatus, ProtocolError> {
    debug!(scheme = %scheme, "protocol.is_registered");

    // Validate scheme
    validate_scheme(&scheme)?;

    platform::is_registered(&scheme).await
}

/// List all protocols registered by this app
#[weld_op]
#[op2]
#[serde]
pub fn op_protocol_list_registered(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<ProtocolInfo>, ProtocolError> {
    debug!("protocol.list_registered");

    let state_ref = state.borrow();
    let proto_state = state_ref
        .try_borrow::<ProtocolState>()
        .ok_or_else(|| ProtocolError::not_initialized("Protocol state not initialized"))?;

    let protocols: Vec<ProtocolInfo> = proto_state
        .registered_schemes
        .iter()
        .map(|scheme| ProtocolInfo {
            scheme: scheme.clone(),
            description: None,
            icon_path: None,
            is_default: true, // We registered it, so we're the default
            registered_by: Some(proto_state.app_identifier.clone()),
        })
        .collect();

    Ok(protocols)
}

/// Set this app as the default handler for a scheme
#[weld_op(async)]
#[op2(async)]
pub async fn op_protocol_set_as_default(
    state: Rc<RefCell<OpState>>,
    #[string] scheme: String,
) -> Result<bool, ProtocolError> {
    debug!(scheme = %scheme, "protocol.set_as_default");

    // Validate scheme
    validate_scheme(&scheme)?;

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<ProtocolCapabilities_>() {
            caps.checker
                .check_register(&scheme)
                .map_err(ProtocolError::permission_denied)?;
        }
    }

    platform::set_as_default(&scheme).await
}

/// Get the URL that launched the app (if applicable)
#[weld_op]
#[op2]
#[string]
pub fn op_protocol_get_launch_url(
    state: Rc<RefCell<OpState>>,
) -> Result<Option<String>, ProtocolError> {
    debug!("protocol.get_launch_url");

    let state_ref = state.borrow();
    let proto_state = state_ref
        .try_borrow::<ProtocolState>()
        .ok_or_else(|| ProtocolError::not_initialized("Protocol state not initialized"))?;

    Ok(proto_state.launch_url.clone())
}

/// Parse a protocol URL into its components
#[weld_op]
#[op2]
#[serde]
pub fn op_protocol_parse_url(#[string] url: String) -> ParsedProtocolUrl {
    debug!(url = %url, "protocol.parse_url");
    parse_protocol_url(&url)
}

/// Build a protocol URL from components
#[weld_op]
#[op2]
#[string]
pub fn op_protocol_build_url(
    #[string] scheme: String,
    #[string] path: String,
    #[serde] query: Option<HashMap<String, String>>,
) -> Result<String, ProtocolError> {
    debug!(scheme = %scheme, path = %path, "protocol.build_url");

    validate_scheme(&scheme)?;

    let mut url = format!("{}://{}", scheme, path.trim_start_matches('/'));

    if let Some(query_params) = query {
        if !query_params.is_empty() {
            let query_string: Vec<String> = query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            url.push('?');
            url.push_str(&query_string.join("&"));
        }
    }

    Ok(url)
}

/// Check platform capabilities for protocol handling
#[weld_op]
#[op2]
#[serde]
pub fn op_protocol_check_capabilities() -> ProtocolCapabilities {
    debug!("protocol.check_capabilities");
    platform::check_capabilities()
}

/// Subscribe to protocol invocation events
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_protocol_receive_invocation(
    state: Rc<RefCell<OpState>>,
) -> Result<ProtocolInvocation, ProtocolError> {
    debug!("protocol.receive_invocation");

    let mut rx = {
        let state_ref = state.borrow();
        let proto_state = state_ref
            .try_borrow::<ProtocolState>()
            .ok_or_else(|| ProtocolError::not_initialized("Protocol state not initialized"))?;
        proto_state.invocation_tx.subscribe()
    };

    // Wait for next invocation
    rx.recv().await.map_err(|e| {
        ProtocolError::invocation_failed(format!("Failed to receive invocation: {}", e))
    })
}

/// Dispatch a protocol invocation (called by the runtime when a URL is opened)
pub fn dispatch_invocation(
    state: &mut OpState,
    url: &str,
    is_launch: bool,
) -> Result<(), ProtocolError> {
    let proto_state = state
        .try_borrow_mut::<ProtocolState>()
        .ok_or_else(|| ProtocolError::not_initialized("Protocol state not initialized"))?;

    let parsed = parse_protocol_url(url);
    if !parsed.is_valid {
        return Err(ProtocolError::invalid_url(format!(
            "Invalid URL: {}",
            parsed.error.unwrap_or_default()
        )));
    }

    let invocation = ProtocolInvocation {
        id: proto_state.generate_invocation_id(),
        url: url.to_string(),
        scheme: parsed.scheme,
        path: parsed.path,
        query: parsed.query,
        fragment: parsed.fragment,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        is_launch,
    };

    // Store as launch URL if this is the launch invocation
    if is_launch {
        proto_state.launch_url = Some(url.to_string());
    }

    // Broadcast to subscribers
    let _ = proto_state.invocation_tx.send(invocation);

    Ok(())
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize protocol state in OpState
pub fn init_protocol_state(
    op_state: &mut OpState,
    app_identifier: String,
    app_name: String,
    exe_path: String,
    launch_url: Option<String>,
    capabilities: Option<Arc<dyn ProtocolCapabilityChecker>>,
) {
    let mut state = ProtocolState::new(app_identifier, app_name, exe_path);
    state.launch_url = launch_url;
    op_state.put(state);

    if let Some(caps) = capabilities {
        op_state.put(ProtocolCapabilities_ { checker: caps });
    } else {
        op_state.put(ProtocolCapabilities_::default());
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

pub fn protocol_extension() -> Extension {
    runtime_protocol::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_scheme_valid() {
        assert!(validate_scheme("myapp").is_ok());
        assert!(validate_scheme("my-app").is_ok());
        assert!(validate_scheme("my.app").is_ok());
        assert!(validate_scheme("my+app").is_ok());
        assert!(validate_scheme("MyApp123").is_ok());
    }

    #[test]
    fn test_validate_scheme_invalid() {
        assert!(validate_scheme("").is_err());
        assert!(validate_scheme("123app").is_err());
        assert!(validate_scheme("my app").is_err());
        assert!(validate_scheme("my_app").is_err());
        assert!(validate_scheme("http").is_err());
        assert!(validate_scheme("https").is_err());
        assert!(validate_scheme("file").is_err());
    }

    #[test]
    fn test_parse_protocol_url() {
        // URL format: scheme://host/path?query#fragment
        // In "myapp://action/path", "action" is the host, "/path" is the path
        let parsed = parse_protocol_url("myapp://action/path?foo=bar&baz=qux#section");
        assert!(parsed.is_valid);
        assert_eq!(parsed.scheme, "myapp");
        assert_eq!(parsed.host, Some("action".to_string()));
        assert_eq!(parsed.path, "/path");
        assert_eq!(parsed.query.get("foo"), Some(&"bar".to_string()));
        assert_eq!(parsed.query.get("baz"), Some(&"qux".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_protocol_url_simple() {
        // URL format: scheme://host - "open" is the host, path is empty
        let parsed = parse_protocol_url("myapp://open");
        assert!(parsed.is_valid);
        assert_eq!(parsed.scheme, "myapp");
        assert_eq!(parsed.host, Some("open".to_string()));
        assert_eq!(parsed.path, "");
    }

    #[test]
    fn test_parse_protocol_url_invalid() {
        let parsed = parse_protocol_url("not a valid url");
        assert!(!parsed.is_valid);
        assert!(parsed.error.is_some());
    }

    #[test]
    fn test_error_codes() {
        let err = ProtocolError::registration_failed("test");
        match err {
            ProtocolError::RegistrationFailed { code, .. } => {
                assert_eq!(code, ProtocolErrorCode::RegistrationFailed as u32);
                assert_eq!(code, 9401);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_default_capability_checker() {
        let checker = DefaultProtocolCapabilityChecker;
        assert!(checker.check_register("myapp").is_ok());
        assert!(checker.check_unregister("myapp").is_ok());
    }

    // =========================================================================
    // Data Structure Tests - Prove all types are properly defined
    // =========================================================================

    #[test]
    fn test_registration_options_default() {
        let opts = RegistrationOptions::default();
        assert!(opts.description.is_none());
        assert!(opts.icon_path.is_none());
        // Default implementation sets set_as_default to Some(true)
        assert_eq!(opts.set_as_default, Some(true));
    }

    #[test]
    fn test_registration_options_full() {
        let opts = RegistrationOptions {
            description: Some("My Protocol Handler".to_string()),
            icon_path: Some("/path/to/icon.png".to_string()),
            set_as_default: Some(true),
        };
        assert_eq!(opts.description.as_deref(), Some("My Protocol Handler"));
        assert_eq!(opts.icon_path.as_deref(), Some("/path/to/icon.png"));
        assert_eq!(opts.set_as_default, Some(true));
    }

    #[test]
    fn test_registration_result_serialization() {
        let result = RegistrationResult {
            success: true,
            scheme: "myapp".to_string(),
            was_already_registered: false,
            previous_handler: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"scheme\":\"myapp\""));

        // Deserialize back
        let parsed: RegistrationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.success, result.success);
        assert_eq!(parsed.scheme, result.scheme);
    }

    #[test]
    fn test_registration_status_serialization() {
        let status = RegistrationStatus {
            is_registered: true,
            is_default: true,
            registered_by: Some("com.myapp.handler".to_string()),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: RegistrationStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.is_registered, true);
        assert_eq!(parsed.is_default, true);
        assert_eq!(parsed.registered_by, Some("com.myapp.handler".to_string()));
    }

    #[test]
    fn test_protocol_info_serialization() {
        let info = ProtocolInfo {
            scheme: "myapp".to_string(),
            description: Some("My Application".to_string()),
            icon_path: None,
            is_default: true,
            registered_by: Some("com.myapp".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"scheme\":\"myapp\""));
        assert!(json.contains("\"is_default\":true"));

        let parsed: ProtocolInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.scheme, "myapp");
        assert_eq!(parsed.description, Some("My Application".to_string()));
    }

    #[test]
    fn test_protocol_invocation_serialization() {
        let invocation = ProtocolInvocation {
            id: "inv-123".to_string(),
            url: "myapp://open/document?id=42".to_string(),
            scheme: "myapp".to_string(),
            path: "/open/document".to_string(),
            query: {
                let mut q = std::collections::HashMap::new();
                q.insert("id".to_string(), "42".to_string());
                q
            },
            fragment: None,
            timestamp: 1700000000000,
            is_launch: true,
        };

        let json = serde_json::to_string(&invocation).unwrap();
        assert!(json.contains("\"id\":\"inv-123\""));
        assert!(json.contains("\"is_launch\":true"));

        let parsed: ProtocolInvocation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "inv-123");
        assert_eq!(parsed.query.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_parsed_protocol_url_serialization() {
        let parsed = ParsedProtocolUrl {
            scheme: "myapp".to_string(),
            path: "/action".to_string(),
            host: None,
            query: std::collections::HashMap::new(),
            fragment: Some("section".to_string()),
            is_valid: true,
            error: None,
        };

        let json = serde_json::to_string(&parsed).unwrap();
        assert!(json.contains("\"is_valid\":true"));

        let roundtrip: ParsedProtocolUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.scheme, "myapp");
        assert_eq!(roundtrip.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_protocol_capabilities_serialization() {
        let caps = ProtocolCapabilities {
            can_register: true,
            can_query: true,
            can_deep_link: true,
            platform: "macos".to_string(),
            notes: Some("Full support".to_string()),
        };

        let json = serde_json::to_string(&caps).unwrap();
        let parsed: ProtocolCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.can_register, true);
        assert_eq!(parsed.platform, "macos");
    }

    // =========================================================================
    // URL Parsing Edge Cases
    // =========================================================================

    #[test]
    fn test_parse_url_with_empty_query() {
        let parsed = parse_protocol_url("myapp://path?");
        assert!(parsed.is_valid);
        assert_eq!(parsed.scheme, "myapp");
        assert!(parsed.query.is_empty());
    }

    #[test]
    fn test_parse_url_with_fragment_only() {
        let parsed = parse_protocol_url("myapp://path#anchor");
        assert!(parsed.is_valid);
        assert_eq!(parsed.fragment, Some("anchor".to_string()));
    }

    #[test]
    fn test_parse_url_with_encoded_query() {
        let parsed = parse_protocol_url("myapp://path?name=hello%20world");
        assert!(parsed.is_valid);
        // URL-decoded value
        assert!(parsed.query.contains_key("name"));
    }

    #[test]
    fn test_parse_url_multiple_query_params() {
        let parsed = parse_protocol_url("myapp://open?a=1&b=2&c=3");
        assert!(parsed.is_valid);
        assert_eq!(parsed.query.len(), 3);
        assert_eq!(parsed.query.get("a"), Some(&"1".to_string()));
        assert_eq!(parsed.query.get("b"), Some(&"2".to_string()));
        assert_eq!(parsed.query.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn test_parse_url_host_in_path() {
        // myapp://host/path - host becomes part of path
        let parsed = parse_protocol_url("myapp://localhost/api/endpoint");
        assert!(parsed.is_valid);
        assert_eq!(parsed.host, Some("localhost".to_string()));
    }

    // =========================================================================
    // Scheme Validation Edge Cases
    // =========================================================================

    #[test]
    fn test_validate_scheme_reserved_protocols() {
        // All reserved schemes should be rejected
        let reserved = [
            "http",
            "https",
            "file",
            "ftp",
            "mailto",
            "tel",
            "data",
            "javascript",
        ];
        for scheme in reserved {
            let result = validate_scheme(scheme);
            assert!(result.is_err(), "Scheme '{}' should be rejected", scheme);
        }
    }

    #[test]
    fn test_validate_scheme_case_insensitive() {
        // HTTP in any case should be rejected
        assert!(validate_scheme("HTTP").is_err());
        assert!(validate_scheme("Http").is_err());
        assert!(validate_scheme("HTTPS").is_err());
    }

    #[test]
    fn test_validate_scheme_with_numbers() {
        // Numbers are allowed but not at start
        assert!(validate_scheme("app123").is_ok());
        assert!(validate_scheme("my2app").is_ok());
        assert!(validate_scheme("123app").is_err());
    }

    #[test]
    fn test_validate_scheme_length() {
        // Very long scheme should still work
        assert!(validate_scheme("myverylongapplicationscheme").is_ok());
        // Single character is valid per RFC 3986
        assert!(validate_scheme("x").is_ok());
    }

    // =========================================================================
    // Error Type Tests
    // =========================================================================

    #[test]
    fn test_all_error_codes_unique() {
        let codes = [
            ProtocolErrorCode::Generic as u32,
            ProtocolErrorCode::RegistrationFailed as u32,
            ProtocolErrorCode::UnregistrationFailed as u32,
            ProtocolErrorCode::AlreadyRegistered as u32,
            ProtocolErrorCode::NotRegistered as u32,
            ProtocolErrorCode::InvalidScheme as u32,
            ProtocolErrorCode::InvalidUrl as u32,
            ProtocolErrorCode::PermissionDenied as u32,
            ProtocolErrorCode::PlatformUnsupported as u32,
            ProtocolErrorCode::InvocationFailed as u32,
            ProtocolErrorCode::ToolNotFound as u32,
            ProtocolErrorCode::DesktopFileError as u32,
            ProtocolErrorCode::RegistryError as u32,
            ProtocolErrorCode::InfoPlistError as u32,
            ProtocolErrorCode::NotInitialized as u32,
        ];

        // Check all codes are in the 9400-9499 range
        for code in codes {
            assert!(code >= 9400 && code < 9500, "Code {} out of range", code);
        }

        // Check all codes are unique
        let mut seen = std::collections::HashSet::new();
        for code in codes {
            assert!(seen.insert(code), "Duplicate error code: {}", code);
        }
    }

    #[test]
    fn test_error_constructors() {
        let e1 = ProtocolError::invalid_scheme("scheme contains invalid chars");
        assert!(format!("{}", e1).contains("invalid"));

        let e2 = ProtocolError::platform_unsupported("registration not available");
        assert!(format!("{}", e2).contains("unsupported"));

        let e3 = ProtocolError::permission_denied("admin required");
        assert!(format!("{}", e3).contains("denied"));

        let e4 = ProtocolError::registration_failed("could not register");
        assert!(format!("{}", e4).contains("Registration"));

        let e5 = ProtocolError::not_registered("scheme not found");
        assert!(format!("{}", e5).contains("registered"));
    }

    // =========================================================================
    // State Management Tests
    // =========================================================================

    #[test]
    fn test_protocol_state_creation() {
        let state = ProtocolState::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
            "/usr/bin/testapp".to_string(),
        );
        assert!(state.registered_schemes.is_empty());
        assert!(state.launch_url.is_none());
        assert_eq!(state.app_identifier, "com.test.app");
        assert_eq!(state.app_name, "Test App");
        assert_eq!(state.exe_path, "/usr/bin/testapp");
    }

    #[test]
    fn test_protocol_state_invocation_id_generation() {
        let mut state = ProtocolState::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
            "/usr/bin/testapp".to_string(),
        );

        let id1 = state.generate_invocation_id();
        let id2 = state.generate_invocation_id();
        let id3 = state.generate_invocation_id();

        assert_eq!(id1, "inv-1");
        assert_eq!(id2, "inv-2");
        assert_eq!(id3, "inv-3");
    }

    #[test]
    fn test_protocol_state_broadcast_channel() {
        let state = ProtocolState::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
            "/usr/bin/testapp".to_string(),
        );

        // Subscribe before sending
        let mut rx = state.invocation_tx.subscribe();

        // Send an invocation
        let invocation = ProtocolInvocation {
            id: "test-1".to_string(),
            url: "myapp://test".to_string(),
            scheme: "myapp".to_string(),
            path: "/test".to_string(),
            query: std::collections::HashMap::new(),
            fragment: None,
            timestamp: 0,
            is_launch: false,
        };

        let _ = state.invocation_tx.send(invocation.clone());

        // Should receive it
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().id, "test-1");
    }

    // =========================================================================
    // Extension Info Test
    // =========================================================================

    #[test]
    fn test_extension_info_struct() {
        let info = ExtensionInfo {
            name: "ext_protocol",
            version: "0.1.0",
            status: "active",
        };
        assert_eq!(info.name, "ext_protocol");
        assert!(!info.version.is_empty());
        assert_eq!(info.status, "active");
    }

    // =========================================================================
    // Capability Checker Tests
    // =========================================================================

    #[test]
    fn test_default_capability_checker_impl() {
        let checker = DefaultProtocolCapabilityChecker;
        assert!(checker.check_register("myapp").is_ok());
        assert!(checker.check_register("another-scheme").is_ok());
        assert!(checker.check_unregister("myapp").is_ok());
    }

    #[test]
    fn test_protocol_capabilities_wrapper_default() {
        let caps = ProtocolCapabilities_::default();
        // Should use DefaultProtocolCapabilityChecker
        assert!(caps.checker.check_register("test").is_ok());
    }
}
