//! Windows implementation for code signing
//!
//! Uses `signtool.exe` from Windows SDK for signing operations
//! and PowerShell/certutil for certificate management.

use crate::{CodesignCapabilities, CodesignError, SignOptions, SigningIdentity, VerifyResult};
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, error, warn};

/// Default timestamp server URL
const DEFAULT_TIMESTAMP_URL: &str = "http://timestamp.digicert.com";

/// Common paths where signtool.exe might be found
const SIGNTOOL_PATHS: &[&str] = &[
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.18362.0\x64\signtool.exe",
    r"C:\Program Files (x86)\Windows Kits\10\bin\x64\signtool.exe",
    r"C:\Program Files (x86)\Microsoft SDKs\Windows\v10.0A\bin\NETFX 4.8 Tools\x64\signtool.exe",
];

/// Find signtool.exe on the system
fn find_signtool() -> Result<PathBuf, CodesignError> {
    // First check if it's in PATH
    if let Ok(output) = std::process::Command::new("where")
        .arg("signtool.exe")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.lines().next().unwrap_or("").trim();
            if !path.is_empty() && std::path::Path::new(path).exists() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    // Check common installation paths
    for path in SIGNTOOL_PATHS {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }

    // Try to find in Windows Kits directory
    let kits_dir = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if kits_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&kits_dir) {
            let mut versions: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();

            // Sort to get latest version first
            versions.sort();
            versions.reverse();

            for version_dir in versions {
                let signtool = version_dir.join("x64").join("signtool.exe");
                if signtool.exists() {
                    return Ok(signtool);
                }
            }
        }
    }

    Err(CodesignError::tool_not_found(
        "signtool.exe not found. Please install Windows SDK.",
    ))
}

/// Sign a file using signtool.exe
pub async fn sign(options: &SignOptions) -> Result<(), CodesignError> {
    let signtool = find_signtool()?;

    let mut cmd = Command::new(&signtool);
    cmd.args(["sign", "/fd", "SHA256"]);

    // Determine if identity is SHA-1 thumbprint or certificate file path
    if is_sha1_thumbprint(&options.identity) {
        // SHA-1 thumbprint (40 hex characters)
        cmd.args(["/sha1", &options.identity]);
    } else if std::path::Path::new(&options.identity).exists() {
        // Certificate file path
        cmd.args(["/f", &options.identity]);
    } else {
        // Try as subject name
        cmd.args(["/n", &options.identity]);
    }

    // Timestamp server
    let timestamp_url = options
        .timestamp_url
        .as_deref()
        .unwrap_or(DEFAULT_TIMESTAMP_URL);
    cmd.args(["/tr", timestamp_url, "/td", "SHA256"]);

    // Path to sign
    cmd.arg(&options.path);

    debug!(
        "Running signtool command: {} sign /fd SHA256 ...",
        signtool.display()
    );

    let output = cmd.output().await.map_err(|e| {
        error!("Failed to execute signtool: {}", e);
        CodesignError::tool_not_found(format!("Failed to execute signtool: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if stderr.is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };
        error!("signtool failed: {}", error_msg);
        return Err(CodesignError::signing_failed(error_msg));
    }

    debug!("Successfully signed: {}", options.path);
    Ok(())
}

/// Ad-hoc signing is not supported on Windows
pub async fn sign_adhoc(_path: &str) -> Result<(), CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Ad-hoc signing is only supported on macOS. On Windows, use a self-signed certificate.",
    ))
}

/// Verify a code signature using signtool verify
pub async fn verify(path: &str) -> Result<VerifyResult, CodesignError> {
    let signtool = find_signtool()?;

    let output = Command::new(&signtool)
        .args(["verify", "/pa", "/v", path])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute signtool: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute signtool: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let valid = output.status.success();

    // Parse signer info from output
    let signer = parse_signer_from_signtool(&stdout);
    let timestamp = parse_timestamp_from_signtool(&stdout);

    let message = if valid {
        format!("Valid signature on {}", path)
    } else {
        stdout.to_string()
    };

    Ok(VerifyResult {
        valid,
        signer,
        timestamp,
        message,
    })
}

/// Parse signer name from signtool verify output
fn parse_signer_from_signtool(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Issued to:") {
            return Some(line.trim_start_matches("Issued to:").trim().to_string());
        }
        if line.starts_with("Signing Certificate Chain:") {
            // Next non-empty line usually has the signer
            continue;
        }
    }
    None
}

/// Parse timestamp from signtool verify output
fn parse_timestamp_from_signtool(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Timestamp:") {
            return Some(line.trim_start_matches("Timestamp:").trim().to_string());
        }
    }
    None
}

/// Get entitlements - not applicable on Windows
pub async fn get_entitlements(_path: &str) -> Result<String, CodesignError> {
    Err(CodesignError::platform_unsupported(
        "Entitlements are a macOS-only concept. Windows uses manifests for application capabilities.",
    ))
}

/// List available signing identities using PowerShell
pub async fn list_identities() -> Result<Vec<SigningIdentity>, CodesignError> {
    // Use PowerShell to list code signing certificates
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Select-Object Thumbprint, Subject, NotAfter, @{N='Valid';E={$_.NotAfter -gt (Get-Date)}} | ConvertTo-Json -Compress"#,
        ])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute PowerShell: {}", e);
            CodesignError::tool_not_found(format!("Failed to execute PowerShell: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("PowerShell returned error: {}", stderr);
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_powershell_cert_output(&stdout)
}

/// Parse PowerShell certificate output JSON
fn parse_powershell_cert_output(output: &str) -> Result<Vec<SigningIdentity>, CodesignError> {
    let output = output.trim();

    if output.is_empty() {
        return Ok(Vec::new());
    }

    // PowerShell returns single object without array brackets if only one cert
    let json_output = if output.starts_with('[') {
        output.to_string()
    } else if output.starts_with('{') {
        format!("[{}]", output)
    } else {
        return Ok(Vec::new());
    };

    let certs: Vec<PowerShellCert> = serde_json::from_str(&json_output).map_err(|e| {
        warn!("Failed to parse PowerShell output: {}", e);
        CodesignError::generic(format!("Failed to parse certificate list: {}", e))
    })?;

    Ok(certs.into_iter().map(|c| c.into()).collect())
}

/// PowerShell certificate output structure
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PowerShellCert {
    thumbprint: String,
    subject: String,
    not_after: String,
    valid: bool,
}

impl From<PowerShellCert> for SigningIdentity {
    fn from(cert: PowerShellCert) -> Self {
        // Parse subject to extract CN
        let name = parse_cn_from_subject(&cert.subject).unwrap_or(cert.subject.clone());

        SigningIdentity {
            id: cert.thumbprint,
            name,
            expires: Some(cert.not_after),
            valid: cert.valid,
            identity_type: "code_signing".to_string(),
        }
    }
}

/// Parse CN (Common Name) from certificate subject
fn parse_cn_from_subject(subject: &str) -> Option<String> {
    for part in subject.split(',') {
        let part = part.trim();
        if part.starts_with("CN=") {
            return Some(part.trim_start_matches("CN=").to_string());
        }
    }
    None
}

/// Get detailed information about a specific signing identity
pub async fn get_identity_info(identity: &str) -> Result<SigningIdentity, CodesignError> {
    // List all identities and find the matching one
    let identities = list_identities().await?;

    // Search by thumbprint or by name
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

/// Check if a string looks like a SHA-1 thumbprint (40 hex characters)
fn is_sha1_thumbprint(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Check available capabilities on Windows
pub fn check_capabilities() -> CodesignCapabilities {
    let signtool = find_signtool().is_ok();

    let certutil = std::process::Command::new("certutil")
        .arg("-?")
        .output()
        .is_ok();

    CodesignCapabilities {
        codesign: false,
        security: false,
        signtool,
        certutil,
        platform: "windows".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sha1_thumbprint() {
        assert!(is_sha1_thumbprint(
            "ABCDEF0123456789ABCDEF0123456789ABCDEF01"
        ));
        assert!(is_sha1_thumbprint(
            "abcdef0123456789abcdef0123456789abcdef01"
        ));
        assert!(!is_sha1_thumbprint("too-short"));
        assert!(!is_sha1_thumbprint(
            "GHIJKL0123456789ABCDEF0123456789ABCDEF01"
        )); // G-L not hex
    }

    #[test]
    fn test_parse_cn_from_subject() {
        assert_eq!(
            parse_cn_from_subject("CN=Test Company, O=Test Org"),
            Some("Test Company".to_string())
        );
        assert_eq!(
            parse_cn_from_subject("O=Test Org, CN=Test Company"),
            Some("Test Company".to_string())
        );
        assert_eq!(parse_cn_from_subject("O=Test Org"), None);
    }

    #[test]
    fn test_parse_powershell_cert_output_empty() {
        let result = parse_powershell_cert_output("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_powershell_cert_output_single() {
        let json =
            r#"{"Thumbprint":"ABC123","Subject":"CN=Test","NotAfter":"2025-01-01","Valid":true}"#;
        let result = parse_powershell_cert_output(json).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "ABC123");
        assert_eq!(result[0].name, "Test");
    }
}
