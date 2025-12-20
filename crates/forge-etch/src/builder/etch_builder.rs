//! EtchBuilder - Main API for documentation generation
//!
//! This module provides the builder pattern API for configuring and
//! running documentation generation from build.rs scripts.

use crate::diagnostics::{EtchError, EtchResult};
use std::path::{Path, PathBuf};

/// Output format for documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OutputFormat {
    /// Astro-compatible markdown (.md)
    #[default]
    Astro,
    /// Standalone HTML
    Html,
    /// Both formats
    Both,
}

/// Build output containing generated documentation
#[derive(Debug)]
pub struct BuildOutput {
    /// Generated Astro markdown files
    pub astro_files: Vec<PathBuf>,
    /// Generated HTML files
    pub html_files: Vec<PathBuf>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Number of symbols documented
    pub symbol_count: usize,
}

impl BuildOutput {
    /// Create a new build output
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            astro_files: vec![],
            html_files: vec![],
            output_dir: output_dir.into(),
            symbol_count: 0,
        }
    }

    /// Get all generated files
    pub fn all_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.astro_files.iter().chain(self.html_files.iter())
    }
}

/// Builder for configuring documentation generation
///
/// # Example
///
/// ```no_run
/// use forge_etch::EtchBuilder;
///
/// EtchBuilder::new("host_fs", "runtime:fs")
///     .rust_source("src/lib.rs")
///     .ts_source("ts/init.ts")
///     .output_dir("docs")
///     .generate_astro(true)
///     .build()
///     .expect("Failed to generate docs");
/// ```
#[derive(Debug)]
pub struct EtchBuilder {
    /// Extension name (e.g., "host_fs")
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub module_specifier: String,
    /// Rust source file path
    pub rust_source: Option<PathBuf>,
    /// TypeScript source file path
    pub ts_source: Option<PathBuf>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Whether to generate Astro markdown
    pub generate_astro: bool,
    /// Whether to generate HTML
    pub generate_html: bool,
    /// Title for documentation
    pub title: Option<String>,
    /// Description for documentation
    pub description: Option<String>,
    /// Additional source directories to scan
    pub source_dirs: Vec<PathBuf>,
    /// Whether to include private symbols
    pub include_private: bool,
    /// Whether to include internal symbols
    pub include_internal: bool,
}

impl EtchBuilder {
    /// Create a new builder for an extension
    ///
    /// # Arguments
    ///
    /// * `name` - The extension name (e.g., "host_fs")
    /// * `module_specifier` - The module specifier (e.g., "runtime:fs")
    pub fn new(name: impl Into<String>, module_specifier: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            module_specifier: module_specifier.into(),
            rust_source: None,
            ts_source: None,
            output_dir: PathBuf::from("docs"),
            generate_astro: true,
            generate_html: false,
            title: None,
            description: None,
            source_dirs: vec![],
            include_private: false,
            include_internal: false,
        }
    }

    /// Set the Rust source file
    pub fn rust_source(mut self, path: impl Into<PathBuf>) -> Self {
        self.rust_source = Some(path.into());
        self
    }

    /// Set the TypeScript source file
    pub fn ts_source(mut self, path: impl Into<PathBuf>) -> Self {
        self.ts_source = Some(path.into());
        self
    }

    /// Set the output directory
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = path.into();
        self
    }

    /// Enable or disable Astro markdown generation
    pub fn generate_astro(mut self, enable: bool) -> Self {
        self.generate_astro = enable;
        self
    }

    /// Enable or disable HTML generation
    pub fn generate_html(mut self, enable: bool) -> Self {
        self.generate_html = enable;
        self
    }

    /// Set the documentation title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the documentation description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a source directory to scan
    pub fn add_source_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_dirs.push(path.into());
        self
    }

    /// Include private symbols in documentation
    pub fn include_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Include internal symbols in documentation
    pub fn include_internal(mut self, include: bool) -> Self {
        self.include_internal = include;
        self
    }

    /// Build the documentation
    ///
    /// This method:
    /// 1. Parses TypeScript sources using SWC
    /// 2. Extracts forge-weld metadata from Rust sources
    /// 3. Merges documentation from both sources
    /// 4. Generates output in the configured formats
    pub fn build(self) -> EtchResult<BuildOutput> {
        // Validate configuration
        if !self.generate_astro && !self.generate_html {
            return Err(EtchError::config(
                "At least one output format must be enabled",
            ));
        }

        // Create output directory
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir)?;
        }

        // Create the etcher and run
        let config = crate::docgen::EtchConfig {
            name: self.name.clone(),
            module_specifier: self.module_specifier.clone(),
            rust_source: self.rust_source.clone(),
            ts_source: self.ts_source.clone(),
            output_dir: self.output_dir.clone(),
            generate_astro: self.generate_astro,
            generate_html: self.generate_html,
            title: self.title.clone(),
            description: self.description.clone(),
            include_private: self.include_private,
            include_internal: self.include_internal,
        };

        let mut etcher = crate::docgen::Etcher::new(config);
        etcher.run()
    }

    /// Build documentation from a crate root
    ///
    /// Automatically discovers ts/init.ts and src/lib.rs
    pub fn from_crate_root(
        name: impl Into<String>,
        module_specifier: impl Into<String>,
        crate_root: impl AsRef<Path>,
    ) -> Self {
        let root = crate_root.as_ref();
        let mut builder = Self::new(name, module_specifier);

        // Check for ts/init.ts
        let ts_path = root.join("ts/init.ts");
        if ts_path.exists() {
            builder = builder.ts_source(ts_path);
        }

        // Check for src/lib.rs
        let rust_path = root.join("src/lib.rs");
        if rust_path.exists() {
            builder = builder.rust_source(rust_path);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = EtchBuilder::new("host_fs", "runtime:fs");
        assert_eq!(builder.name, "host_fs");
        assert_eq!(builder.module_specifier, "runtime:fs");
        assert!(builder.generate_astro);
        assert!(!builder.generate_html);
    }

    #[test]
    fn test_builder_configuration() {
        let builder = EtchBuilder::new("host_fs", "runtime:fs")
            .rust_source("src/lib.rs")
            .ts_source("ts/init.ts")
            .output_dir("generated/docs")
            .generate_html(true)
            .title("File System API");

        assert_eq!(builder.rust_source, Some(PathBuf::from("src/lib.rs")));
        assert_eq!(builder.ts_source, Some(PathBuf::from("ts/init.ts")));
        assert_eq!(builder.output_dir, PathBuf::from("generated/docs"));
        assert!(builder.generate_html);
        assert_eq!(builder.title, Some("File System API".to_string()));
    }
}
