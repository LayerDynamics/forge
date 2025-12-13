//! Bundle configuration parsing from manifest.app.toml
//!
//! Extends the app manifest with platform-specific bundle settings.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Full app manifest with bundle configuration
#[derive(Debug, Deserialize, Clone)]
pub struct AppManifest {
    pub app: AppConfig,
    #[serde(default)]
    #[allow(dead_code)] // Used by forge-host, not bundler
    pub windows: WindowConfig,
    #[serde(default)]
    pub bundle: BundleConfig,
    #[serde(default)]
    #[allow(dead_code)] // Used by forge-host, not bundler
    pub permissions: Option<toml::Value>,
}

/// Core app identification
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub identifier: String,
    pub version: String,
}

/// Window configuration (existing)
#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)] // Used by forge-host, not bundler
pub struct WindowConfig {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub resizable: Option<bool>,
}

/// Bundle configuration for all platforms
#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct BundleConfig {
    /// Path to icon (without extension), e.g., "assets/icon"
    /// Will look for .png, .icns, .ico variants
    pub icon: Option<String>,
    /// Windows-specific bundle settings
    pub windows: Option<WindowsBundleConfig>,
    /// macOS-specific bundle settings
    pub macos: Option<MacosBundleConfig>,
    /// Linux-specific bundle settings
    pub linux: Option<LinuxBundleConfig>,
}

/// Windows MSIX bundle configuration
#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct WindowsBundleConfig {
    /// Package format: "msix" (default)
    pub format: Option<String>,
    /// Enable code signing
    pub sign: Option<bool>,
    /// Path to .pfx certificate file
    pub certificate: Option<String>,
    /// Certificate password (can be $ENV_VAR reference)
    pub password: Option<String>,
    /// Publisher Distinguished Name (e.g., "CN=My Company, O=My Company, C=US")
    pub publisher: Option<String>,
    /// Minimum Windows version (default: "10.0.17763.0" for Windows 10 1809)
    pub min_version: Option<String>,
    /// Additional capabilities beyond defaults
    pub capabilities: Option<Vec<String>>,
}

/// macOS bundle configuration
#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct MacosBundleConfig {
    /// Enable code signing
    pub sign: Option<bool>,
    /// Enable notarization (requires sign=true)
    pub notarize: Option<bool>,
    /// Apple Developer Team ID
    pub team_id: Option<String>,
    /// Signing identity (e.g., "Developer ID Application: My Company (TEAMID)")
    pub signing_identity: Option<String>,
    /// Path to entitlements.plist file
    pub entitlements: Option<String>,
    /// App Store category (e.g., "public.app-category.developer-tools")
    pub category: Option<String>,
    /// Minimum macOS version (default: "12.0")
    pub minimum_system_version: Option<String>,
}

/// Linux AppImage bundle configuration
#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct LinuxBundleConfig {
    /// Package format: "appimage" (default) or "tarball"
    pub format: Option<String>,
    /// Desktop entry categories (e.g., ["Development", "Utility"])
    pub categories: Option<Vec<String>>,
    /// Generic name for desktop entry
    pub generic_name: Option<String>,
    /// Comment/description for desktop entry
    pub comment: Option<String>,
    /// Supported MIME types
    pub mime_types: Option<Vec<String>>,
    /// Whether to run in terminal
    pub terminal: Option<bool>,
}

impl AppManifest {
    /// Parse manifest from app directory
    pub fn from_app_dir(app_dir: &Path) -> Result<Self> {
        let manifest_path = app_dir.join("manifest.app.toml");
        Self::from_file(&manifest_path)
    }

    /// Parse manifest from file path
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at {}", path.display()))?;
        let manifest: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest at {}", path.display()))?;
        Ok(manifest)
    }

    /// Get the executable name (sanitized app name)
    #[allow(dead_code)]
    pub fn executable_name(&self) -> String {
        sanitize_name(&self.app.name)
    }

    /// Get version in 4-part format for Windows (major.minor.build.0)
    #[cfg(target_os = "windows")]
    pub fn version_quad(&self) -> String {
        normalize_version(&self.app.version)
    }
}

#[cfg(target_os = "windows")]
impl WindowsBundleConfig {
    /// Get the publisher DN, defaulting to CN={app_name}
    pub fn publisher_or_default(&self, app_name: &str) -> String {
        self.publisher
            .clone()
            .unwrap_or_else(|| format!("CN={}", app_name))
    }

    /// Get minimum version, defaulting to Windows 10 1809
    pub fn min_version_or_default(&self) -> String {
        self.min_version
            .clone()
            .unwrap_or_else(|| "10.0.17763.0".to_string())
    }

    /// Get capabilities, defaulting to internetClient
    pub fn capabilities_or_default(&self) -> Vec<String> {
        self.capabilities
            .clone()
            .unwrap_or_else(|| vec!["internetClient".to_string()])
    }

    /// Resolve password, checking for $ENV_VAR references
    pub fn resolve_password(&self) -> Option<String> {
        self.password.as_ref().map(|p| {
            if p.starts_with('$') {
                std::env::var(&p[1..]).unwrap_or_else(|_| p.clone())
            } else {
                p.clone()
            }
        })
    }
}

#[cfg(target_os = "macos")]
impl MacosBundleConfig {
    /// Get minimum system version, defaulting to macOS 12.0
    pub fn min_version_or_default(&self) -> String {
        self.minimum_system_version
            .clone()
            .unwrap_or_else(|| "12.0".to_string())
    }

    /// Get app category, defaulting to developer-tools
    pub fn category_or_default(&self) -> String {
        self.category
            .clone()
            .unwrap_or_else(|| "public.app-category.developer-tools".to_string())
    }
}

#[cfg(target_os = "linux")]
impl LinuxBundleConfig {
    /// Get format, defaulting to appimage
    pub fn format_or_default(&self) -> String {
        self.format
            .clone()
            .unwrap_or_else(|| "appimage".to_string())
    }

    /// Get categories, defaulting to Utility
    pub fn categories_or_default(&self) -> Vec<String> {
        self.categories
            .clone()
            .unwrap_or_else(|| vec!["Utility".to_string()])
    }

    /// Get generic name, defaulting to app name
    pub fn generic_name_or_default(&self, app_name: &str) -> String {
        self.generic_name
            .clone()
            .unwrap_or_else(|| app_name.to_string())
    }

    /// Get comment, defaulting to "{app_name} application"
    pub fn comment_or_default(&self, app_name: &str) -> String {
        self.comment
            .clone()
            .unwrap_or_else(|| format!("{} application", app_name))
    }
}

/// Sanitize a name for use as executable/identifier
/// - Lowercase
/// - Replace spaces with hyphens
/// - Remove non-alphanumeric characters (except hyphens)
pub fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '_' {
                '-'
            } else if c == '-' {
                c
            } else {
                // Skip invalid characters
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect()
}

/// Sanitize identifier for MSIX package name
/// MSIX allows: a-z, A-Z, 0-9, period, hyphen
/// Length: 3-50 characters, no leading/trailing periods
#[cfg(target_os = "windows")]
pub fn sanitize_msix_name(identifier: &str) -> String {
    let sanitized: String = identifier
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' => c,
            _ => '-',
        })
        .collect();

    let trimmed = sanitized.trim_matches('.');
    if trimmed.len() < 3 {
        format!("{}-app", trimmed)
    } else if trimmed.len() > 50 {
        trimmed[..50].to_string()
    } else {
        trimmed.to_string()
    }
}

/// Normalize version to 4-part format for Windows MSIX
/// Input: "1.0.0" or "1.0" or "1"
/// Output: "1.0.0.0"
/// Note: Last part must be 0 for Store submissions
#[cfg(target_os = "windows")]
pub fn normalize_version(version: &str) -> String {
    // Strip any pre-release suffix (e.g., "-beta.1")
    let clean_version = version.split('-').next().unwrap_or(version);
    let parts: Vec<&str> = clean_version.split('.').collect();

    match parts.len() {
        1 => format!("{}.0.0.0", parts[0]),
        2 => format!("{}.{}.0.0", parts[0], parts[1]),
        3 => format!("{}.{}.{}.0", parts[0], parts[1], parts[2]),
        _ => format!("{}.{}.{}.0", parts[0], parts[1], parts[2]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My App Name"), "my-app-name");
        assert_eq!(sanitize_name("HelloWorld"), "helloworld");
        assert_eq!(sanitize_name("Test_App!@#"), "test-app");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_sanitize_msix_name() {
        assert_eq!(sanitize_msix_name("com.example.app"), "com.example.app");
        assert_eq!(sanitize_msix_name("My App!"), "My-App-");
        assert_eq!(sanitize_msix_name("ab"), "ab-app"); // Too short
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("1"), "1.0.0.0");
        assert_eq!(normalize_version("1.0"), "1.0.0.0");
        assert_eq!(normalize_version("1.0.0"), "1.0.0.0");
        assert_eq!(normalize_version("1.2.3.4"), "1.2.3.0");
        assert_eq!(normalize_version("1.0.0-beta.1"), "1.0.0.0");
    }
}
