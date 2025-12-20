//! runtime:codesign extension - Code signing operations for Forge apps
//!
//! Provides cross-platform code signing capabilities:
//! - Sign binaries and app bundles with signing identities
//! - Ad-hoc signing (macOS only)
//! - Verify existing signatures
//! - List available signing identities
//! - Extract entitlements (macOS only)
//!
//! Uses system tools (codesign on macOS, signtool on Windows) for signing operations.

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
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
// Error Types with Structured Codes (8300-8319)
// ============================================================================

/// Error codes for codesign operations (8300-8319)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CodesignErrorCode {
    /// Generic codesign error
    Generic = 8300,
    /// Path not found
    PathNotFound = 8301,
    /// Identity not found
    IdentityNotFound = 8302,
    /// Signing failed
    SigningFailed = 8303,
    /// Verification failed
    VerificationFailed = 8304,
    /// Permission denied
    PermissionDenied = 8305,
    /// Platform not supported
    PlatformUnsupported = 8306,
    /// Invalid identity format
    InvalidIdentity = 8307,
    /// Entitlements extraction failed
    EntitlementsFailed = 8308,
    /// Tool not found (codesign/signtool)
    ToolNotFound = 8309,
    /// Invalid entitlements file
    InvalidEntitlements = 8310,
    /// Certificate expired
    CertificateExpired = 8311,
    /// Certificate not trusted
    CertificateNotTrusted = 8312,
}

impl std::fmt::Display for CodesignErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}

/// Custom error type for codesign operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum CodesignError {
    #[error("[{code}] Codesign error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Path not found: {message}")]
    #[class(generic)]
    PathNotFound { code: u32, message: String },

    #[error("[{code}] Identity not found: {message}")]
    #[class(generic)]
    IdentityNotFound { code: u32, message: String },

    #[error("[{code}] Signing failed: {message}")]
    #[class(generic)]
    SigningFailed { code: u32, message: String },

    #[error("[{code}] Verification failed: {message}")]
    #[class(generic)]
    VerificationFailed { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] Platform not supported: {message}")]
    #[class(generic)]
    PlatformUnsupported { code: u32, message: String },

    #[error("[{code}] Invalid identity: {message}")]
    #[class(generic)]
    InvalidIdentity { code: u32, message: String },

    #[error("[{code}] Entitlements failed: {message}")]
    #[class(generic)]
    EntitlementsFailed { code: u32, message: String },

    #[error("[{code}] Tool not found: {message}")]
    #[class(generic)]
    ToolNotFound { code: u32, message: String },

    #[error("[{code}] Invalid entitlements: {message}")]
    #[class(generic)]
    InvalidEntitlements { code: u32, message: String },

    #[error("[{code}] Certificate expired: {message}")]
    #[class(generic)]
    CertificateExpired { code: u32, message: String },

    #[error("[{code}] Certificate not trusted: {message}")]
    #[class(generic)]
    CertificateNotTrusted { code: u32, message: String },
}

impl CodesignError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: CodesignErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn path_not_found(message: impl Into<String>) -> Self {
        Self::PathNotFound {
            code: CodesignErrorCode::PathNotFound as u32,
            message: message.into(),
        }
    }

    pub fn identity_not_found(message: impl Into<String>) -> Self {
        Self::IdentityNotFound {
            code: CodesignErrorCode::IdentityNotFound as u32,
            message: message.into(),
        }
    }

    pub fn signing_failed(message: impl Into<String>) -> Self {
        Self::SigningFailed {
            code: CodesignErrorCode::SigningFailed as u32,
            message: message.into(),
        }
    }

    pub fn verification_failed(message: impl Into<String>) -> Self {
        Self::VerificationFailed {
            code: CodesignErrorCode::VerificationFailed as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: CodesignErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn platform_unsupported(message: impl Into<String>) -> Self {
        Self::PlatformUnsupported {
            code: CodesignErrorCode::PlatformUnsupported as u32,
            message: message.into(),
        }
    }

    pub fn invalid_identity(message: impl Into<String>) -> Self {
        Self::InvalidIdentity {
            code: CodesignErrorCode::InvalidIdentity as u32,
            message: message.into(),
        }
    }

    pub fn entitlements_failed(message: impl Into<String>) -> Self {
        Self::EntitlementsFailed {
            code: CodesignErrorCode::EntitlementsFailed as u32,
            message: message.into(),
        }
    }

    pub fn tool_not_found(message: impl Into<String>) -> Self {
        Self::ToolNotFound {
            code: CodesignErrorCode::ToolNotFound as u32,
            message: message.into(),
        }
    }

    pub fn invalid_entitlements(message: impl Into<String>) -> Self {
        Self::InvalidEntitlements {
            code: CodesignErrorCode::InvalidEntitlements as u32,
            message: message.into(),
        }
    }

    pub fn certificate_expired(message: impl Into<String>) -> Self {
        Self::CertificateExpired {
            code: CodesignErrorCode::CertificateExpired as u32,
            message: message.into(),
        }
    }

    pub fn certificate_not_trusted(message: impl Into<String>) -> Self {
        Self::CertificateNotTrusted {
            code: CodesignErrorCode::CertificateNotTrusted as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Options for code signing
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignOptions {
    /// Path to the file or bundle to sign
    pub path: String,
    /// Signing identity (certificate name or SHA-1 thumbprint)
    pub identity: String,
    /// Path to entitlements file (macOS only)
    pub entitlements: Option<String>,
    /// Enable hardened runtime (macOS, default: true)
    pub hardened_runtime: Option<bool>,
    /// Deep sign embedded code (macOS)
    pub deep: Option<bool>,
    /// Timestamp server URL (Windows, default: DigiCert)
    pub timestamp_url: Option<String>,
}

/// Information about a signing identity/certificate
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningIdentity {
    /// Certificate ID (SHA-1 thumbprint)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Expiration date (ISO 8601 format)
    pub expires: Option<String>,
    /// Whether the certificate is currently valid
    pub valid: bool,
    /// Type: "developer_id", "distribution", "development", "self_signed", "unknown"
    pub identity_type: String,
}

/// Result of signature verification
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    /// Whether the signature is valid
    pub valid: bool,
    /// Identity of the signer
    pub signer: Option<String>,
    /// Timestamp of signature (if timestamped)
    pub timestamp: Option<String>,
    /// Detailed status message
    pub message: String,
}

/// Available signing capabilities on current platform
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodesignCapabilities {
    /// macOS codesign tool available
    pub codesign: bool,
    /// macOS security tool available
    pub security: bool,
    /// Windows SignTool available
    pub signtool: bool,
    /// Windows certutil available
    pub certutil: bool,
    /// Current platform
    pub platform: String,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Trait for checking codesign permissions
pub trait CodesignCapabilityChecker: Send + Sync + 'static {
    /// Check if code signing is allowed
    fn check_sign(&self) -> Result<(), String>;

    /// Check if verification is allowed (generally always allowed - read-only)
    fn check_verify(&self) -> Result<(), String> {
        Ok(()) // Verification is always allowed by default
    }

    /// Check if listing identities is allowed
    fn check_list_identities(&self) -> Result<(), String>;
}

/// Default capability checker that allows all operations
pub struct DefaultCodesignCapabilityChecker;

impl CodesignCapabilityChecker for DefaultCodesignCapabilityChecker {
    fn check_sign(&self) -> Result<(), String> {
        Ok(())
    }

    fn check_list_identities(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store capability checker in OpState
pub struct CodesignState {
    pub checker: Arc<dyn CodesignCapabilityChecker>,
}

impl Default for CodesignState {
    fn default() -> Self {
        Self {
            checker: Arc::new(DefaultCodesignCapabilityChecker),
        }
    }
}

// ============================================================================
// Operations
// ============================================================================

/// Sign a file or application bundle with a code signing identity
#[weld_op(async)]
#[op2(async)]
pub async fn op_codesign_sign(
    state: Rc<RefCell<OpState>>,
    #[serde] options: SignOptions,
) -> Result<(), CodesignError> {
    debug!(path = %options.path, identity = %options.identity, "codesign.sign");

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_sign()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    // Validate path exists
    if !std::path::Path::new(&options.path).exists() {
        return Err(CodesignError::path_not_found(format!(
            "Path does not exist: {}",
            options.path
        )));
    }

    platform::sign(&options).await
}

/// Sign with an ad-hoc signature (macOS only, no identity required)
#[weld_op(async)]
#[op2(async)]
pub async fn op_codesign_sign_adhoc(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<(), CodesignError> {
    debug!(path = %path, "codesign.sign_adhoc");

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_sign()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    // Validate path exists
    if !std::path::Path::new(&path).exists() {
        return Err(CodesignError::path_not_found(format!(
            "Path does not exist: {}",
            path
        )));
    }

    platform::sign_adhoc(&path).await
}

/// Verify a code signature
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_codesign_verify(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<VerifyResult, CodesignError> {
    debug!(path = %path, "codesign.verify");

    // Check capability (verification is usually always allowed)
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_verify()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    // Validate path exists
    if !std::path::Path::new(&path).exists() {
        return Err(CodesignError::path_not_found(format!(
            "Path does not exist: {}",
            path
        )));
    }

    platform::verify(&path).await
}

/// Get entitlements from a signed binary (macOS only)
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_codesign_get_entitlements(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, CodesignError> {
    debug!(path = %path, "codesign.get_entitlements");

    // Check capability (read-only, like verify)
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_verify()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    // Validate path exists
    if !std::path::Path::new(&path).exists() {
        return Err(CodesignError::path_not_found(format!(
            "Path does not exist: {}",
            path
        )));
    }

    platform::get_entitlements(&path).await
}

/// List available signing identities/certificates
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_codesign_list_identities(
    state: Rc<RefCell<OpState>>,
) -> Result<Vec<SigningIdentity>, CodesignError> {
    debug!("codesign.list_identities");

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_list_identities()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    platform::list_identities().await
}

/// Get detailed information about a signing identity
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_codesign_get_identity_info(
    state: Rc<RefCell<OpState>>,
    #[string] identity: String,
) -> Result<SigningIdentity, CodesignError> {
    debug!(identity = %identity, "codesign.get_identity_info");

    // Check capability
    {
        let state_ref = state.borrow();
        if let Some(caps) = state_ref.try_borrow::<CodesignState>() {
            caps.checker
                .check_list_identities()
                .map_err(CodesignError::permission_denied)?;
        }
    }

    platform::get_identity_info(&identity).await
}

/// Check what signing capabilities are available on the current platform
#[weld_op]
#[op2]
#[serde]
pub fn op_codesign_check_capabilities() -> CodesignCapabilities {
    debug!("codesign.check_capabilities");
    platform::check_capabilities()
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize codesign state in OpState
pub fn init_codesign_state(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn CodesignCapabilityChecker>>,
) {
    if let Some(caps) = capabilities {
        op_state.put(CodesignState { checker: caps });
    } else {
        op_state.put(CodesignState::default());
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

pub fn codesign_extension() -> Extension {
    runtime_codesign::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = CodesignError::signing_failed("test");
        match err {
            CodesignError::SigningFailed { code, .. } => {
                assert_eq!(code, CodesignErrorCode::SigningFailed as u32);
                assert_eq!(code, 8303);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_default_capability_checker() {
        let checker = DefaultCodesignCapabilityChecker;
        assert!(checker.check_sign().is_ok());
        assert!(checker.check_verify().is_ok());
        assert!(checker.check_list_identities().is_ok());
    }

    #[test]
    fn test_sign_options_serialization() {
        let options = SignOptions {
            path: "/path/to/app".to_string(),
            identity: "Developer ID".to_string(),
            entitlements: Some("/path/to/entitlements.plist".to_string()),
            hardened_runtime: Some(true),
            deep: Some(true),
            timestamp_url: None,
        };

        let json = serde_json::to_string(&options).unwrap();
        let parsed: SignOptions = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.path, options.path);
        assert_eq!(parsed.identity, options.identity);
        assert_eq!(parsed.hardened_runtime, Some(true));
    }
}
