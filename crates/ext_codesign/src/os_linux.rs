//! Linux stub implementation for code signing
//!
//! Linux does not have a standard code signing mechanism like macOS or Windows.
//! This module provides stub implementations that return appropriate errors or
//! no-op results.

use crate::{CodesignCapabilities, CodesignError, SignOptions, SigningIdentity, VerifyResult};

/// Sign a file - not supported on Linux
pub async fn sign(_options: &SignOptions) -> Result<(), CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Code signing is not supported on Linux. Consider using GPG for signature verification.",
    ))
}

/// Ad-hoc signing - not supported on Linux
pub async fn sign_adhoc(_path: &str) -> Result<(), CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Ad-hoc signing is only supported on macOS",
    ))
}

/// Verify a signature - returns success since Linux doesn't verify
pub async fn verify(path: &str) -> Result<VerifyResult, CodesignError> {
    Ok(VerifyResult {
        valid: true,
        signer: None,
        timestamp: None,
        message: format!(
            "Code signature verification not available on Linux. Path: {}",
            path
        ),
    })
}

/// Get entitlements - not supported on Linux
pub async fn get_entitlements(_path: &str) -> Result<String, CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Entitlements are a macOS-only concept",
    ))
}

/// List signing identities - returns empty list on Linux
pub async fn list_identities() -> Result<Vec<SigningIdentity>, CodesignError> {
    Ok(Vec::new())
}

/// Get identity info - not supported on Linux
pub async fn get_identity_info(_identity: &str) -> Result<SigningIdentity, CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Code signing identities are not available on Linux",
    ))
}

/// Check available capabilities on Linux
pub fn check_capabilities() -> CodesignCapabilities {
    CodesignCapabilities {
        codesign: false,
        security: false,
        signtool: false,
        certutil: false,
        platform: "linux".to_string(),
    }
}
