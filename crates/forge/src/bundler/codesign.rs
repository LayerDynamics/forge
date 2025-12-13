//! Code signing utilities for platform packaging
//!
//! Provides unified code signing functionality:
//! - macOS: codesign + notarytool
//! - Windows: SignTool from Windows SDK
//!
//! This module can be used standalone via `forge sign` command
//! or integrated into the bundling pipeline.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Code signing configuration
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Path to certificate file (Windows .pfx, or macOS identity name)
    pub identity: String,
    /// Optional password for certificate (Windows)
    pub password: Option<String>,
    /// Team ID (macOS notarization)
    pub team_id: Option<String>,
    /// Path to entitlements file (macOS)
    pub entitlements: Option<PathBuf>,
    /// Whether to notarize (macOS)
    pub notarize: bool,
    /// Keychain profile name for notarization (macOS)
    pub keychain_profile: Option<String>,
}

#[allow(dead_code)]
impl SigningConfig {
    pub fn new(identity: String) -> Self {
        Self {
            identity,
            password: None,
            team_id: None,
            entitlements: None,
            notarize: false,
            keychain_profile: None,
        }
    }

    pub fn with_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    pub fn with_team_id(mut self, team_id: Option<String>) -> Self {
        self.team_id = team_id;
        self
    }

    pub fn with_entitlements(mut self, entitlements: Option<PathBuf>) -> Self {
        self.entitlements = entitlements;
        self
    }

    pub fn with_notarize(mut self, notarize: bool) -> Self {
        self.notarize = notarize;
        self
    }

    pub fn with_keychain_profile(mut self, profile: Option<String>) -> Self {
        self.keychain_profile = profile;
        self
    }
}

/// Sign an artifact based on its type and current platform
pub fn sign(path: &Path, config: &SigningConfig) -> Result<()> {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension.to_lowercase().as_str() {
        // macOS artifacts
        "app" => sign_macos_bundle(path, config),
        "dmg" => sign_macos_dmg(path, config),

        // Windows artifacts
        "msix" | "exe" | "dll" => sign_windows(path, config),

        // Linux doesn't have standard signing, but we can GPG sign
        "appimage" => sign_linux_gpg(path, config),

        _ => bail!(
            "Unknown artifact type: {}. Supported: .app, .dmg, .msix, .exe, .dll, .AppImage",
            extension
        ),
    }
}

// =============================================================================
// macOS Signing
// =============================================================================

/// Sign a macOS .app bundle with codesign
pub fn sign_macos_bundle(bundle_path: &Path, config: &SigningConfig) -> Result<()> {
    println!("Signing macOS bundle: {}", bundle_path.display());

    if !bundle_path.exists() {
        bail!("Bundle not found: {}", bundle_path.display());
    }

    // First sign any embedded frameworks/helpers
    let frameworks_dir = bundle_path.join("Contents/Frameworks");
    if frameworks_dir.exists() {
        for entry in fs::read_dir(&frameworks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map(|e| e == "framework" || e == "dylib")
                .unwrap_or(false)
            {
                sign_single_macos(&path, config, false)?;
            }
        }
    }

    // Sign helper apps
    let helpers_dir = bundle_path.join("Contents/Helpers");
    if helpers_dir.exists() {
        for entry in fs::read_dir(&helpers_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "app").unwrap_or(false) {
                sign_single_macos(&path, config, true)?;
            }
        }
    }

    // Sign the main bundle
    sign_single_macos(bundle_path, config, true)?;

    // Verify signature
    verify_macos_signature(bundle_path)?;

    println!("  Bundle signed successfully");
    Ok(())
}

/// Sign a single macOS artifact (bundle or binary)
fn sign_single_macos(path: &Path, config: &SigningConfig, deep: bool) -> Result<()> {
    let mut cmd = Command::new("codesign");

    cmd.args([
        "--sign",
        &config.identity,
        "--force",
        "--timestamp",
        "--options",
        "runtime", // Hardened runtime, required for notarization
    ]);

    if deep {
        cmd.arg("--deep");
    }

    if let Some(ref entitlements) = config.entitlements {
        cmd.args(["--entitlements", &entitlements.display().to_string()]);
    }

    cmd.arg(path.display().to_string());

    let output = cmd.output().context("Failed to run codesign")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("codesign failed: {}", stderr);
    }

    Ok(())
}

/// Verify a macOS code signature
pub fn verify_macos_signature(path: &Path) -> Result<()> {
    let output = Command::new("codesign")
        .args([
            "--verify",
            "--deep",
            "--strict",
            "--verbose=2",
            &path.display().to_string(),
        ])
        .output()
        .context("Failed to verify code signature")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Signature verification failed: {}", stderr);
    }

    Ok(())
}

/// Sign a macOS DMG
pub fn sign_macos_dmg(dmg_path: &Path, config: &SigningConfig) -> Result<()> {
    println!("Signing DMG: {}", dmg_path.display());

    let mut cmd = Command::new("codesign");
    cmd.args([
        "--sign",
        &config.identity,
        "--force",
        "--timestamp",
        &dmg_path.display().to_string(),
    ]);

    let output = cmd.output().context("Failed to sign DMG")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("DMG signing failed: {}", stderr);
    }

    // Optionally notarize
    if config.notarize {
        notarize_macos(dmg_path, config)?;
    }

    println!("  DMG signed successfully");
    Ok(())
}

/// Notarize a macOS artifact with Apple
pub fn notarize_macos(path: &Path, config: &SigningConfig) -> Result<()> {
    let team_id = config
        .team_id
        .as_ref()
        .context("Team ID required for notarization")?;

    let keychain_profile = config
        .keychain_profile
        .as_deref()
        .unwrap_or("forge-notarize");

    println!("  Submitting for notarization (this may take several minutes)...");

    // Submit for notarization
    let submit_output = Command::new("xcrun")
        .args([
            "notarytool",
            "submit",
            &path.display().to_string(),
            "--keychain-profile",
            keychain_profile,
            "--team-id",
            team_id,
            "--wait",
        ])
        .output()
        .context("Failed to submit for notarization")?;

    if !submit_output.status.success() {
        let stderr = String::from_utf8_lossy(&submit_output.stderr);
        let stdout = String::from_utf8_lossy(&submit_output.stdout);

        // Check if it's a credentials issue
        if stderr.contains("credentials") || stdout.contains("credentials") {
            bail!(
                "Notarization failed - credentials not configured.\n\
                Set up credentials with:\n\
                  xcrun notarytool store-credentials {}\n\
                Then provide your Apple ID, team ID, and app-specific password.",
                keychain_profile
            );
        }

        bail!("Notarization failed: {}\n{}", stderr, stdout);
    }

    println!("  Notarization successful");

    // Staple the notarization ticket
    staple_notarization(path)?;

    Ok(())
}

/// Staple a notarization ticket to an artifact
pub fn staple_notarization(path: &Path) -> Result<()> {
    println!("  Stapling notarization ticket...");

    let output = Command::new("xcrun")
        .args(["stapler", "staple", &path.display().to_string()])
        .output()
        .context("Failed to staple notarization ticket")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Stapling failure is non-fatal, the app will still work
        println!("  Warning: Failed to staple ticket: {}", stderr);
    } else {
        println!("  Ticket stapled successfully");
    }

    Ok(())
}

/// Check notarization status
#[allow(dead_code)]
pub fn check_notarization_status(submission_id: &str, keychain_profile: &str) -> Result<String> {
    let output = Command::new("xcrun")
        .args([
            "notarytool",
            "info",
            submission_id,
            "--keychain-profile",
            keychain_profile,
        ])
        .output()
        .context("Failed to check notarization status")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// =============================================================================
// Windows Signing
// =============================================================================

/// Sign a Windows artifact with SignTool
pub fn sign_windows(path: &Path, config: &SigningConfig) -> Result<()> {
    println!("Signing Windows artifact: {}", path.display());

    let signtool = find_signtool()?;

    let mut cmd = Command::new(&signtool);
    cmd.args([
        "sign",
        "/fd",
        "SHA256",
        "/tr",
        "http://timestamp.digicert.com",
    ]);
    cmd.args(["/td", "SHA256"]);
    cmd.args(["/f", &config.identity]);

    if let Some(ref password) = config.password {
        cmd.args(["/p", password]);
    }

    cmd.arg(path.display().to_string());

    let output = cmd.output().context("Failed to run SignTool")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!("SignTool failed: {}\n{}", stderr, stdout);
    }

    // Verify signature
    verify_windows_signature(path)?;

    println!("  Artifact signed successfully");
    Ok(())
}

/// Verify a Windows code signature
pub fn verify_windows_signature(path: &Path) -> Result<()> {
    let signtool = find_signtool()?;

    let output = Command::new(&signtool)
        .args(["verify", "/pa", "/v", &path.display().to_string()])
        .output()
        .context("Failed to verify signature")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!("Signature verification failed: {}\n{}", stderr, stdout);
    }

    Ok(())
}

/// Find SignTool.exe in Windows SDK locations
pub fn find_signtool() -> Result<PathBuf> {
    // Check PATH first
    if let Ok(output) = Command::new("where").arg("signtool.exe").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    // Common Windows SDK paths
    let sdk_base = r"C:\Program Files (x86)\Windows Kits\10\bin";

    if let Ok(entries) = fs::read_dir(sdk_base) {
        let mut versions: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();

        // Sort descending to prefer newer versions
        versions.sort();
        versions.reverse();

        for version_dir in versions {
            let signtool = version_dir.join("x64").join("signtool.exe");
            if signtool.exists() {
                return Ok(signtool);
            }
            // Also check x86 if x64 not found
            let signtool_x86 = version_dir.join("x86").join("signtool.exe");
            if signtool_x86.exists() {
                return Ok(signtool_x86);
            }
        }
    }

    bail!(
        "SignTool.exe not found.\n\
        Install Windows SDK from:\n\
        https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/\n\
        Or add SignTool to PATH."
    )
}

/// Create a self-signed certificate for testing (Windows)
#[allow(dead_code)]
pub fn create_test_certificate(subject: &str, output_path: &Path, password: &str) -> Result<()> {
    println!("Creating self-signed certificate for testing...");

    // Use PowerShell to create a self-signed certificate
    let script = format!(
        r#"
        $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN={}" -CertStoreLocation Cert:\CurrentUser\My
        $pwd = ConvertTo-SecureString -String "{}" -Force -AsPlainText
        Export-PfxCertificate -Cert $cert -FilePath "{}" -Password $pwd
        "#,
        subject,
        password,
        output_path.display()
    );

    let output = Command::new("powershell")
        .args(["-Command", &script])
        .output()
        .context("Failed to create self-signed certificate")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create certificate: {}", stderr);
    }

    println!("  Certificate created: {}", output_path.display());
    println!("  Note: This is for testing only. Use a trusted certificate for distribution.");

    Ok(())
}

// =============================================================================
// Linux Signing (GPG)
// =============================================================================

/// Sign a Linux artifact with GPG (optional)
pub fn sign_linux_gpg(path: &Path, config: &SigningConfig) -> Result<()> {
    println!("Signing Linux artifact with GPG: {}", path.display());

    // Check if GPG is available
    if Command::new("gpg").arg("--version").output().is_err() {
        println!("  Warning: GPG not available, skipping signature");
        return Ok(());
    }

    let sig_path = format!("{}.sig", path.display());

    let mut cmd = Command::new("gpg");
    cmd.args(["--detach-sign", "--armor", "--output", &sig_path]);

    // Use specific key if provided
    if !config.identity.is_empty() {
        cmd.args(["--local-user", &config.identity]);
    }

    cmd.arg(path.display().to_string());

    let output = cmd.output().context("Failed to run GPG")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // GPG signing failure is non-fatal for Linux
        println!("  Warning: GPG signing failed: {}", stderr);
        return Ok(());
    }

    println!("  Signature created: {}", sig_path);
    Ok(())
}

/// Verify a GPG signature
#[allow(dead_code)]
pub fn verify_linux_gpg(path: &Path) -> Result<bool> {
    let sig_path = format!("{}.sig", path.display());

    if !PathBuf::from(&sig_path).exists() {
        return Ok(false);
    }

    let output = Command::new("gpg")
        .args(["--verify", &sig_path, &path.display().to_string()])
        .output()
        .context("Failed to verify GPG signature")?;

    Ok(output.status.success())
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Detect available signing tools on the current system
pub fn detect_signing_capabilities() -> SigningCapabilities {
    SigningCapabilities {
        codesign: Command::new("codesign").arg("--help").output().is_ok(),
        notarytool: Command::new("xcrun")
            .args(["notarytool", "--help"])
            .output()
            .is_ok(),
        signtool: find_signtool().is_ok(),
        gpg: Command::new("gpg").arg("--version").output().is_ok(),
    }
}

/// Available signing tools
#[derive(Debug)]
pub struct SigningCapabilities {
    pub codesign: bool,
    pub notarytool: bool,
    pub signtool: bool,
    pub gpg: bool,
}

impl std::fmt::Display for SigningCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Signing capabilities:")?;
        writeln!(
            f,
            "  macOS codesign: {}",
            if self.codesign { "✓" } else { "✗" }
        )?;
        writeln!(
            f,
            "  macOS notarytool: {}",
            if self.notarytool { "✓" } else { "✗" }
        )?;
        writeln!(
            f,
            "  Windows SignTool: {}",
            if self.signtool { "✓" } else { "✗" }
        )?;
        writeln!(f, "  GPG: {}", if self.gpg { "✓" } else { "✗" })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_config_builder() {
        let config = SigningConfig::new("Developer ID Application: Test".into())
            .with_team_id(Some("ABC123".into()))
            .with_notarize(true);

        assert_eq!(config.identity, "Developer ID Application: Test");
        assert_eq!(config.team_id, Some("ABC123".into()));
        assert!(config.notarize);
    }

    #[test]
    fn test_detect_capabilities() {
        // Just ensure this doesn't panic
        let caps = detect_signing_capabilities();
        println!("{}", caps);
    }
}
