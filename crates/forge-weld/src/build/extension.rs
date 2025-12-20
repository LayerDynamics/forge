//! ExtensionBuilder for simplified build.rs scripts
//!
//! This module provides a high-level API for building Forge extensions
//! from build.rs scripts with minimal boilerplate.

use crate::build::transpile::TranspileError;
use crate::codegen::{DtsGenerator, ExtensionGenerator, TypeScriptGenerator};
use crate::ir::{SymbolRegistry, WeldModule};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Documentation output format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DocFormat {
    /// Generate Astro-compatible markdown (default)
    #[default]
    Astro,
    /// Generate standalone HTML documentation
    Html,
    /// Generate both Astro and HTML
    Both,
}

/// Documentation configuration for extension builder
#[derive(Debug, Clone)]
pub struct DocConfig {
    /// Output directory for generated documentation
    pub output_dir: PathBuf,
    /// Documentation format
    pub format: DocFormat,
    /// Optional documentation title
    pub title: Option<String>,
    /// Optional documentation description
    pub description: Option<String>,
}

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

    /// Documentation generation error
    #[error("Documentation generation error: {0}")]
    DocGeneration(String),
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
/// ```no_run
/// use forge_weld::ExtensionBuilder;
///
/// ExtensionBuilder::new("host_fs", "runtime:fs")
///     .ts_path("ts/init.ts")
///     .ops(&[
///         "op_fs_read_text",
///         "op_fs_write_text",
///     ])
///     .generate_sdk_types("../../sdk")
///     .build()
///     .expect("Failed to build extension");
/// ```
pub struct ExtensionBuilder {
    module: WeldModule,
    ts_path: Option<PathBuf>,
    sdk_path: Option<PathBuf>,
    sdk_module_path: Option<PathBuf>,
    use_inventory_types: bool,
    additional_watch: Vec<PathBuf>,
    dts_generator: Option<Box<dyn Fn() -> String>>,
    /// Documentation generation configuration
    doc_config: Option<DocConfig>,
}

impl ExtensionBuilder {
    /// Create a new extension builder
    ///
    /// # Arguments
    /// * `name` - Internal module name (e.g., "host_fs")
    /// * `specifier` - Import specifier (e.g., "runtime:fs")
    pub fn new(name: impl Into<String>, specifier: impl Into<String>) -> Self {
        Self {
            module: WeldModule::new(name, specifier),
            ts_path: None,
            sdk_path: None,
            sdk_module_path: None,
            use_inventory_types: false,
            additional_watch: Vec::new(),
            dts_generator: None,
            doc_config: None,
        }
    }

    /// Create a new extension builder for a host module
    ///
    /// # Arguments
    /// * `name` - Module name without "host_" prefix (e.g., "fs" -> "host_fs", "runtime:fs")
    pub fn host(name: &str) -> Self {
        Self {
            module: WeldModule::host(name),
            ts_path: None,
            sdk_path: None,
            sdk_module_path: None,
            use_inventory_types: false,
            additional_watch: Vec::new(),
            dts_generator: None,
            doc_config: None,
        }
    }

    /// Set the TypeScript source path (relative to crate root)
    ///
    /// The TypeScript file will be transpiled to JavaScript during the build
    /// and included in the extension.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")  // Located at crates/ext_fs/ts/init.ts
    ///     .build();
    /// ```
    pub fn ts_path(mut self, path: impl AsRef<Path>) -> Self {
        self.ts_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the ops for this extension
    ///
    /// Registers op functions to be included in the generated TypeScript SDK.
    /// Op names must match the actual Rust function names.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ops(&[
    ///         "op_fs_read_text",   // Becomes readText() in TS
    ///         "op_fs_write_text",  // Becomes writeText() in TS
    ///         "op_fs_stat",        // Becomes stat() in TS
    ///     ])
    ///     .build();
    /// ```
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

    /// Enable SDK module generation (full .ts implementation)
    ///
    /// Generates a complete TypeScript SDK file at the specified path.
    /// This differs from `generate_sdk_types()` which only generates type definitions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .ops(&["op_fs_read_text"])
    ///     .generate_sdk_module("../../sdk")  // Creates sdk/runtime.fs.ts
    ///     .build();
    /// ```
    pub fn generate_sdk_module(mut self, sdk_relative_path: impl AsRef<Path>) -> Self {
        self.sdk_module_path = Some(sdk_relative_path.as_ref().to_path_buf());
        self
    }

    /// Merge inventory-collected type metadata (from #[weld_op]/#[weld_struct]/#[weld_enum])
    /// into the module before codegen.
    ///
    /// This is required if your extension uses the `#[weld_struct]` or `#[weld_enum]`
    /// macros to generate TypeScript types from Rust types.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// // Your extension defines:
    /// // #[weld_struct]
    /// // pub struct FileStat { ... }
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .use_inventory_types()  // Include FileStat in generated types
    ///     .build();
    /// ```
    pub fn use_inventory_types(mut self) -> Self {
        self.use_inventory_types = true;
        self
    }

    /// Add additional files to watch for rebuilds
    ///
    /// Cargo will rerun the build script when these files change.
    /// Useful for watching configuration files or additional TypeScript sources.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .watch("ts/types.ts")       // Also watch types file
    ///     .watch("fs-config.toml")    // Watch config file
    ///     .build();
    /// ```
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

    /// Enable documentation generation
    ///
    /// When enabled, the builder will generate API documentation alongside
    /// the extension code during the build process.
    ///
    /// # Arguments
    /// * `output_dir` - Directory where documentation will be generated
    ///
    /// # Example
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("host_fs", "runtime:fs")
    ///     .generate_docs("../../site/src/content/docs/api/fs")
    ///     .build();
    /// ```
    pub fn generate_docs(mut self, output_dir: impl AsRef<Path>) -> Self {
        self.doc_config = Some(DocConfig {
            output_dir: output_dir.as_ref().to_path_buf(),
            format: DocFormat::default(),
            title: None,
            description: None,
        });
        self
    }

    /// Set documentation output format
    ///
    /// Only has effect if `generate_docs()` was called first.
    pub fn doc_format(mut self, format: DocFormat) -> Self {
        if let Some(ref mut config) = self.doc_config {
            config.format = format;
        }
        self
    }

    /// Set documentation title
    ///
    /// Only has effect if `generate_docs()` was called first.
    pub fn doc_title(mut self, title: impl Into<String>) -> Self {
        if let Some(ref mut config) = self.doc_config {
            config.title = Some(title.into());
        }
        self
    }

    /// Set documentation description
    ///
    /// Only has effect if `generate_docs()` was called first.
    pub fn doc_description(mut self, description: impl Into<String>) -> Self {
        if let Some(ref mut config) = self.doc_config {
            config.description = Some(description.into());
        }
        self
    }

    // =========================================================================
    // Extensibility Configuration
    // =========================================================================

    /// Enable full extensibility (hooks + handlers)
    ///
    /// This enables both lifecycle hooks and custom handler registration
    /// for all ops in this extension.
    ///
    /// # Example
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .ops(&["op_fs_read_text", "op_fs_write_text"])
    ///     .enable_extensibility()
    ///     .build();
    /// ```
    pub fn enable_extensibility(mut self) -> Self {
        self.module.extensibility.hooks_enabled = true;
        self.module.extensibility.handlers_enabled = true;
        self
    }

    /// Enable lifecycle hooks only
    ///
    /// Generates onBefore, onAfter, and onError hook registration functions
    /// that allow users to intercept and modify op calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .ops(&["op_fs_read_text"])
    ///     .enable_hooks()  // Users can now do: fs.onBefore("readText", ...)
    ///     .build();
    /// ```
    ///
    /// Generated TypeScript API:
    /// ```typescript
    /// // Users can intercept ops:
    /// fs.onBefore("readText", (path) => {
    ///     console.log("Reading:", path);
    ///     return path; // Can modify arguments
    /// });
    /// ```
    pub fn enable_hooks(mut self) -> Self {
        self.module.extensibility.hooks_enabled = true;
        self
    }

    /// Enable custom handler registration only
    ///
    /// Generates registerHandler, invokeHandler, listHandlers functions
    /// for plugin-style custom functionality.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .enable_handlers()  // Users can register custom handlers
    ///     .build();
    /// ```
    ///
    /// Generated TypeScript API:
    /// ```typescript
    /// // Users can register handlers:
    /// fs.registerHandler("custom-transform", async (data) => {
    ///     return transformData(data);
    /// });
    ///
    /// // And invoke them:
    /// const result = await fs.invokeHandler("custom-transform", input);
    /// ```
    pub fn enable_handlers(mut self) -> Self {
        self.module.extensibility.handlers_enabled = true;
        self
    }

    /// Specify which ops support hooks (default: all ops)
    ///
    /// If not called, all ops are hookable.
    /// If called with an empty slice, no ops are hookable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ops(&["op_fs_read_text", "op_fs_write_text", "op_fs_stat"])
    ///     .enable_hooks()
    ///     .hookable_ops(&["op_fs_read_text", "op_fs_write_text"])
    ///     // Only read/write are hookable, not stat
    ///     .build();
    /// ```
    pub fn hookable_ops(mut self, ops: &[&str]) -> Self {
        self.module.extensibility.hookable_ops = ops.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a configuration option for the extension
    ///
    /// Configuration options allow users to customize extension behavior
    /// through the `extend(config)` API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::{ExtensionBuilder, WeldType};
    ///
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .config_option("maxFileSize", WeldType::number(), Some("1048576"))
    ///     .config_option("encoding", WeldType::string(), Some("\"utf-8\""))
    ///     .build();
    /// ```
    ///
    /// Generated TypeScript API:
    /// ```typescript
    /// // Users can configure the extension:
    /// import * as fs from "runtime:fs";
    ///
    /// fs.extend({
    ///     maxFileSize: 5_000_000,  // 5MB
    ///     encoding: "latin1",
    /// });
    /// ```
    pub fn config_option(
        mut self,
        name: &str,
        ts_type: crate::ir::WeldType,
        default: Option<&str>,
    ) -> Self {
        self.module.extensibility.config_options.push(
            crate::ir::ConfigOption::new(name, ts_type)
                .with_default(default.unwrap_or("undefined")),
        );
        self
    }

    /// Build the extension
    ///
    /// This will:
    /// 1. Transpile the TypeScript source
    /// 2. Generate the extension.rs file
    /// 3. Optionally generate .d.ts for SDK
    /// 4. Print cargo:rerun-if-changed directives
    ///
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// - [`ExtensionBuilderError::EnvVarMissing`]
    ///   - When `OUT_DIR` or `CARGO_MANIFEST_DIR` not set (non-cargo build)
    ///   - **Recovery:** Only call from build.rs scripts run by cargo
    ///
    /// - [`ExtensionBuilderError::TsNotFound`]
    ///   - When TypeScript source file doesn't exist at specified path
    ///   - Example: `ts_path("ts/init.ts")` but file is missing
    ///   - **Recovery:** Verify file path is correct relative to crate root
    ///
    /// - [`ExtensionBuilderError::TranspileError`]
    ///   - When Deno fails to transpile TypeScript to JavaScript
    ///   - Causes: Syntax errors, type errors (if strict mode), import resolution
    ///   - **Recovery:** Check TypeScript source for errors, verify imports
    ///
    /// - [`ExtensionBuilderError::IoError`]
    ///   - When file system operations fail (read/write)
    ///   - Causes: Permission denied, disk full, path too long
    ///   - **Recovery:** Check file permissions and disk space
    ///
    /// - [`ExtensionBuilderError::ValidationError`]
    ///   - When module validation fails (duplicate ops, invalid names)
    ///   - Example: Two ops with same name
    ///   - **Recovery:** Fix op name conflicts in module definition
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use forge_weld::ExtensionBuilder;
    ///
    /// // Typical build.rs usage
    /// ExtensionBuilder::new("runtime_fs", "runtime:fs")
    ///     .ts_path("ts/init.ts")
    ///     .ops(&["op_fs_read_text", "op_fs_write_text"])
    ///     .generate_sdk_module("../../sdk")
    ///     .use_inventory_types()
    ///     .build()
    ///     .expect("Failed to build extension");
    /// ```
    ///
    /// # Error Handling
    ///
    /// Build errors should fail the build immediately. They indicate structural
    /// problems that must be fixed before the extension can compile:
    ///
    /// ```no_run
    /// # use forge_weld::ExtensionBuilder;
    /// # let builder = ExtensionBuilder::new("test", "test:test");
    /// if let Err(e) = builder.build() {
    ///     eprintln!("Extension build failed: {}", e);
    ///     std::process::exit(1);
    /// }
    /// ```
    pub fn build(self) -> Result<(), ExtensionBuilderError> {
        let out_dir = env::var("OUT_DIR")
            .map_err(|_| ExtensionBuilderError::EnvVarMissing("OUT_DIR".to_string()))?;
        let out_path = Path::new(&out_dir);

        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| ExtensionBuilderError::EnvVarMissing("CARGO_MANIFEST_DIR".to_string()))?;
        let manifest_path = Path::new(&manifest_dir);

        // Start with the configured module and optionally enrich it with inventory types.
        let mut module = self.module;
        if self.use_inventory_types {
            let registry = SymbolRegistry::from_inventory();
            let typed_structs = registry.structs().to_vec();
            let typed_enums = registry.enums().to_vec();
            let typed_ops = registry.ops().to_vec();

            module.structs = typed_structs;
            module.enums = typed_enums;
            if module.ops.is_empty() {
                module.ops = typed_ops;
            } else {
                for op in &mut module.ops {
                    if let Some(typed) = typed_ops.iter().find(|t| t.rust_name == op.rust_name) {
                        *op = typed.clone();
                    }
                }
            }
        }

        // Transpile TypeScript if path is set
        let js_code = if let Some(ref ts_path) = self.ts_path {
            let full_ts_path = manifest_path.join(ts_path);

            if !full_ts_path.exists() {
                return Err(ExtensionBuilderError::TsNotFound(full_ts_path));
            }

            // Print rerun directive
            println!("cargo:rerun-if-changed={}", ts_path.display());

            // Read the TypeScript source
            let mut ts_source = fs::read_to_string(&full_ts_path)?;

            // Append extensibility APIs if enabled
            if module.extensibility.is_enabled() {
                let ext_gen = crate::codegen::ExtensibilityGenerator::new(&module);
                ts_source.push_str("\n\n");
                ts_source.push_str(&ext_gen.generate());
            }

            // Transpile the combined TypeScript
            let specifier = format!("file://{}", full_ts_path.display());
            crate::build::transpile::transpile_ts(&ts_source, &specifier)?
        } else {
            // No TypeScript, generate minimal JS
            "// No TypeScript source provided\n".to_string()
        };

        // Generate extension.rs
        let extension_gen = ExtensionGenerator::new(&module);
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
                let dts_gen = DtsGenerator::new(&module);
                dts_gen.generate()
            };

            // Write to SDK generated directory
            let dts_filename = format!("{}.d.ts", module.specifier.replace(':', "."));
            let dts_path = generated_dir.join(&dts_filename);
            fs::write(&dts_path, &dts_content)?;

            // Also write to OUT_DIR for reference
            let out_dts_path = out_path.join(&dts_filename);
            fs::write(&out_dts_path, &dts_content)?;
        }

        // Generate SDK .ts module if requested
        if let Some(ref sdk_relative_path) = self.sdk_module_path {
            let workspace_root = manifest_path.parent().unwrap().parent().unwrap();
            let sdk_dir = workspace_root.join(sdk_relative_path);
            fs::create_dir_all(&sdk_dir)?;

            // Prefer the author-provided TypeScript source (ts/init.ts) so the SDK
            // mirrors the actual runtime module with its full typings and comments.
            // If no ts_path was provided, fall back to synthesized codegen.
            let mut ts_content = if let Some(ref ts_path) = self.ts_path {
                fs::read_to_string(manifest_path.join(ts_path))?
            } else {
                let ts_gen = TypeScriptGenerator::new(&module);
                ts_gen.generate()
            };

            // Ensure top-level type declarations are exported for SDK consumers.
            ts_content = export_types(ts_content);

            // Append extensibility APIs if enabled
            if module.extensibility.is_enabled() {
                let ext_gen = crate::codegen::ExtensibilityGenerator::new(&module);
                ts_content.push_str("\n\n");
                ts_content.push_str(&ext_gen.generate());
            }

            let module_name = module
                .specifier
                .split_once(':')
                .map(|(_, right)| right.to_string())
                .unwrap_or_else(|| module.specifier.replace(':', "."));
            let ts_filename = format!("runtime.{}.ts", module_name);
            let ts_path = sdk_dir.join(&ts_filename);
            fs::write(&ts_path, &ts_content)?;

            // Also write to OUT_DIR for reference
            let out_ts_path = out_path.join(&ts_filename);
            fs::write(&out_ts_path, &ts_content)?;
        }

        // Save module info for documentation generation by external tools (forge_cli)
        // This avoids the cyclic dependency (forge-etch depends on forge-weld)
        if let Some(ref doc_config) = self.doc_config {
            // Serialize the WeldModule as JSON for later doc generation
            let module_json = serde_json::to_string_pretty(&module).map_err(|e| {
                ExtensionBuilderError::DocGeneration(format!(
                    "Failed to serialize module for docs: {}",
                    e
                ))
            })?;

            // Save to OUT_DIR for forge_cli to discover
            let docs_dir = out_path.join("docs");
            fs::create_dir_all(&docs_dir)?;

            let module_json_path = docs_dir.join("module.json");
            fs::write(&module_json_path, &module_json)?;

            // Save doc config as JSON
            let doc_config_json = serde_json::json!({
                "output_dir": doc_config.output_dir,
                "format": match doc_config.format {
                    DocFormat::Astro => "astro",
                    DocFormat::Html => "html",
                    DocFormat::Both => "both",
                },
                "title": doc_config.title,
                "description": doc_config.description,
            });
            let config_json_path = docs_dir.join("doc_config.json");
            fs::write(
                &config_json_path,
                serde_json::to_string_pretty(&doc_config_json)
                    .map_err(|e| ExtensionBuilderError::DocGeneration(e.to_string()))?,
            )?;

            // Also create a marker file with the specifier for easy discovery
            let marker_path = docs_dir.join("specifier.txt");
            fs::write(&marker_path, &module.specifier)?;

            println!(
                "cargo:warning=Documentation config saved for '{}'. Run 'forge docs' to generate.",
                module.specifier
            );
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

        let spec = self.module.specifier.clone();
        let dts_filename = format!("{}.d.ts", spec.replace(':', "."));
        let module_name = spec
            .split_once(':')
            .map(|(_, right)| right.to_string())
            .unwrap_or_else(|| spec.replace(':', "."));
        let ts_filename = format!("runtime.{}.ts", module_name);

        let has_sdk_path = self.sdk_path.is_some();
        let has_sdk_module_path = self.sdk_module_path.is_some();

        self.build()?;

        Ok(BuildOutput {
            extension_rs: PathBuf::from(&out_dir).join("extension.rs"),
            dts_path: if has_sdk_path {
                Some(PathBuf::from(&out_dir).join(dts_filename))
            } else {
                None
            },
            sdk_ts_path: if has_sdk_module_path {
                Some(PathBuf::from(&out_dir).join(ts_filename))
            } else {
                None
            },
        })
    }

    // Note: Direct documentation generation is not possible here due to cyclic dependency
    // (forge-etch depends on forge-weld). Instead, when generate_docs() is called:
    // 1. The WeldModule is serialized to JSON in OUT_DIR/docs/module.json
    // 2. The DocConfig is saved to OUT_DIR/docs/doc_config.json
    // 3. forge_cli can read these files and generate docs using forge-etch
    // Use: forge docs --extension <name>  or  forge docs --all
}

/// Output paths from a successful build
#[derive(Debug)]
pub struct BuildOutput {
    /// Path to generated extension.rs
    pub extension_rs: PathBuf,
    /// Path to generated .d.ts (if SDK generation was enabled)
    pub dts_path: Option<PathBuf>,
    /// Path to generated runtime .ts SDK module (if enabled)
    pub sdk_ts_path: Option<PathBuf>,
}

/// Shorthand for creating a host extension builder
///
/// # Example
/// ```no_run
/// use forge_weld::build::host_extension;
///
/// host_extension("fs")
///     .ts_path("ts/init.ts")
///     .ops(&["op_fs_read_text"])
///     .build()
///     .unwrap();
/// ```
pub fn host_extension(name: &str) -> ExtensionBuilder {
    ExtensionBuilder::host(name)
}

fn export_types(ts: String) -> String {
    ts.lines()
        .map(|line| {
            // Only modify top-level declarations (no leading export/declare already)
            let trimmed = line.trim_start();
            let needs_export = (trimmed.starts_with("interface ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("enum "))
                && !trimmed.starts_with("export ")
                && !trimmed.starts_with("declare ");

            if needs_export {
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                format!("{}export {}", indent, trimmed)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_builder_creation() {
        let builder = ExtensionBuilder::new("host_fs", "runtime:fs")
            .ts_path("ts/init.ts")
            .ops(&["op_fs_read_text", "op_fs_write_text"]);

        assert_eq!(builder.module.name, "host_fs");
        assert_eq!(builder.module.specifier, "runtime:fs");
        assert_eq!(builder.module.ops.len(), 2);
    }

    #[test]
    fn test_host_extension() {
        let builder = ExtensionBuilder::host("net").ops(&["op_net_fetch"]);

        assert_eq!(builder.module.name, "host_net");
        assert_eq!(builder.module.specifier, "runtime:net");
    }
}
