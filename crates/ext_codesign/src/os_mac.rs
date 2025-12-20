//! macOS implementation for code signing
//!
//! Uses the `codesign` and `security` CLI tools for signing operations.
//! - `codesign` - Sign and verify code signatures
//! - `security` - Manage keychains and list signing identities

use crate::{CodesignCapabilities, CodesignError, SignOptions, SigningIdentity, VerifyResult};
use tokio::process::Command;
use tracing::{debug, error, warn};

/// Sign a file or application bundle with a code signing identity
pub async fn sign(options: &SignOptions) -> Result<(), CodesignError> {
    let mut cmd = Command::new("codesign");

    // Basic signing arguments
    cmd.args(["--sign", &options.identity, "--force", "--timestamp"]);

    // Hardened runtime (default: true for modern macOS apps)
    if options.hardened_runtime.unwrap_or(true) {
        cmd.args(["--options", "runtime"]);
    }

    // Deep sign (sign embedded frameworks/bundles)
    if options.deep.unwrap_or(false) {
        cmd.arg("--deep");
    }

    // Entitlements file
    if let Some(ref entitlements) = options.entitlements {
        if !std::path::Path::new(entitlements).exists() {
            return Err(CodesignError::invalid_entitlements(format!(
                "Entitlements file does not exist: {}",
                entitlements
            )));
        }
        cmd.args(["--entitlements", entitlements]);
    }

    // Path to sign
    cmd.arg(&options.path);

    debug!(
        "Running codesign command: codesign --sign {} --force --timestamp {}",
        options.identity, options.path
    );

    let output = cmd.output().await.map_err(|e| {
        error!("Failed to execute codesign: {}", e);
        CodesignError::tool_not_found(format!("Failed to execute codesign: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("codesign failed: {}", stderr);
        return Err(CodesignError::signing_failed(stderr.to_string()));
    }

    debug!("Successfully signed: {}", options.path);
    Ok(())
}

/// Sign with an ad-hoc signature (identity = "-")
pub async fn sign_adhoc(path: &str) -> Result<(), CodesignError> {
    let output = Command::new("codesign")
        .args(["--sign", "-", "--force", path])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute codesign: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute codesign: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Ad-hoc signing failed: {}", stderr);
        return Err(CodesignError::signing_failed(stderr.to_string()));
    }

    debug!("Successfully ad-hoc signed: {}", path);
    Ok(())
}

/// Verify a code signature using `codesign --verify`
pub async fn verify(path: &str) -> Result<VerifyResult, CodesignError> {
    // First, verify the signature
    let verify_output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict", "--verbose=2", path])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute codesign: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute codesign: {}", e))
        })?;

    let valid = verify_output.status.success();
    let stderr = String::from_utf8_lossy(&verify_output.stderr);

    // Get signer info if valid
    let signer = if valid {
        get_signer_info(path).await.ok()
    } else {
        None
    };

    // Get timestamp if available
    let timestamp = if valid {
        get_timestamp_info(path).await.ok().flatten()
    } else {
        None
    };

    let message = if valid {
        format!("Valid signature on {}", path)
    } else {
        stderr.to_string()
    };

    Ok(VerifyResult {
        valid,
        signer,
        timestamp,
        message,
    })
}

/// Get signer information from a signed binary
async fn get_signer_info(path: &str) -> Result<String, CodesignError> {
    let output = Command::new("codesign")
        .args(["-d", "--verbose=2", path])
        .output()
        .await
        .map_err(|e| CodesignError::verification_failed(e.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse "Authority=" line from output
    for line in stderr.lines() {
        if line.starts_with("Authority=") {
            return Ok(line.trim_start_matches("Authority=").to_string());
        }
    }

    // Fallback: try to find TeamIdentifier
    for line in stderr.lines() {
        if line.starts_with("TeamIdentifier=") {
            return Ok(line.trim_start_matches("TeamIdentifier=").to_string());
        }
    }

    Err(CodesignError::verification_failed(
        "Could not extract signer info",
    ))
}

/// Get timestamp information from a signed binary
async fn get_timestamp_info(path: &str) -> Result<Option<String>, CodesignError> {
    let output = Command::new("codesign")
        .args(["-d", "--verbose=4", path])
        .output()
        .await
        .map_err(|e| CodesignError::verification_failed(e.to_string()))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse "Timestamp=" line from output
    for line in stderr.lines() {
        if line.starts_with("Timestamp=") {
            let timestamp = line.trim_start_matches("Timestamp=").to_string();
            if timestamp != "none" {
                return Ok(Some(timestamp));
            }
        }
    }

    Ok(None)
}

/// Get entitlements from a signed binary
pub async fn get_entitlements(path: &str) -> Result<String, CodesignError> {
    let output = Command::new("codesign")
        .args(["-d", "--entitlements", ":-", path])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute codesign: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute codesign: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if it's just "no entitlements" which is not an error
        if stderr.contains("no entitlements") || stderr.is_empty() {
            return Ok(String::new());
        }
        return Err(CodesignError::entitlements_failed(stderr.to_string()));
    }

    // Output is on stdout for entitlements
    let entitlements = String::from_utf8_lossy(&output.stdout).to_string();

    // If stderr has the entitlements (older behavior)
    if entitlements.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Ok(stderr);
    }

    Ok(entitlements)
}

/// List available signing identities using `security find-identity`
pub async fn list_identities() -> Result<Vec<SigningIdentity>, CodesignError> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute security: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute security: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("security find-identity returned error: {}", stderr);
        // Return empty list rather than error - keychain might be locked
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let identities = parse_security_output(&stdout);

    Ok(identities)
}

/// Parse output from `security find-identity -v -p codesigning`
/// Format: "  1) HASH "Name" (Status)"
fn parse_security_output(output: &str) -> Vec<SigningIdentity> {
    let mut identities = Vec::new();

    for line in output.lines() {
        let line = line.trim();

        // Skip empty lines and summary lines
        if line.is_empty() || line.contains("valid identities found") || line.contains("matching") {
            continue;
        }

        // Parse lines like: 1) ABC123DEF456... "Developer ID Application: Name (TEAMID)"
        if let Some(parsed) = parse_identity_line(line) {
            identities.push(parsed);
        }
    }

    identities
}

/// Parse a single identity line from security output
fn parse_identity_line(line: &str) -> Option<SigningIdentity> {
    // Expected format: "  1) ABC123... "Name" (optional status)"
    // Find the SHA-1 hash (40 hex characters)
    let parts: Vec<&str> = line.splitn(2, ')').collect();
    if parts.len() < 2 {
        return None;
    }

    let remainder = parts[1].trim();

    // Extract SHA-1 (first 40 characters should be hex)
    let hash_end = remainder.find(' ')?;
    let hash = &remainder[..hash_end];

    if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    // Extract name (quoted string)
    let name_start = remainder.find('"')? + 1;
    let name_end = remainder[name_start..].find('"')? + name_start;
    let name = remainder[name_start..name_end].to_string();

    // Determine identity type from name
    let identity_type = determine_identity_type(&name);

    // Check for CSSMERR (invalid status) - these are marked as invalid
    let valid = !line.contains("CSSMERR");

    Some(SigningIdentity {
        id: hash.to_string(),
        name,
        expires: None, // Would need additional cert inspection
        valid,
        identity_type,
    })
}

/// Determine the identity type from the certificate name
fn determine_identity_type(name: &str) -> String {
    if name.contains("Developer ID Application") {
        "developer_id_application".to_string()
    } else if name.contains("Developer ID Installer") {
        "developer_id_installer".to_string()
    } else if name.contains("Apple Distribution") {
        "distribution".to_string()
    } else if name.contains("Mac Developer") || name.contains("Apple Development") {
        "development".to_string()
    } else if name.contains("3rd Party Mac Developer") {
        "third_party".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get detailed information about a specific signing identity
pub async fn get_identity_info(identity: &str) -> Result<SigningIdentity, CodesignError> {
    // List all identities and find the matching one
    let identities = list_identities().await?;

    // Search by ID (SHA-1) or by name
    for id in identities {
        if id.id.eq_ignore_ascii_case(identity) || id.name.contains(identity) {
            return Ok(id);
        }
    }

    Err(CodesignError::identity_not_found(format!(
        "Identity not found: {}",
        identity
    )))
}

/// Check available capabilities on macOS
pub fn check_capabilities() -> CodesignCapabilities {
    let codesign = std::process::Command::new("codesign")
        .arg("--help")
        .output()
        .is_ok();

    let security = std::process::Command::new("security")
        .arg("help")
        .output()
        .is_ok();

    CodesignCapabilities {
        codesign,
        security,
        signtool: false,
        certutil: false,
        platform: "macos".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity_line() {
        let line = r#"  1) ABCDEF0123456789ABCDEF0123456789ABCDEF01 "Developer ID Application: Test Company (ABCD1234)""#;

        let identity = parse_identity_line(line).unwrap();

        assert_eq!(identity.id, "ABCDEF0123456789ABCDEF0123456789ABCDEF01");
        assert_eq!(
            identity.name,
            "Developer ID Application: Test Company (ABCD1234)"
        );
        assert_eq!(identity.identity_type, "developer_id_application");
        assert!(identity.valid);
    }

    #[test]
    fn test_determine_identity_type() {
        assert_eq!(
            determine_identity_type("Developer ID Application: Foo"),
            "developer_id_application"
        );
        assert_eq!(
            determine_identity_type("Apple Distribution: Foo"),
            "distribution"
        );
        assert_eq!(determine_identity_type("Mac Developer: Foo"), "development");
        assert_eq!(determine_identity_type("Some Random Cert"), "unknown");
    }

    #[test]
    fn test_parse_security_output() {
        let output = r#"
  1) ABCDEF0123456789ABCDEF0123456789ABCDEF01 "Developer ID Application: Test (ABC123)"
  2) 1234567890ABCDEF1234567890ABCDEF12345678 "Apple Development: Test (DEF456)"
     2 valid identities found
"#;

        let identities = parse_security_output(output);

        assert_eq!(identities.len(), 2);
        assert_eq!(identities[0].identity_type, "developer_id_application");
        assert_eq!(identities[1].identity_type, "development");
    }
}
