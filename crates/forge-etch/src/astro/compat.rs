//! Astro version compatibility utilities
//!
//! This module provides utilities for detecting Astro versions and
//! handling differences between major versions.

use crate::diagnostics::{EtchError, EtchResult};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Supported Astro major versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AstroVersion {
    /// Astro 3.x
    V3,
    /// Astro 4.x (current stable)
    #[default]
    V4,
    /// Astro 5.x (future/beta)
    V5,
    /// Unknown version
    Unknown,
}

impl AstroVersion {
    /// Parse version from a semver string (e.g., "4.15.0", "^4.0.0")
    pub fn from_version_str(version: &str) -> Self {
        // Strip leading ^ or ~ if present
        let clean = version
            .trim_start_matches('^')
            .trim_start_matches('~')
            .trim_start_matches('v');

        // Extract major version
        let major = clean.split('.').next().and_then(|s| s.parse::<u32>().ok());

        match major {
            Some(3) => AstroVersion::V3,
            Some(4) => AstroVersion::V4,
            Some(5) => AstroVersion::V5,
            _ => AstroVersion::Unknown,
        }
    }

    /// Check if this version supports content collections v2
    pub fn supports_content_collections_v2(&self) -> bool {
        matches!(self, AstroVersion::V4 | AstroVersion::V5)
    }

    /// Check if this version uses the new config format
    pub fn uses_new_config_format(&self) -> bool {
        matches!(self, AstroVersion::V4 | AstroVersion::V5)
    }
}

/// Compatibility configuration detected from the project
#[derive(Debug, Clone, Default)]
pub struct CompatConfig {
    /// Detected Astro version
    pub astro_version: AstroVersion,
    /// Detected Starlight version (if present)
    pub starlight_version: Option<String>,
    /// Whether content collections v2 is supported
    pub content_collections_v2: bool,
    /// Whether the project uses TypeScript
    pub uses_typescript: bool,
}

/// Frontmatter style based on Astro version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterStyle {
    /// Standard YAML frontmatter (Astro 3.x)
    Standard,
    /// Extended frontmatter with Starlight fields (Astro 4.x+)
    Starlight,
}

impl FrontmatterStyle {
    /// Get required frontmatter fields for this style
    pub fn required_fields(&self) -> &[&str] {
        match self {
            FrontmatterStyle::Standard => &["title"],
            FrontmatterStyle::Starlight => &["title"],
        }
    }

    /// Get optional frontmatter fields for this style
    pub fn optional_fields(&self) -> &[&str] {
        match self {
            FrontmatterStyle::Standard => &["description", "layout"],
            FrontmatterStyle::Starlight => &[
                "description",
                "sidebar",
                "tableOfContents",
                "editUrl",
                "head",
                "banner",
                "hero",
                "lastUpdated",
                "prev",
                "next",
                "pagefind",
            ],
        }
    }
}

/// Partial package.json structure for version detection
#[derive(Debug, Deserialize)]
struct PackageJson {
    dependencies: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<std::collections::HashMap<String, String>>,
}

/// Detect Astro version and compatibility settings from a project directory.
///
/// Reads package.json to determine the Astro version and checks for
/// Starlight integration.
pub fn detect_version(project_dir: &Path) -> EtchResult<CompatConfig> {
    let package_json_path = project_dir.join("package.json");

    if !package_json_path.exists() {
        return Ok(CompatConfig::default());
    }

    let content = fs::read_to_string(&package_json_path).map_err(|e| {
        EtchError::Io(std::io::Error::other(format!(
            "Failed to read package.json: {}",
            e
        )))
    })?;

    let package: PackageJson = serde_json::from_str(&content).map_err(|e| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse package.json: {}", e),
        ))
    })?;

    let mut config = CompatConfig::default();

    // Check both dependencies and devDependencies
    let all_deps: Vec<(&String, &String)> = package
        .dependencies
        .iter()
        .flatten()
        .chain(package.dev_dependencies.iter().flatten())
        .collect();

    for (name, version) in all_deps {
        match name.as_str() {
            "astro" => {
                config.astro_version = AstroVersion::from_version_str(version);
            }
            "@astrojs/starlight" => {
                config.starlight_version = Some(version.clone());
            }
            "typescript" => {
                config.uses_typescript = true;
            }
            _ => {}
        }
    }

    // Set content collections v2 support based on version
    config.content_collections_v2 = config.astro_version.supports_content_collections_v2();

    Ok(config)
}

/// Get the appropriate frontmatter style for an Astro version.
pub fn frontmatter_for_version(version: AstroVersion) -> FrontmatterStyle {
    match version {
        AstroVersion::V3 => FrontmatterStyle::Standard,
        AstroVersion::V4 | AstroVersion::V5 => FrontmatterStyle::Starlight,
        AstroVersion::Unknown => FrontmatterStyle::Starlight, // Default to modern style
    }
}

/// Check if a specific feature is supported in the given version.
pub fn supports_feature(version: AstroVersion, feature: &str) -> bool {
    match feature {
        "content_collections_v2" => version.supports_content_collections_v2(),
        "view_transitions" => matches!(
            version,
            AstroVersion::V3 | AstroVersion::V4 | AstroVersion::V5
        ),
        "server_islands" => matches!(version, AstroVersion::V4 | AstroVersion::V5),
        "experimental_actions" => matches!(version, AstroVersion::V4 | AstroVersion::V5),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_version_parsing() {
        assert_eq!(AstroVersion::from_version_str("4.15.0"), AstroVersion::V4);
        assert_eq!(AstroVersion::from_version_str("^4.0.0"), AstroVersion::V4);
        assert_eq!(AstroVersion::from_version_str("~3.6.0"), AstroVersion::V3);
        assert_eq!(
            AstroVersion::from_version_str("5.0.0-beta.1"),
            AstroVersion::V5
        );
        assert_eq!(
            AstroVersion::from_version_str("invalid"),
            AstroVersion::Unknown
        );
    }

    #[test]
    fn test_content_collections_support() {
        assert!(!AstroVersion::V3.supports_content_collections_v2());
        assert!(AstroVersion::V4.supports_content_collections_v2());
        assert!(AstroVersion::V5.supports_content_collections_v2());
    }

    #[test]
    fn test_frontmatter_style() {
        assert_eq!(
            frontmatter_for_version(AstroVersion::V3),
            FrontmatterStyle::Standard
        );
        assert_eq!(
            frontmatter_for_version(AstroVersion::V4),
            FrontmatterStyle::Starlight
        );
    }

    #[test]
    fn test_detect_version_with_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");

        let content = r#"{
            "dependencies": {
                "astro": "^4.15.0",
                "@astrojs/starlight": "^0.28.0"
            },
            "devDependencies": {
                "typescript": "^5.0.0"
            }
        }"#;

        let mut file = fs::File::create(&package_json).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let config = detect_version(temp_dir.path()).unwrap();
        assert_eq!(config.astro_version, AstroVersion::V4);
        assert!(config.starlight_version.is_some());
        assert!(config.uses_typescript);
        assert!(config.content_collections_v2);
    }

    #[test]
    fn test_detect_version_missing_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = detect_version(temp_dir.path()).unwrap();
        assert_eq!(config.astro_version, AstroVersion::V4); // Default
    }

    #[test]
    fn test_supports_feature() {
        assert!(!supports_feature(
            AstroVersion::V3,
            "content_collections_v2"
        ));
        assert!(supports_feature(AstroVersion::V4, "content_collections_v2"));
        assert!(supports_feature(AstroVersion::V4, "view_transitions"));
        assert!(!supports_feature(AstroVersion::V3, "server_islands"));
    }

    #[test]
    fn test_frontmatter_fields() {
        let standard = FrontmatterStyle::Standard;
        assert!(standard.required_fields().contains(&"title"));

        let starlight = FrontmatterStyle::Starlight;
        assert!(starlight.optional_fields().contains(&"sidebar"));
        assert!(starlight.optional_fields().contains(&"tableOfContents"));
    }
}
