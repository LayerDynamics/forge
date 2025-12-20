//! forge:etcher extension - Documentation generation from TypeScript and Rust sources
//!
//! Provides runtime access to Forge Etch documentation capabilities:
//! - Parse TypeScript sources for JSDoc and type definitions
//! - Generate Astro-compatible markdown documentation
//! - Generate standalone HTML documentation
//! - Use EtchBuilder for complete documentation pipelines

use deno_core::{op2, Extension, OpState};
use forge_etch::{EtchBuilder, EtchNode};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum EtcherErrorCode {
    /// Configuration error
    ConfigError = 9000,
    /// Parse error
    ParseError = 9001,
    /// Generation error
    GenerationError = 9002,
    /// IO error
    IoError = 9003,
    /// Source not found
    SourceNotFound = 9004,
}

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum EtcherError {
    #[error("[{code}] Config error: {message}")]
    #[class(generic)]
    ConfigError { code: u32, message: String },

    #[error("[{code}] Parse error: {message}")]
    #[class(generic)]
    ParseError { code: u32, message: String },

    #[error("[{code}] Generation error: {message}")]
    #[class(generic)]
    GenerationError { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },

    #[error("[{code}] Source not found: {message}")]
    #[class(generic)]
    SourceNotFound { code: u32, message: String },
}

impl EtcherError {
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            code: EtcherErrorCode::ConfigError as u32,
            message: message.into(),
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            code: EtcherErrorCode::ParseError as u32,
            message: message.into(),
        }
    }

    pub fn generation_error(message: impl Into<String>) -> Self {
        Self::GenerationError {
            code: EtcherErrorCode::GenerationError as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: EtcherErrorCode::IoError as u32,
            message: message.into(),
        }
    }

    pub fn source_not_found(message: impl Into<String>) -> Self {
        Self::SourceNotFound {
            code: EtcherErrorCode::SourceNotFound as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// Etcher extension state (currently empty, reserved for future caching)
#[derive(Default)]
pub struct EtcherState;

impl EtcherState {
    pub fn new() -> Self {
        Self
    }
}

// ============================================================================
// Types for Runtime Usage
// ============================================================================

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: Vec<&'static str>,
}

/// Configuration for documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocGenConfig {
    /// Extension/module name
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub specifier: String,
    /// Path to TypeScript source file
    pub ts_source: Option<String>,
    /// Path to Rust source file
    pub rust_source: Option<String>,
    /// Output directory
    pub output_dir: String,
    /// Generate Astro markdown
    pub generate_astro: Option<bool>,
    /// Generate HTML
    pub generate_html: Option<bool>,
    /// Documentation title
    pub title: Option<String>,
    /// Documentation description
    pub description: Option<String>,
    /// Include private symbols
    pub include_private: Option<bool>,
}

/// Result of documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct DocGenResult {
    /// Number of symbols documented
    pub symbol_count: usize,
    /// Output directory path
    pub output_dir: String,
    /// Generated Astro files
    pub astro_files: Vec<String>,
    /// Generated HTML files
    pub html_files: Vec<String>,
}

/// Serializable documentation node for runtime usage
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct DocNodeInfo {
    /// Symbol name
    pub name: String,
    /// Node kind as string
    pub kind: String,
    /// Module specifier (if any)
    pub module: Option<String>,
    /// Description from JSDoc/doc comments
    pub description: Option<String>,
    /// TypeScript signature (if applicable)
    pub signature: Option<String>,
    /// Whether this is a default export
    pub is_default: bool,
    /// Visibility (public, private, internal)
    pub visibility: String,
}

/// Result of TypeScript parsing
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ParseResult {
    /// Number of nodes parsed
    pub node_count: usize,
    /// Information about parsed nodes
    pub nodes: Vec<DocNodeInfo>,
}

// ============================================================================
// Ops
// ============================================================================

/// Get extension info
#[weld_op]
#[op2]
#[serde]
pub fn op_etcher_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_etcher",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: vec![
            "parse_ts",
            "generate_astro",
            "generate_html",
            "etch_builder",
        ],
    }
}

/// Generate documentation from a configuration
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_generate_docs(
    #[serde] config: DocGenConfig,
) -> Result<DocGenResult, EtcherError> {
    debug!(name = %config.name, specifier = %config.specifier, "etcher.generate_docs");

    let mut builder =
        EtchBuilder::new(&config.name, &config.specifier).output_dir(&config.output_dir);

    if let Some(ts) = &config.ts_source {
        builder = builder.ts_source(ts);
    }

    if let Some(rust) = &config.rust_source {
        builder = builder.rust_source(rust);
    }

    if let Some(astro) = config.generate_astro {
        builder = builder.generate_astro(astro);
    }

    if let Some(html) = config.generate_html {
        builder = builder.generate_html(html);
    }

    if let Some(title) = &config.title {
        builder = builder.title(title);
    }

    if let Some(desc) = &config.description {
        builder = builder.description(desc);
    }

    if let Some(private) = config.include_private {
        builder = builder.include_private(private);
    }

    let output = builder
        .build()
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(DocGenResult {
        symbol_count: output.symbol_count,
        output_dir: output.output_dir.to_string_lossy().to_string(),
        astro_files: output
            .astro_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        html_files: output
            .html_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

/// Parse TypeScript source file and extract documentation info
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_parse_ts(#[string] source_path: String) -> Result<ParseResult, EtcherError> {
    debug!(path = %source_path, "etcher.parse_ts");

    let path = PathBuf::from(&source_path);
    if !path.exists() {
        return Err(EtcherError::source_not_found(format!(
            "TypeScript source not found: {}",
            source_path
        )));
    }

    // Use forge-etch parser
    let nodes = forge_etch::parser::parse_typescript(&path)
        .map_err(|e| EtcherError::parse_error(e.to_string()))?;

    let node_infos: Vec<DocNodeInfo> = nodes.iter().map(etch_node_to_info).collect();

    Ok(ParseResult {
        node_count: node_infos.len(),
        nodes: node_infos,
    })
}

/// Parse Rust source file (extracts weld metadata)
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_parse_rust(
    #[string] _source_path: String,
) -> Result<ParseResult, EtcherError> {
    // Note: Rust parsing in forge-etch is primarily done through the weld
    // inventory system at build time. For runtime usage, use EtchBuilder
    // which automatically handles Rust source extraction.
    Err(EtcherError::config_error(
        "Direct Rust parsing not yet implemented. Use op_etcher_generate_docs with rust_source option instead.",
    ))
}

/// Merge TypeScript and Rust documentation (using EtchBuilder internally)
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_merge_nodes(
    #[string] name: String,
    #[string] specifier: String,
    #[string] ts_source: Option<String>,
    #[string] rust_source: Option<String>,
) -> Result<ParseResult, EtcherError> {
    debug!(name = %name, specifier = %specifier, ?ts_source, ?rust_source, "etcher.merge_nodes");

    // Parse TypeScript source if provided
    let ts_nodes = if let Some(ts_path) = &ts_source {
        let path = PathBuf::from(ts_path);
        if path.exists() {
            forge_etch::parser::parse_typescript(&path)
                .map_err(|e| EtcherError::parse_error(e.to_string()))?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Note: Runtime Rust parsing is not yet implemented. Rust source metadata
    // is collected at build time via the weld inventory system.
    // For full merge including Rust types, use op_etcher_generate_docs instead.
    let rust_nodes: Vec<EtchNode> = if rust_source.is_some() {
        debug!("rust_source provided but runtime Rust parsing not implemented");
        Vec::new()
    } else {
        Vec::new()
    };

    // Merge nodes (TSDoc/JSDoc takes precedence)
    let merged = if rust_nodes.is_empty() {
        ts_nodes
    } else {
        forge_etch::parser::merge_nodes(ts_nodes, rust_nodes)
    };

    let node_infos: Vec<DocNodeInfo> = merged.iter().map(etch_node_to_info).collect();

    Ok(ParseResult {
        node_count: node_infos.len(),
        nodes: node_infos,
    })
}

/// Generate Astro markdown from a crate directory
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_nodes_to_astro(
    #[string] name: String,
    #[string] specifier: String,
    #[string] crate_root: String,
    #[string] output_dir: String,
) -> Result<DocGenResult, EtcherError> {
    debug!(crate_root = %crate_root, output_dir = %output_dir, "etcher.nodes_to_astro");

    let builder = EtchBuilder::from_crate_root(&name, &specifier, &crate_root)
        .output_dir(&output_dir)
        .generate_astro(true)
        .generate_html(false);

    let output = builder
        .build()
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(DocGenResult {
        symbol_count: output.symbol_count,
        output_dir: output.output_dir.to_string_lossy().to_string(),
        astro_files: output
            .astro_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        html_files: vec![],
    })
}

/// Generate HTML from a crate directory
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_nodes_to_html(
    #[string] name: String,
    #[string] specifier: String,
    #[string] crate_root: String,
    #[string] output_dir: String,
) -> Result<DocGenResult, EtcherError> {
    debug!(crate_root = %crate_root, output_dir = %output_dir, "etcher.nodes_to_html");

    let builder = EtchBuilder::from_crate_root(&name, &specifier, &crate_root)
        .output_dir(&output_dir)
        .generate_astro(false)
        .generate_html(true);

    let output = builder
        .build()
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(DocGenResult {
        symbol_count: output.symbol_count,
        output_dir: output.output_dir.to_string_lossy().to_string(),
        astro_files: vec![],
        html_files: output
            .html_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

// ============================================================================
// WeldModule-based Documentation Ops
// ============================================================================

/// Configuration for WeldModule documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeldModuleDocConfig {
    /// Module name (e.g., "host_fs")
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub specifier: String,
    /// Module documentation
    pub doc: Option<String>,
    /// Output directory
    pub output_dir: String,
    /// Generate Astro markdown
    pub generate_astro: Option<bool>,
    /// Generate HTML
    pub generate_html: Option<bool>,
    /// Documentation title
    pub title: Option<String>,
    /// Documentation description
    pub description: Option<String>,
    /// Struct definitions as JSON
    pub structs_json: Option<String>,
    /// Enum definitions as JSON
    pub enums_json: Option<String>,
    /// Op definitions as JSON
    pub ops_json: Option<String>,
}

/// Generate documentation from WeldModule data passed as JSON
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_from_weld_module(
    #[serde] config: WeldModuleDocConfig,
) -> Result<DocGenResult, EtcherError> {
    debug!(name = %config.name, specifier = %config.specifier, "etcher.from_weld_module");

    // Build WeldModule from the config
    let mut module = forge_weld::ir::WeldModule::new(&config.name, &config.specifier);
    if let Some(doc) = &config.doc {
        module = module.with_doc(doc.clone());
    }

    // Parse structs from JSON if provided
    if let Some(ref structs_json) = config.structs_json {
        let structs: Vec<forge_weld::ir::WeldStruct> = serde_json::from_str(structs_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid structs JSON: {}", e)))?;
        module.structs = structs;
    }

    // Parse enums from JSON if provided
    if let Some(ref enums_json) = config.enums_json {
        let enums: Vec<forge_weld::ir::WeldEnum> = serde_json::from_str(enums_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid enums JSON: {}", e)))?;
        module.enums = enums;
    }

    // Parse ops from JSON if provided
    if let Some(ref ops_json) = config.ops_json {
        let ops: Vec<forge_weld::ir::OpSymbol> = serde_json::from_str(ops_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid ops JSON: {}", e)))?;
        module.ops = ops;
    }

    // Create Etcher with the WeldModule
    let etch_config = forge_etch::docgen::EtchConfig {
        name: config.name.clone(),
        module_specifier: config.specifier.clone(),
        rust_source: None,
        ts_source: None,
        output_dir: PathBuf::from(&config.output_dir),
        generate_astro: config.generate_astro.unwrap_or(true),
        generate_html: config.generate_html.unwrap_or(false),
        title: config.title.clone(),
        description: config.description.clone(),
        include_private: false,
        include_internal: false,
    };

    let mut etcher = forge_etch::docgen::Etcher::new(etch_config).with_weld_module(module);

    let output = etcher
        .run()
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(DocGenResult {
        symbol_count: output.symbol_count,
        output_dir: output.output_dir.to_string_lossy().to_string(),
        astro_files: output
            .astro_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        html_files: output
            .html_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

// ============================================================================
// Site Update/Regeneration Ops
// ============================================================================

/// Configuration for site update
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteUpdateConfig {
    /// Output directory for the site
    pub output_dir: String,
    /// Whether to clean the output directory first
    pub clean_first: Option<bool>,
    /// Extension documentation definitions as JSON array
    pub docs_json: Option<String>,
}

/// Result of site update
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SiteUpdateResult {
    /// Number of files generated
    pub generated_count: usize,
    /// Paths to generated files
    pub generated_files: Vec<String>,
    /// Number of files removed (when cleaning)
    pub removed_count: usize,
    /// Paths to removed files
    pub removed_files: Vec<String>,
    /// Total number of symbols documented
    pub total_symbols: usize,
    /// Number of modules processed
    pub module_count: usize,
}

/// Astro configuration validation result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct AstroConfigValidation {
    /// Whether all validation checks passed
    pub valid: bool,
    /// Individual validation results
    pub checks: Vec<ValidationCheck>,
    /// Site URL if found
    pub site_url: Option<String>,
    /// Starlight title if found
    pub starlight_title: Option<String>,
}

/// Individual validation check result
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ValidationCheck {
    /// Whether this check passed
    pub passed: bool,
    /// Description of what was checked
    pub description: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Extension documentation info for site updates
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDocInfo {
    /// Extension name
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub specifier: String,
    /// Documentation title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
}

/// Update the Astro site with new documentation
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_update_site(
    #[serde] config: SiteUpdateConfig,
) -> Result<SiteUpdateResult, EtcherError> {
    debug!(output_dir = %config.output_dir, "etcher.update_site");

    let output_dir = PathBuf::from(&config.output_dir);

    // Parse docs from JSON if provided
    let docs: Vec<forge_etch::docgen::ExtensionDoc> = if let Some(ref docs_json) = config.docs_json
    {
        serde_json::from_str(docs_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid docs JSON: {}", e)))?
    } else {
        Vec::new()
    };

    // Build the site update request
    let update = forge_etch::astro::SiteUpdate::new(output_dir.clone()).with_docs(docs);

    let update = if config.clean_first.unwrap_or(false) {
        update.clean()
    } else {
        update
    };

    // Create generator and run update
    let generator = forge_etch::astro::AstroGenerator::new(output_dir);
    let result = forge_etch::astro::update_site(&generator, &update)
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(SiteUpdateResult {
        generated_count: result.generated.len(),
        generated_files: result
            .generated
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        removed_count: result.removed.len(),
        removed_files: result
            .removed
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        total_symbols: result.total_symbols,
        module_count: result.module_count,
    })
}

/// Site regeneration configuration
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteRegenConfig {
    /// Output directory for the site
    pub output_dir: String,
    /// Extension documentation definitions as JSON array
    pub docs_json: Option<String>,
}

/// Regenerate the entire site documentation (cleans first)
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_regenerate_site(
    #[serde] config: SiteRegenConfig,
) -> Result<SiteUpdateResult, EtcherError> {
    debug!(output_dir = %config.output_dir, "etcher.regenerate_site");

    let output_dir = PathBuf::from(&config.output_dir);

    // Parse docs from JSON if provided
    let docs: Vec<forge_etch::docgen::ExtensionDoc> = if let Some(ref docs_json) = config.docs_json
    {
        serde_json::from_str(docs_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid docs JSON: {}", e)))?
    } else {
        Vec::new()
    };

    let result = forge_etch::astro::regenerate_site(output_dir, docs)
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(SiteUpdateResult {
        generated_count: result.generated.len(),
        generated_files: result
            .generated
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        removed_count: result.removed.len(),
        removed_files: result
            .removed
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        total_symbols: result.total_symbols,
        module_count: result.module_count,
    })
}

/// Site index generation configuration
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteIndexConfig {
    /// Output directory for the index
    pub output_dir: String,
    /// Extension documentation definitions as JSON array
    pub docs_json: Option<String>,
}

/// Generate a site-wide index page listing all modules
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_etcher_generate_site_index(
    #[serde] config: SiteIndexConfig,
) -> Result<String, EtcherError> {
    debug!(output_dir = %config.output_dir, "etcher.generate_site_index");

    let output_dir = PathBuf::from(&config.output_dir);

    // Parse docs from JSON if provided
    let docs: Vec<forge_etch::docgen::ExtensionDoc> = if let Some(ref docs_json) = config.docs_json
    {
        serde_json::from_str(docs_json)
            .map_err(|e| EtcherError::parse_error(format!("Invalid docs JSON: {}", e)))?
    } else {
        Vec::new()
    };

    let index_path = forge_etch::astro::generate_site_index(&output_dir, &docs)
        .map_err(|e| EtcherError::generation_error(e.to_string()))?;

    Ok(index_path.to_string_lossy().to_string())
}

/// Validate Astro configuration
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_validate_config(
    #[string] project_dir: String,
) -> Result<AstroConfigValidation, EtcherError> {
    debug!(project_dir = %project_dir, "etcher.validate_config");

    let project_path = PathBuf::from(&project_dir);

    // Run comprehensive validation checks
    let results = forge_etch::astro::validate_config(&project_path)
        .map_err(|e| EtcherError::config_error(e.to_string()))?;

    // Check overall validity - all checks must pass
    let valid = results.iter().all(|r| r.passed);

    // Convert results to our format
    let checks: Vec<ValidationCheck> = results
        .iter()
        .map(|r| ValidationCheck {
            passed: r.passed,
            description: r.description.clone(),
            error: r.error.clone(),
        })
        .collect();

    // Also try to get the config for additional info
    let (site_url, starlight_title) =
        if let Ok(config) = forge_etch::astro::check_config(&project_path) {
            (config.site_url, config.starlight.map(|s| s.title))
        } else {
            (None, None)
        };

    Ok(AstroConfigValidation {
        valid,
        checks,
        site_url,
        starlight_title,
    })
}

/// Output directory validation configuration
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDirValidationConfig {
    /// Target directory or slug to validate
    pub target: String,
    /// Project directory path (for loading astro config)
    pub project_dir: String,
}

/// Check if output directory is properly configured in sidebar
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_etcher_validate_output_dir(
    #[serde] config: OutputDirValidationConfig,
) -> Result<ValidationCheck, EtcherError> {
    debug!(target = %config.target, project_dir = %config.project_dir, "etcher.validate_output_dir");

    let project_path = PathBuf::from(&config.project_dir);

    // First get the Astro config
    let astro_config = forge_etch::astro::check_config(&project_path)
        .map_err(|e| EtcherError::config_error(e.to_string()))?;

    // Validate the output directory
    match forge_etch::astro::validate_output_dir(&astro_config, &config.target) {
        Ok(()) => Ok(ValidationCheck {
            passed: true,
            description: format!(
                "Target '{}' is configured in Starlight sidebar",
                config.target
            ),
            error: None,
        }),
        Err(e) => Ok(ValidationCheck {
            passed: false,
            description: format!("Target '{}' sidebar configuration check", config.target),
            error: Some(e.to_string()),
        }),
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert EtchNode to serializable DocNodeInfo
fn etch_node_to_info(node: &EtchNode) -> DocNodeInfo {
    DocNodeInfo {
        name: node.name.clone(),
        kind: node.kind().display_name().to_string(),
        module: node.module.clone(),
        description: node.doc.description.clone(),
        signature: node.to_typescript_signature_opt(),
        is_default: node.is_default.unwrap_or(false),
        visibility: format!("{:?}", node.visibility),
    }
}

// ============================================================================
// Extension Setup
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn etcher_extension() -> Extension {
    ext_etcher_runtime::ext()
}

/// Initialize the etcher extension state in the op state
pub fn init_etcher_state(state: &mut OpState) {
    state.put(EtcherState::new());
}

// Re-export forge_etch for direct access if needed
pub use forge_etch;
