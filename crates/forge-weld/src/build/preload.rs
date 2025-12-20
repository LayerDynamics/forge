//! Preload script builder for Forge applications
//!
//! This module provides build-time utilities for generating the preload.ts
//! script from extension metadata.

use crate::codegen::PreloadGenerator;
use crate::ir::WeldModule;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during preload building
#[derive(Debug, Error)]
pub enum PreloadBuilderError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// No modules found
    #[error("No extension modules found")]
    NoModulesFound,

    /// Output path not set
    #[error("Output path not set")]
    OutputPathNotSet,
}

/// Builder for generating preload.ts scripts
///
/// This builder collects extension modules and generates a preload.ts
/// script that includes:
/// - Host bridge implementation
/// - Extension metadata (derived from modules)
/// - HMR client for development
///
/// # Example
/// ```no_run
/// use forge_weld::PreloadBuilder;
///
/// PreloadBuilder::new()
///     .output_path("sdk/preload.ts")
///     .discover_modules("target/debug/build")
///     .build()
///     .expect("Failed to build preload");
/// ```
pub struct PreloadBuilder {
    modules: Vec<WeldModule>,
    output_path: Option<PathBuf>,
    enable_hmr: bool,
}

impl PreloadBuilder {
    /// Create a new preload builder
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            output_path: None,
            enable_hmr: true,
        }
    }

    /// Set the output path for the generated preload.ts
    pub fn output_path(mut self, path: impl AsRef<Path>) -> Self {
        self.output_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Add a module to include in the preload
    pub fn add_module(mut self, module: WeldModule) -> Self {
        self.modules.push(module);
        self
    }

    /// Add multiple modules
    pub fn add_modules(mut self, modules: Vec<WeldModule>) -> Self {
        self.modules.extend(modules);
        self
    }

    /// Discover modules from extension JSON files in a build directory
    ///
    /// This scans the given directory for `module.json` files generated
    /// by ExtensionBuilder and loads them.
    pub fn discover_modules(mut self, build_dir: impl AsRef<Path>) -> Self {
        if let Ok(modules) = discover_modules_from_build_dir(build_dir.as_ref()) {
            self.modules.extend(modules);
        }
        self
    }

    /// Load a module from a JSON file
    pub fn load_module(mut self, json_path: impl AsRef<Path>) -> Result<Self, PreloadBuilderError> {
        let content = fs::read_to_string(json_path)?;
        let module: WeldModule = serde_json::from_str(&content)?;
        self.modules.push(module);
        Ok(self)
    }

    /// Disable HMR client in the generated preload
    pub fn disable_hmr(mut self) -> Self {
        self.enable_hmr = false;
        self
    }

    /// Build the preload.ts file
    pub fn build(self) -> Result<(), PreloadBuilderError> {
        let output_path = self
            .output_path
            .ok_or(PreloadBuilderError::OutputPathNotSet)?;

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate preload content
        let mut generator = PreloadGenerator::with_modules(self.modules);
        if !self.enable_hmr {
            generator = generator.disable_hmr();
        }
        let preload_ts = generator.generate();

        // Write to file
        fs::write(&output_path, preload_ts)?;

        Ok(())
    }

    /// Build and return the generated content (for testing)
    pub fn build_to_string(self) -> String {
        let mut generator = PreloadGenerator::with_modules(self.modules);
        if !self.enable_hmr {
            generator = generator.disable_hmr();
        }
        generator.generate()
    }
}

impl Default for PreloadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover WeldModule JSON files from a cargo build directory
///
/// Scans `target/{profile}/build/*/out/docs/module.json` for module metadata.
fn discover_modules_from_build_dir(
    build_dir: &Path,
) -> Result<Vec<WeldModule>, PreloadBuilderError> {
    let mut modules = Vec::new();

    // Walk through build output directories
    if let Ok(entries) = fs::read_dir(build_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Check for out/docs/module.json
                let module_json = path.join("out").join("docs").join("module.json");
                if module_json.exists() {
                    if let Ok(content) = fs::read_to_string(&module_json) {
                        if let Ok(module) = serde_json::from_str::<WeldModule>(&content) {
                            modules.push(module);
                        }
                    }
                }
            }
        }
    }

    Ok(modules)
}

/// Generate a preload script from a list of modules
///
/// This is a convenience function for simple use cases.
pub fn generate_preload(modules: Vec<WeldModule>) -> String {
    PreloadGenerator::with_modules(modules).generate()
}

/// Generate a preload script and write to file
pub fn generate_preload_to_file(
    modules: Vec<WeldModule>,
    output_path: impl AsRef<Path>,
) -> Result<(), PreloadBuilderError> {
    PreloadBuilder::new()
        .add_modules(modules)
        .output_path(output_path)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_builder() {
        let module = WeldModule::host("test").with_extensibility();
        let builder = PreloadBuilder::new().add_module(module);

        let output = builder.build_to_string();
        assert!(output.contains("globalThis.host"));
        assert!(output.contains("_extensionMeta"));
    }

    #[test]
    fn test_preload_builder_no_hmr() {
        let builder = PreloadBuilder::new().disable_hmr();
        let output = builder.build_to_string();

        assert!(!output.contains("connectHMR"));
    }
}
