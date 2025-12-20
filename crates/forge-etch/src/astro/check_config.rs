//! Astro configuration validation
//!
//! This module provides utilities for parsing and validating Astro
//! project configuration, including Starlight sidebar entries and
//! content collection setup.

use crate::diagnostics::{EtchError, EtchResult};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Parsed Astro configuration
#[derive(Debug, Clone, Default)]
pub struct AstroConfig {
    /// Site URL (from `site` field in config)
    pub site_url: Option<String>,
    /// Starlight-specific configuration
    pub starlight: Option<StarlightConfig>,
    /// Output directory for generated documentation
    pub output_dir: PathBuf,
    /// Path to the astro.config.mjs file
    pub config_path: PathBuf,
    /// Whether content.config.ts exists
    pub has_content_config: bool,
}

/// Starlight-specific configuration
#[derive(Debug, Clone, Default)]
pub struct StarlightConfig {
    /// Site title
    pub title: String,
    /// Site description
    pub description: Option<String>,
    /// Sidebar configuration
    pub sidebar: Vec<SidebarItem>,
}

/// Sidebar item configuration
#[derive(Debug, Clone)]
pub enum SidebarItem {
    /// Manual list of items
    Manual {
        /// Group label
        label: String,
        /// List of page slugs
        items: Vec<String>,
    },
    /// Auto-generated from directory
    Autogenerate {
        /// Group label
        label: String,
        /// Directory to autogenerate from
        directory: String,
    },
}

/// Validation result for a specific check
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the check passed
    pub passed: bool,
    /// Description of what was checked
    pub description: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Check Astro configuration in a project directory.
///
/// Parses astro.config.mjs and validates the configuration structure,
/// including Starlight sidebar entries and content collections.
pub fn check_config(project_dir: &Path) -> EtchResult<AstroConfig> {
    let config_path = find_config_file(project_dir)?;
    let content = fs::read_to_string(&config_path).map_err(|e| {
        EtchError::Io(std::io::Error::other(format!(
            "Failed to read astro config: {}",
            e
        )))
    })?;

    let mut config = AstroConfig {
        config_path: config_path.clone(),
        output_dir: project_dir.join("src/content/docs"),
        ..Default::default()
    };

    // Parse site URL
    config.site_url = parse_site_url(&content);

    // Parse Starlight config if present
    if content.contains("starlight(") || content.contains("@astrojs/starlight") {
        config.starlight = Some(parse_starlight_config(&content)?);
    }

    // Check for content.config.ts
    let content_config_path = project_dir.join("src/content.config.ts");
    config.has_content_config = content_config_path.exists();

    Ok(config)
}

/// Validate that an output directory is properly configured in the sidebar.
///
/// Checks that the target directory either exists in an autogenerate entry
/// or has corresponding manual sidebar entries.
pub fn validate_output_dir(config: &AstroConfig, target: &str) -> EtchResult<()> {
    let starlight = config.starlight.as_ref().ok_or_else(|| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Starlight configuration found",
        ))
    })?;

    // Check if target directory is in an autogenerate entry
    for item in &starlight.sidebar {
        match item {
            SidebarItem::Autogenerate { directory, .. } => {
                if directory == target || target.starts_with(&format!("{}/", directory)) {
                    return Ok(());
                }
            }
            SidebarItem::Manual { items, .. } => {
                // Check if any manual items reference the target
                for item_slug in items {
                    if item_slug == target || item_slug.starts_with(&format!("{}/", target)) {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(EtchError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "Target directory '{}' is not configured in Starlight sidebar",
            target
        ),
    )))
}

/// Run comprehensive validation checks on the configuration.
pub fn validate_config(project_dir: &Path) -> EtchResult<Vec<ValidationResult>> {
    let mut results = Vec::new();

    // Check 1: astro.config.mjs exists
    let config_result = check_config_file_exists(project_dir);
    results.push(config_result.clone());

    if !config_result.passed {
        return Ok(results);
    }

    // Check 2: content.config.ts exists
    results.push(check_content_config_exists(project_dir));

    // Check 3: content/docs directory exists
    results.push(check_docs_directory_exists(project_dir));

    // Check 4: Starlight integration is configured
    let config = check_config(project_dir)?;
    results.push(ValidationResult {
        passed: config.starlight.is_some(),
        description: "Starlight integration is configured".to_string(),
        error: if config.starlight.is_none() {
            Some("No Starlight configuration found in astro.config.mjs".to_string())
        } else {
            None
        },
    });

    // Check 5: Sidebar has at least one entry
    if let Some(starlight) = &config.starlight {
        results.push(ValidationResult {
            passed: !starlight.sidebar.is_empty(),
            description: "Sidebar has at least one entry".to_string(),
            error: if starlight.sidebar.is_empty() {
                Some("Sidebar is empty".to_string())
            } else {
                None
            },
        });
    }

    Ok(results)
}

/// Find the Astro config file in a project directory.
fn find_config_file(project_dir: &Path) -> EtchResult<PathBuf> {
    // Check for common config file names
    let candidates = [
        "astro.config.mjs",
        "astro.config.js",
        "astro.config.ts",
        "astro.config.mts",
    ];

    for name in &candidates {
        let path = project_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(EtchError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No astro.config.* file found",
    )))
}

/// Parse site URL from config content.
fn parse_site_url(content: &str) -> Option<String> {
    let re = Regex::new(r#"site:\s*['"]([^'"]+)['"]"#).ok()?;
    re.captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Parse Starlight configuration from config content.
fn parse_starlight_config(content: &str) -> EtchResult<StarlightConfig> {
    let mut config = StarlightConfig::default();

    // Parse title
    let title_re = Regex::new(r#"title:\s*['"]([^'"]+)['"]"#).map_err(|e| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid regex: {}", e),
        ))
    })?;
    if let Some(caps) = title_re.captures(content) {
        if let Some(m) = caps.get(1) {
            config.title = m.as_str().to_string();
        }
    }

    // Parse description
    let desc_re = Regex::new(r#"description:\s*['"]([^'"]+)['"]"#).map_err(|e| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid regex: {}", e),
        ))
    })?;
    if let Some(caps) = desc_re.captures(content) {
        if let Some(m) = caps.get(1) {
            config.description = Some(m.as_str().to_string());
        }
    }

    // Parse sidebar entries
    config.sidebar = parse_sidebar(content)?;

    Ok(config)
}

/// Parse sidebar configuration from config content.
fn parse_sidebar(content: &str) -> EtchResult<Vec<SidebarItem>> {
    let mut items = Vec::new();

    // Match autogenerate entries: autogenerate: { directory: 'api' }
    let auto_re = Regex::new(
        r#"label:\s*['"]([^'"]+)['"][^}]*autogenerate:\s*\{\s*directory:\s*['"]([^'"]+)['"]"#,
    )
    .map_err(|e| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid regex: {}", e),
        ))
    })?;

    for caps in auto_re.captures_iter(content) {
        if let (Some(label), Some(dir)) = (caps.get(1), caps.get(2)) {
            items.push(SidebarItem::Autogenerate {
                label: label.as_str().to_string(),
                directory: dir.as_str().to_string(),
            });
        }
    }

    // Match manual entries: label: 'Getting Started', items: ['getting-started', ...]
    let manual_re =
        Regex::new(r#"label:\s*['"]([^'"]+)['"][^}]*items:\s*\[([^\]]+)\]"#).map_err(|e| {
            EtchError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid regex: {}", e),
            ))
        })?;

    let item_re = Regex::new(r#"['"]([^'"]+)['"]"#).map_err(|e| {
        EtchError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid regex: {}", e),
        ))
    })?;

    for caps in manual_re.captures_iter(content) {
        if let (Some(label), Some(items_str)) = (caps.get(1), caps.get(2)) {
            // Skip if this is actually an autogenerate entry (already captured)
            let label_str = label.as_str();
            if items
                .iter()
                .any(|i| matches!(i, SidebarItem::Autogenerate { label, .. } if label == label_str))
            {
                continue;
            }

            let mut page_items = Vec::new();
            for item_cap in item_re.captures_iter(items_str.as_str()) {
                if let Some(m) = item_cap.get(1) {
                    page_items.push(m.as_str().to_string());
                }
            }

            if !page_items.is_empty() {
                items.push(SidebarItem::Manual {
                    label: label_str.to_string(),
                    items: page_items,
                });
            }
        }
    }

    Ok(items)
}

fn check_config_file_exists(project_dir: &Path) -> ValidationResult {
    let exists = find_config_file(project_dir).is_ok();
    ValidationResult {
        passed: exists,
        description: "astro.config.* file exists".to_string(),
        error: if !exists {
            Some("No Astro configuration file found".to_string())
        } else {
            None
        },
    }
}

fn check_content_config_exists(project_dir: &Path) -> ValidationResult {
    let path = project_dir.join("src/content.config.ts");
    let exists = path.exists();
    ValidationResult {
        passed: exists,
        description: "src/content.config.ts exists".to_string(),
        error: if !exists {
            Some("Content configuration file not found".to_string())
        } else {
            None
        },
    }
}

fn check_docs_directory_exists(project_dir: &Path) -> ValidationResult {
    let path = project_dir.join("src/content/docs");
    let exists = path.exists();
    ValidationResult {
        passed: exists,
        description: "src/content/docs directory exists".to_string(),
        error: if !exists {
            Some("Docs directory not found".to_string())
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("astro.config.mjs");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_parse_site_url() {
        let content = r#"
            export default defineConfig({
                site: 'https://example.com',
            });
        "#;
        assert_eq!(
            parse_site_url(content),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_parse_starlight_config() {
        let content = r#"
            starlight({
                title: 'My Docs',
                description: 'Documentation for my project',
                sidebar: [
                    {
                        label: 'API Reference',
                        autogenerate: { directory: 'api' },
                    },
                ],
            })
        "#;

        let config = parse_starlight_config(content).unwrap();
        assert_eq!(config.title, "My Docs");
        assert_eq!(
            config.description,
            Some("Documentation for my project".to_string())
        );
        assert_eq!(config.sidebar.len(), 1);
    }

    #[test]
    fn test_parse_sidebar_autogenerate() {
        let content = r#"
            sidebar: [
                {
                    label: 'API Reference',
                    autogenerate: { directory: 'api' },
                },
                {
                    label: 'Crates',
                    autogenerate: { directory: 'crates' },
                },
            ]
        "#;

        let sidebar = parse_sidebar(content).unwrap();
        assert_eq!(sidebar.len(), 2);

        match &sidebar[0] {
            SidebarItem::Autogenerate { label, directory } => {
                assert_eq!(label, "API Reference");
                assert_eq!(directory, "api");
            }
            _ => panic!("Expected autogenerate item"),
        }
    }

    #[test]
    fn test_parse_sidebar_manual() {
        let content = r#"
            sidebar: [
                {
                    label: 'Getting Started',
                    items: [
                        'getting-started',
                        'architecture',
                        'roadmap',
                    ],
                },
            ]
        "#;

        let sidebar = parse_sidebar(content).unwrap();
        assert_eq!(sidebar.len(), 1);

        match &sidebar[0] {
            SidebarItem::Manual { label, items } => {
                assert_eq!(label, "Getting Started");
                assert_eq!(items.len(), 3);
                assert!(items.contains(&"getting-started".to_string()));
            }
            _ => panic!("Expected manual item"),
        }
    }

    #[test]
    fn test_check_config() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"
            import { defineConfig } from 'astro/config';
            import starlight from '@astrojs/starlight';

            export default defineConfig({
                site: 'https://example.com',
                integrations: [
                    starlight({
                        title: 'Test Docs',
                        sidebar: [
                            {
                                label: 'API',
                                autogenerate: { directory: 'api' },
                            },
                        ],
                    }),
                ],
            });
        "#;

        create_test_config(temp_dir.path(), content);
        fs::create_dir_all(temp_dir.path().join("src/content/docs")).unwrap();

        let config = check_config(temp_dir.path()).unwrap();
        assert_eq!(config.site_url, Some("https://example.com".to_string()));
        assert!(config.starlight.is_some());

        let starlight = config.starlight.unwrap();
        assert_eq!(starlight.title, "Test Docs");
        assert_eq!(starlight.sidebar.len(), 1);
    }

    #[test]
    fn test_validate_output_dir() {
        let config = AstroConfig {
            starlight: Some(StarlightConfig {
                title: "Test".to_string(),
                description: None,
                sidebar: vec![SidebarItem::Autogenerate {
                    label: "API".to_string(),
                    directory: "api".to_string(),
                }],
            }),
            ..Default::default()
        };

        // Should pass - directory is in autogenerate
        assert!(validate_output_dir(&config, "api").is_ok());

        // Should fail - directory not configured
        assert!(validate_output_dir(&config, "guides").is_err());
    }

    #[test]
    fn test_find_config_file() {
        let temp_dir = TempDir::new().unwrap();

        // No config file
        assert!(find_config_file(temp_dir.path()).is_err());

        // Create mjs file
        fs::write(temp_dir.path().join("astro.config.mjs"), "").unwrap();
        let result = find_config_file(temp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("astro.config.mjs"));
    }
}
