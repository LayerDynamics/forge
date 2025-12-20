//! Source detection utilities
//!
//! This module provides utilities for detecting and categorizing
//! source files in a Forge extension.

use std::path::PathBuf;
use walkdir::WalkDir;

/// Source detector for finding documentation sources
pub struct SourceDetector {
    /// Root directory to search
    root: PathBuf,
}

impl SourceDetector {
    /// Create a new detector for a directory
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Find all TypeScript source files
    pub fn find_typescript_files(&self) -> Vec<PathBuf> {
        self.find_files_by_extension(&["ts", "tsx", "mts", "cts"])
    }

    /// Find all Rust source files
    pub fn find_rust_files(&self) -> Vec<PathBuf> {
        self.find_files_by_extension(&["rs"])
    }

    /// Find the main TypeScript entry point (ts/init.ts)
    pub fn find_ts_entry(&self) -> Option<PathBuf> {
        let candidates = [
            self.root.join("ts/init.ts"),
            self.root.join("ts/mod.ts"),
            self.root.join("ts/index.ts"),
            self.root.join("src/init.ts"),
            self.root.join("src/mod.ts"),
            self.root.join("src/index.ts"),
        ];

        candidates.into_iter().find(|candidate| candidate.exists())
    }

    /// Find the main Rust entry point (src/lib.rs)
    pub fn find_rust_entry(&self) -> Option<PathBuf> {
        let candidates = [
            self.root.join("src/lib.rs"),
            self.root.join("src/main.rs"),
            self.root.join("lib.rs"),
        ];

        candidates.into_iter().find(|candidate| candidate.exists())
    }

    /// Find Cargo.toml
    pub fn find_cargo_toml(&self) -> Option<PathBuf> {
        let path = self.root.join("Cargo.toml");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Find manifest.app.toml (for Forge apps)
    pub fn find_manifest(&self) -> Option<PathBuf> {
        let path = self.root.join("manifest.app.toml");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Detect the project type
    pub fn detect_project_type(&self) -> ProjectType {
        if self.find_cargo_toml().is_some() {
            if self.root.to_string_lossy().contains("ext_") {
                ProjectType::Extension
            } else {
                ProjectType::RustCrate
            }
        } else if self.find_manifest().is_some() {
            ProjectType::ForgeApp
        } else if self.find_ts_entry().is_some() {
            ProjectType::TypeScriptModule
        } else {
            ProjectType::Unknown
        }
    }

    /// Find files by extension
    fn find_files_by_extension(&self, extensions: &[&str]) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        files.sort();
        files
    }

    /// Get the crate name from Cargo.toml
    pub fn get_crate_name(&self) -> Option<String> {
        let cargo_toml = self.find_cargo_toml()?;
        let content = std::fs::read_to_string(cargo_toml).ok()?;
        let parsed: toml::Value = content.parse().ok()?;

        parsed
            .get("package")?
            .get("name")?
            .as_str()
            .map(|s| s.to_string())
    }
}

/// Type of project detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// A Forge extension crate (ext_*)
    Extension,
    /// A generic Rust crate
    RustCrate,
    /// A Forge application
    ForgeApp,
    /// A TypeScript module
    TypeScriptModule,
    /// Unknown project type
    Unknown,
}

impl ProjectType {
    /// Get display name
    pub fn display(&self) -> &'static str {
        match self {
            ProjectType::Extension => "Forge Extension",
            ProjectType::RustCrate => "Rust Crate",
            ProjectType::ForgeApp => "Forge App",
            ProjectType::TypeScriptModule => "TypeScript Module",
            ProjectType::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Extension.display(), "Forge Extension");
        assert_eq!(ProjectType::ForgeApp.display(), "Forge App");
    }

    #[test]
    fn test_source_detector() {
        let temp = TempDir::new().unwrap();
        let detector = SourceDetector::new(temp.path());

        // Empty directory should have no files
        assert!(detector.find_typescript_files().is_empty());
        assert!(detector.find_rust_files().is_empty());
        assert_eq!(detector.detect_project_type(), ProjectType::Unknown);
    }
}
