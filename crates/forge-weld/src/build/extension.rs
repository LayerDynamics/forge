//! ExtensionBuilder for simplified build.rs scripts
//!
//! This module provides a high-level API for building Forge extensions
//! from build.rs scripts with minimal boilerplate.

use crate::build::transpile::{transpile_file, TranspileError};
use crate::codegen::{DtsGenerator, ExtensionGenerator};
use crate::ir::WeldModule;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during extension building
#[derive(Debug, Error)]
pub enum ExtensionBuilderError {
    /// Environment variable not set
    #[error("Environment variable not set: {0}")]
    EnvVarMissing(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Transpilation error
    #[error("Transpilation error: {0}")]
    TranspileError(#[from] TranspileError),

    /// TypeScript source not found
    #[error("TypeScript source not found: {0}")]
    TsNotFound(PathBuf),

    /// Module validation error
    #[error("Module validation error: {0}")]
    ValidationError(String),
}

/// Builder for Forge extension crates
///
/// Simplifies build.rs scripts by handling:
/// - TypeScript transpilation
/// - extension.rs generation
/// - .d.ts generation for SDK
/// - cargo:rerun-if-changed directives
///
/// # Example
/// ```ignore
/// use forge_weld::build::ExtensionBuilder;
///
/// fn main() {
///     ExtensionBuilder::new("host_fs", "host:fs")
///         .ts_path("ts/init.ts")
///         .ops(&[
///             "op_fs_read_text",
///             "op_fs_write_text",
///         ])
///         .generate_sdk_types("../../sdk")
///         .build()
///         .expect("Failed to build extension");
/// }
/// ```
pub struct ExtensionBuilder {
    module: WeldModule,
    ts_path: Option<PathBuf>,
    sdk_path: Option<PathBuf>,
    additional_watch: Vec<PathBuf>,
    dts_generator: Option<Box<dyn Fn() -> String>>,
}

impl ExtensionBuilder {
    /// Create a new extension builder
    ///
    /// # Arguments
    /// * `name` - Internal module name (e.g., "host_fs")
    /// * `specifier` - Import specifier (e.g., "host:fs")
    pub fn new(name: impl Into<String>, specifier: impl Into<String>) -> Self {
        Self {
            module: WeldModule::new(name, specifier),
            ts_path: None,
            sdk_path: None,
            additional_watch: Vec::new(),
            dts_generator: None,
        }
    }

    /// Create a new extension builder for a host module
    ///
    /// # Arguments
    /// * `name` - Module name without "host_" prefix (e.g., "fs" -> "host_fs", "host:fs")
    pub fn host(name: &str) -> Self {
        Self {
            module: WeldModule::host(name),
            ts_path: None,
            sdk_path: None,
            additional_watch: Vec::new(),
            dts_generator: None,
        }
    }

    /// Set the TypeScript source path (relative to crate root)
    pub fn ts_path(mut self, path: impl AsRef<Path>) -> Self {
        self.ts_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the ops for this extension
    pub fn ops(mut self, ops: &[&str]) -> Self {
        for op_name in ops {
            self.module = self
                .module
                .op(crate::ir::OpSymbol::from_rust_name(*op_name));
        }
        self
    }

    /// Set the WeldModule directly (for more control)
    pub fn module(mut self, module: WeldModule) -> Self {
        self.module = module;
        self
    }

    /// Enable SDK type generation
    pub fn generate_sdk_types(mut self, sdk_relative_path: impl AsRef<Path>) -> Self {
        self.sdk_path = Some(sdk_relative_path.as_ref().to_path_buf());
        self
    }

    /// Add additional files to watch for rebuilds
    pub fn watch(mut self, path: impl AsRef<Path>) -> Self {
        self.additional_watch.push(path.as_ref().to_path_buf());
        self
    }

    /// Set a custom .d.ts generator function
    pub fn dts_generator<F>(mut self, generator: F) -> Self
    where
        F: Fn() -> String + 'static,
    {
        self.dts_generator = Some(Box::new(generator));
        self
    }

    /// Set module documentation
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.module = self.module.with_doc(doc);
        self
    }

    /// Build the extension
    ///
    /// This will:
    /// 1. Transpile the TypeScript source
    /// 2. Generate the extension.rs file
    /// 3. Optionally generate .d.ts for SDK
    /// 4. Print cargo:rerun-if-changed directives
    pub fn build(self) -> Result<(), ExtensionBuilderError> {
        let out_dir = env::var("OUT_DIR")
            .map_err(|_| ExtensionBuilderError::EnvVarMissing("OUT_DIR".to_string()))?;
        let out_path = Path::new(&out_dir);

        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| ExtensionBuilderError::EnvVarMissing("CARGO_MANIFEST_DIR".to_string()))?;
        let manifest_path = Path::new(&manifest_dir);

        // Transpile TypeScript if path is set
        let js_code = if let Some(ref ts_path) = self.ts_path {
            let full_ts_path = manifest_path.join(ts_path);

            if !full_ts_path.exists() {
                return Err(ExtensionBuilderError::TsNotFound(full_ts_path));
            }

            // Print rerun directive
            println!("cargo:rerun-if-changed={}", ts_path.display());

            transpile_file(&full_ts_path)?
        } else {
            // No TypeScript, generate minimal JS
            "// No TypeScript source provided\n".to_string()
        };

        // Generate extension.rs
        let extension_gen = ExtensionGenerator::new(&self.module);
        let extension_rs = extension_gen.generate(&js_code);
        fs::write(out_path.join("extension.rs"), extension_rs)?;

        // Generate .d.ts for SDK if requested
        if let Some(ref sdk_relative_path) = self.sdk_path {
            let workspace_root = manifest_path.parent().unwrap().parent().unwrap();
            let sdk_dir = workspace_root.join(sdk_relative_path);
            let generated_dir = sdk_dir.join("generated");

            // Create generated directory if it doesn't exist
            fs::create_dir_all(&generated_dir)?;

            // Generate .d.ts content
            let dts_content = if let Some(ref generator) = self.dts_generator {
                generator()
            } else {
                let dts_gen = DtsGenerator::new(&self.module);
                dts_gen.generate()
            };

            // Write to SDK generated directory
            let dts_filename = format!("{}.d.ts", self.module.specifier.replace(':', "."));
            let dts_path = generated_dir.join(&dts_filename);
            fs::write(&dts_path, &dts_content)?;

            // Also write to OUT_DIR for reference
            let out_dts_path = out_path.join(&dts_filename);
            fs::write(&out_dts_path, &dts_content)?;
        }

        // Print additional watch directives
        for watch_path in &self.additional_watch {
            println!("cargo:rerun-if-changed={}", watch_path.display());
        }

        // Always watch lib.rs for op changes
        println!("cargo:rerun-if-changed=src/lib.rs");

        Ok(())
    }

    /// Build and return the generated paths (for testing)
    pub fn build_returning_paths(self) -> Result<BuildOutput, ExtensionBuilderError> {
        let out_dir = env::var("OUT_DIR")
            .map_err(|_| ExtensionBuilderError::EnvVarMissing("OUT_DIR".to_string()))?;

        let dts_filename = format!("{}.d.ts", self.module.specifier.replace(':', "."));

        let has_sdk_path = self.sdk_path.is_some();

        self.build()?;

        Ok(BuildOutput {
            extension_rs: PathBuf::from(&out_dir).join("extension.rs"),
            dts_path: if has_sdk_path {
                Some(PathBuf::from(&out_dir).join(dts_filename))
            } else {
                None
            },
        })
    }
}

/// Output paths from a successful build
#[derive(Debug)]
pub struct BuildOutput {
    /// Path to generated extension.rs
    pub extension_rs: PathBuf,
    /// Path to generated .d.ts (if SDK generation was enabled)
    pub dts_path: Option<PathBuf>,
}

/// Shorthand for creating a host extension builder
///
/// # Example
/// ```ignore
/// use forge_weld::build::host_extension;
///
/// fn main() {
///     host_extension("fs")
///         .ts_path("ts/init.ts")
///         .ops(&["op_fs_read_text"])
///         .build()
///         .unwrap();
/// }
/// ```
pub fn host_extension(name: &str) -> ExtensionBuilder {
    ExtensionBuilder::host(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_builder_creation() {
        let builder = ExtensionBuilder::new("host_fs", "host:fs")
            .ts_path("ts/init.ts")
            .ops(&["op_fs_read_text", "op_fs_write_text"]);

        assert_eq!(builder.module.name, "host_fs");
        assert_eq!(builder.module.specifier, "host:fs");
        assert_eq!(builder.module.ops.len(), 2);
    }

    #[test]
    fn test_host_extension() {
        let builder = ExtensionBuilder::host("net").ops(&["op_net_fetch"]);

        assert_eq!(builder.module.name, "host_net");
        assert_eq!(builder.module.specifier, "host:net");
    }
}
