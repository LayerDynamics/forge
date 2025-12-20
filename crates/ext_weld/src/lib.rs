//! forge:weld extension - Code generation and TypeScript binding utilities
//!
//! Provides runtime access to Forge Weld code generation capabilities:
//! - TypeScript transpilation
//! - Type generation from Rust definitions
//! - SDK module generation
//! - DTS and TypeScript code generation from module definitions

use deno_core::{op2, Extension, OpState};
use forge_weld::{
    transpile_ts, DtsGenerator, EnumVariant, OpParam, OpSymbol, StructField, TypeScriptGenerator,
    WeldEnum, WeldModule, WeldPrimitive, WeldStruct, WeldType,
};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WeldErrorCode {
    /// Transpilation failed
    TranspileError = 8000,
    /// Code generation failed
    CodegenError = 8001,
    /// Invalid input
    InvalidInput = 8002,
    /// Module not found
    ModuleNotFound = 8003,
}

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum WeldError {
    #[error("[{code}] Transpile error: {message}")]
    #[class(generic)]
    TranspileError { code: u32, message: String },

    #[error("[{code}] Codegen error: {message}")]
    #[class(generic)]
    CodegenError { code: u32, message: String },

    #[error("[{code}] Invalid input: {message}")]
    #[class(generic)]
    InvalidInput { code: u32, message: String },

    #[error("[{code}] Module not found: {message}")]
    #[class(generic)]
    ModuleNotFound { code: u32, message: String },
}

impl WeldError {
    pub fn transpile_error(message: impl Into<String>) -> Self {
        Self::TranspileError {
            code: WeldErrorCode::TranspileError as u32,
            message: message.into(),
        }
    }

    pub fn codegen_error(message: impl Into<String>) -> Self {
        Self::CodegenError {
            code: WeldErrorCode::CodegenError as u32,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: WeldErrorCode::InvalidInput as u32,
            message: message.into(),
        }
    }

    pub fn module_not_found(message: impl Into<String>) -> Self {
        Self::ModuleNotFound {
            code: WeldErrorCode::ModuleNotFound as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// State
// ============================================================================

/// Weld extension state for storing registered modules
#[derive(Default)]
pub struct WeldState {
    /// Cached module definitions for code generation
    modules: HashMap<String, WeldModule>,
}

impl WeldState {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module: WeldModule) {
        self.modules.insert(module.specifier.clone(), module);
    }

    pub fn get_module(&self, specifier: &str) -> Option<&WeldModule> {
        self.modules.get(specifier)
    }

    pub fn list_modules(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

// ============================================================================
// Types for Runtime Module Definition
// ============================================================================

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspileOptions {
    /// Source file name (for error messages)
    pub filename: Option<String>,
    /// Whether to include source maps
    pub source_map: Option<bool>,
    /// Whether to minify output
    pub minify: Option<bool>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct TranspileResult {
    /// Transpiled JavaScript code
    pub code: String,
    /// Source map (if requested)
    pub source_map: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,
    /// TypeScript type definition
    pub definition: String,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ExtensionInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub capabilities: Vec<&'static str>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

/// Runtime module definition that can be converted to WeldModule
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeModuleDefinition {
    /// Module name (e.g., "my_module")
    pub name: String,
    /// Module specifier (e.g., "custom:my-module")
    pub specifier: String,
    /// Documentation for the module
    pub doc: Option<String>,
    /// Struct definitions
    pub structs: Vec<RuntimeStructDefinition>,
    /// Enum definitions
    pub enums: Vec<RuntimeEnumDefinition>,
    /// Op/function definitions
    pub ops: Vec<RuntimeOpDefinition>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStructDefinition {
    pub name: String,
    pub ts_name: Option<String>,
    pub doc: Option<String>,
    pub fields: Vec<RuntimeFieldDefinition>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFieldDefinition {
    pub name: String,
    pub ts_name: Option<String>,
    pub ts_type: String,
    pub doc: Option<String>,
    pub optional: Option<bool>,
    pub readonly: Option<bool>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnumDefinition {
    pub name: String,
    pub ts_name: Option<String>,
    pub doc: Option<String>,
    pub variants: Vec<RuntimeVariantDefinition>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVariantDefinition {
    pub name: String,
    pub value: Option<String>,
    pub doc: Option<String>,
    pub data_type: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOpDefinition {
    pub rust_name: String,
    pub ts_name: Option<String>,
    pub doc: Option<String>,
    pub is_async: Option<bool>,
    pub params: Vec<RuntimeParamDefinition>,
    pub return_type: Option<String>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeParamDefinition {
    pub name: String,
    pub ts_name: Option<String>,
    pub ts_type: String,
    pub doc: Option<String>,
    pub optional: Option<bool>,
}

#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedCode {
    /// Generated TypeScript/JavaScript code
    pub code: String,
    /// Generated .d.ts declarations
    pub dts: String,
}

// ============================================================================
// Ops
// ============================================================================

/// Get extension info
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_info() -> ExtensionInfo {
    ExtensionInfo {
        name: "ext_weld",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: vec![
            "transpile",
            "generate_types",
            "generate_dts",
            "generate_ts",
            "register_module",
        ],
    }
}

/// Transpile TypeScript to JavaScript
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_transpile(
    #[string] source: String,
    #[serde] options: Option<TranspileOptions>,
) -> Result<TranspileResult, WeldError> {
    let opts = options.unwrap_or(TranspileOptions {
        filename: None,
        source_map: None,
        minify: None,
    });

    debug!(filename = ?opts.filename, "weld.transpile");

    let specifier = opts.filename.as_deref().unwrap_or("input.ts");
    let code =
        transpile_ts(&source, specifier).map_err(|e| WeldError::transpile_error(e.to_string()))?;

    Ok(TranspileResult {
        code,
        source_map: None, // TODO: implement source map support
    })
}

/// Generate TypeScript type declarations from type definitions
#[weld_op]
#[op2]
#[string]
pub fn op_weld_generate_dts(#[serde] types: Vec<TypeDefinition>) -> Result<String, WeldError> {
    debug!(type_count = types.len(), "weld.generate_dts");

    let mut output = String::new();
    output.push_str("// Generated by forge:weld\n\n");

    for typedef in types {
        output.push_str(&format!("export {};\n", typedef.definition));
    }

    Ok(output)
}

/// Generate a TypeScript interface from a JSON schema-like definition
#[weld_op]
#[op2]
#[string]
pub fn op_weld_json_to_interface(
    #[string] name: String,
    #[string] json_schema: String,
) -> Result<String, WeldError> {
    debug!(name = %name, "weld.json_to_interface");

    // Parse the JSON to validate it
    let value: serde_json::Value = serde_json::from_str(&json_schema)
        .map_err(|e| WeldError::invalid_input(format!("Invalid JSON: {}", e)))?;

    let mut output = String::new();
    output.push_str(&format!("export interface {} {{\n", name));

    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            let ts_type = json_value_to_ts_type(&val);
            output.push_str(&format!("  {}: {};\n", key, ts_type));
        }
    }

    output.push_str("}\n");
    Ok(output)
}

/// Validate TypeScript syntax
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_validate_ts(#[string] source: String) -> Result<ValidationResult, WeldError> {
    debug!("weld.validate_ts");

    // Try to transpile - if it succeeds, syntax is valid
    match transpile_ts(&source, "validation.ts") {
        Ok(_) => Ok(ValidationResult {
            valid: true,
            errors: vec![],
        }),
        Err(e) => Ok(ValidationResult {
            valid: false,
            errors: vec![e.to_string()],
        }),
    }
}

/// Register a module definition for code generation
#[weld_op]
#[op2]
pub fn op_weld_register_module(
    state: &mut OpState,
    #[serde] definition: RuntimeModuleDefinition,
) -> Result<(), WeldError> {
    debug!(name = %definition.name, specifier = %definition.specifier, "weld.register_module");

    let module = convert_runtime_to_weld_module(definition)?;
    let weld_state = state.borrow_mut::<WeldState>();
    weld_state.register_module(module);

    Ok(())
}

/// List all registered modules
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_list_modules(state: &mut OpState) -> Vec<String> {
    debug!("weld.list_modules");

    let weld_state = state.borrow::<WeldState>();
    weld_state
        .list_modules()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Generate TypeScript code for a registered module using TypeScriptGenerator
#[weld_op]
#[op2]
#[string]
pub fn op_weld_generate_module_ts(
    state: &mut OpState,
    #[string] specifier: String,
) -> Result<String, WeldError> {
    debug!(specifier = %specifier, "weld.generate_module_ts");

    let weld_state = state.borrow::<WeldState>();
    let module = weld_state.get_module(&specifier).ok_or_else(|| {
        WeldError::module_not_found(format!("Module '{}' not registered", specifier))
    })?;

    let generator = TypeScriptGenerator::new(module);
    Ok(generator.generate())
}

/// Generate .d.ts declarations for a registered module using DtsGenerator
#[weld_op]
#[op2]
#[string]
pub fn op_weld_generate_module_dts(
    state: &mut OpState,
    #[string] specifier: String,
) -> Result<String, WeldError> {
    debug!(specifier = %specifier, "weld.generate_module_dts");

    let weld_state = state.borrow::<WeldState>();
    let module = weld_state.get_module(&specifier).ok_or_else(|| {
        WeldError::module_not_found(format!("Module '{}' not registered", specifier))
    })?;

    let generator = DtsGenerator::new(module);
    Ok(generator.generate())
}

/// Generate both TypeScript and .d.ts for a registered module
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_generate_module(
    state: &mut OpState,
    #[string] specifier: String,
) -> Result<GeneratedCode, WeldError> {
    debug!(specifier = %specifier, "weld.generate_module");

    let weld_state = state.borrow::<WeldState>();
    let module = weld_state.get_module(&specifier).ok_or_else(|| {
        WeldError::module_not_found(format!("Module '{}' not registered", specifier))
    })?;

    let ts_generator = TypeScriptGenerator::new(module);
    let dts_generator = DtsGenerator::new(module);

    Ok(GeneratedCode {
        code: ts_generator.generate(),
        dts: dts_generator.generate(),
    })
}

/// Generate TypeScript code from an inline module definition (without registering)
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_generate_from_definition(
    #[serde] definition: RuntimeModuleDefinition,
) -> Result<GeneratedCode, WeldError> {
    debug!(name = %definition.name, "weld.generate_from_definition");

    let module = convert_runtime_to_weld_module(definition)?;

    let ts_generator = TypeScriptGenerator::new(&module);
    let dts_generator = DtsGenerator::new(&module);

    Ok(GeneratedCode {
        code: ts_generator.generate(),
        dts: dts_generator.generate(),
    })
}

// ============================================================================
// Weld + Etcher Integration Ops
// ============================================================================

/// Configuration for documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeldDocGenConfig {
    /// Module specifier to generate documentation for
    pub specifier: String,
    /// Output directory for documentation files
    pub output_dir: String,
    /// Generate Astro markdown
    pub generate_astro: Option<bool>,
    /// Generate HTML
    pub generate_html: Option<bool>,
    /// Documentation title
    pub title: Option<String>,
    /// Documentation description
    pub description: Option<String>,
}

/// Configuration for SDK + documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkDocsConfig {
    /// Module specifier to generate for
    pub specifier: String,
    /// Output directory for SDK files
    pub sdk_output_dir: String,
    /// Output directory for documentation files
    pub docs_output_dir: String,
    /// Generate Astro markdown
    pub generate_astro: Option<bool>,
    /// Generate HTML
    pub generate_html: Option<bool>,
    /// Documentation title
    pub title: Option<String>,
    /// Documentation description
    pub description: Option<String>,
}

/// Configuration for register and document operation
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAndDocumentConfig {
    /// Module definition
    pub definition: RuntimeModuleDefinition,
    /// Output directory for documentation files
    pub docs_output_dir: String,
    /// Generate Astro markdown
    pub generate_astro: Option<bool>,
    /// Generate HTML
    pub generate_html: Option<bool>,
}

/// Result of SDK + documentation generation
#[weld_struct]
#[derive(Debug, Clone, Serialize)]
pub struct SdkDocsResult {
    /// Generated TypeScript code
    pub code: String,
    /// Generated .d.ts declarations
    pub dts: String,
    /// Number of symbols documented
    pub symbol_count: usize,
    /// Generated Astro files
    pub astro_files: Vec<String>,
    /// Generated HTML files
    pub html_files: Vec<String>,
}

/// Generate documentation for a registered module using forge-etch
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_generate_docs(
    state: &mut OpState,
    #[serde] doc_config: WeldDocGenConfig,
) -> Result<DocGenResult, WeldError> {
    debug!(specifier = %doc_config.specifier, output_dir = %doc_config.output_dir, "weld.generate_docs");

    // Clone module inside a scope to release the borrow
    let module = {
        let weld_state = state.borrow::<WeldState>();
        weld_state
            .get_module(&doc_config.specifier)
            .ok_or_else(|| {
                WeldError::module_not_found(format!(
                    "Module '{}' not registered",
                    doc_config.specifier
                ))
            })?
            .clone()
    };

    // Create a custom Etcher with the nodes from WeldModule
    let config = forge_etch::docgen::EtchConfig {
        name: module.name.clone(),
        module_specifier: module.specifier.clone(),
        rust_source: None,
        ts_source: None,
        output_dir: std::path::PathBuf::from(&doc_config.output_dir),
        generate_astro: doc_config.generate_astro.unwrap_or(true),
        generate_html: doc_config.generate_html.unwrap_or(false),
        title: doc_config.title.or(module.doc.clone()),
        description: doc_config.description,
        include_private: false,
        include_internal: false,
    };

    let mut etcher = forge_etch::docgen::Etcher::new(config).with_weld_module(module);

    let output = etcher
        .run()
        .map_err(|e| WeldError::codegen_error(e.to_string()))?;

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

/// Register a module and generate documentation in one call
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_register_and_document(
    state: &mut OpState,
    #[serde] reg_config: RegisterAndDocumentConfig,
) -> Result<DocGenResult, WeldError> {
    debug!(name = %reg_config.definition.name, specifier = %reg_config.definition.specifier, "weld.register_and_document");

    let module = convert_runtime_to_weld_module(reg_config.definition.clone())?;

    // Register the module
    {
        let weld_state = state.borrow_mut::<WeldState>();
        weld_state.register_module(module.clone());
    }

    // Generate documentation
    let config = forge_etch::docgen::EtchConfig {
        name: module.name.clone(),
        module_specifier: module.specifier.clone(),
        rust_source: None,
        ts_source: None,
        output_dir: std::path::PathBuf::from(&reg_config.docs_output_dir),
        generate_astro: reg_config.generate_astro.unwrap_or(true),
        generate_html: reg_config.generate_html.unwrap_or(false),
        title: module.doc.clone(),
        description: None,
        include_private: false,
        include_internal: false,
    };

    let mut etcher = forge_etch::docgen::Etcher::new(config).with_weld_module(module);

    let output = etcher
        .run()
        .map_err(|e| WeldError::codegen_error(e.to_string()))?;

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

/// Generate SDK code and documentation together
#[weld_op]
#[op2]
#[serde]
pub fn op_weld_generate_sdk_with_docs(
    state: &mut OpState,
    #[serde] config: SdkDocsConfig,
) -> Result<SdkDocsResult, WeldError> {
    debug!(specifier = %config.specifier, "weld.generate_sdk_with_docs");

    // Clone module inside a scope to release the borrow
    let module = {
        let weld_state = state.borrow::<WeldState>();
        weld_state
            .get_module(&config.specifier)
            .ok_or_else(|| {
                WeldError::module_not_found(format!("Module '{}' not registered", config.specifier))
            })?
            .clone()
    };

    // Generate SDK code
    let ts_generator = TypeScriptGenerator::new(&module);
    let dts_generator = DtsGenerator::new(&module);
    let code = ts_generator.generate();
    let dts = dts_generator.generate();

    // Generate documentation
    let doc_config = forge_etch::docgen::EtchConfig {
        name: module.name.clone(),
        module_specifier: module.specifier.clone(),
        rust_source: None,
        ts_source: None,
        output_dir: std::path::PathBuf::from(&config.docs_output_dir),
        generate_astro: config.generate_astro.unwrap_or(true),
        generate_html: config.generate_html.unwrap_or(false),
        title: config.title.or(module.doc.clone()),
        description: config.description,
        include_private: false,
        include_internal: false,
    };

    let mut etcher = forge_etch::docgen::Etcher::new(doc_config).with_weld_module(module.clone());

    let output = etcher
        .run()
        .map_err(|e| WeldError::codegen_error(e.to_string()))?;

    // Write SDK files if output directory is specified
    if !config.sdk_output_dir.is_empty() {
        let sdk_dir = std::path::Path::new(&config.sdk_output_dir);
        std::fs::create_dir_all(sdk_dir).map_err(|e| {
            WeldError::codegen_error(format!("Failed to create SDK directory: {}", e))
        })?;

        let module_name = module
            .specifier
            .split_once(':')
            .map(|(_, right)| right.to_string())
            .unwrap_or_else(|| module.specifier.replace(':', "."));

        let ts_path = sdk_dir.join(format!("runtime.{}.ts", module_name));
        let dts_path = sdk_dir.join(format!("runtime.{}.d.ts", module_name));

        std::fs::write(&ts_path, &code)
            .map_err(|e| WeldError::codegen_error(format!("Failed to write TypeScript: {}", e)))?;
        std::fs::write(&dts_path, &dts)
            .map_err(|e| WeldError::codegen_error(format!("Failed to write .d.ts: {}", e)))?;
    }

    Ok(SdkDocsResult {
        code,
        dts,
        symbol_count: output.symbol_count,
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

/// Documentation generation result (compatible with etcher types)
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

// ============================================================================
// Helpers
// ============================================================================

fn json_value_to_ts_type(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "unknown[]".to_string()
            } else {
                format!("{}[]", json_value_to_ts_type(&arr[0]))
            }
        }
        serde_json::Value::Object(_) => "Record<string, unknown>".to_string(),
    }
}

/// Parse a TypeScript type string into a WeldType
fn parse_ts_type(ts_type: &str) -> WeldType {
    match ts_type {
        "string" => WeldType::string(),
        "number" => WeldType::Primitive(WeldPrimitive::F64),
        "boolean" => WeldType::bool(),
        "bigint" => WeldType::Primitive(WeldPrimitive::I64),
        "void" => WeldType::void(),
        "null" | "undefined" | "unknown" | "any" => WeldType::Unknown,
        "never" => WeldType::Never,
        s if s.ends_with("[]") => {
            let inner = &s[..s.len() - 2];
            WeldType::vec(parse_ts_type(inner))
        }
        s if s.starts_with("Promise<") && s.ends_with('>') => {
            let inner = &s[8..s.len() - 1];
            // Promise<T> is represented as Result<T, Error> in Weld
            WeldType::result(parse_ts_type(inner), WeldType::struct_ref("Error"))
        }
        s if s.starts_with("Record<") => WeldType::hashmap(WeldType::string(), WeldType::Unknown),
        other => WeldType::struct_ref(other),
    }
}

/// Convert runtime module definition to WeldModule
fn convert_runtime_to_weld_module(def: RuntimeModuleDefinition) -> Result<WeldModule, WeldError> {
    let mut module = WeldModule::new(&def.name, &def.specifier);

    if let Some(doc) = def.doc {
        module = module.with_doc(doc);
    }

    // Convert structs
    for struct_def in def.structs {
        let mut weld_struct = WeldStruct::new(&struct_def.name);
        if let Some(ts_name) = struct_def.ts_name {
            weld_struct = weld_struct.with_ts_name(&ts_name);
        }
        if let Some(doc) = struct_def.doc {
            weld_struct = weld_struct.with_doc(&doc);
        }

        for field_def in struct_def.fields {
            let mut field = StructField::new(
                field_def.ts_name.as_deref().unwrap_or(&field_def.name),
                parse_ts_type(&field_def.ts_type),
            );
            if let Some(doc) = field_def.doc {
                field = field.with_doc(&doc);
            }
            if field_def.optional.unwrap_or(false) {
                field = field.optional();
            }
            if field_def.readonly.unwrap_or(false) {
                field = field.readonly();
            }
            weld_struct = weld_struct.field(field);
        }

        module = module.struct_def(weld_struct);
    }

    // Convert enums
    for enum_def in def.enums {
        let mut weld_enum = WeldEnum::new(&enum_def.name);
        // Set ts_name directly on the struct field
        if let Some(ts_name) = enum_def.ts_name {
            weld_enum.ts_name = ts_name;
        }
        // Set doc directly on the struct field
        if let Some(doc) = enum_def.doc {
            weld_enum.doc = Some(doc);
        }

        for variant_def in enum_def.variants {
            let variant = EnumVariant {
                name: variant_def.name,
                value: variant_def.value,
                data: variant_def.data_type.map(|dt| parse_ts_type(&dt)),
                doc: variant_def.doc,
            };
            weld_enum = weld_enum.variant(variant);
        }

        module = module.enum_def(weld_enum);
    }

    // Convert ops
    for op_def in def.ops {
        let mut op = OpSymbol::from_rust_name(&op_def.rust_name);
        if let Some(ts_name) = op_def.ts_name {
            op = op.ts_name(&ts_name);
        }
        if let Some(doc) = op_def.doc {
            op = op.with_doc(&doc);
        }
        if op_def.is_async.unwrap_or(false) {
            op = op.async_op();
        }

        for param_def in op_def.params {
            let mut param = OpParam::new(
                param_def.ts_name.as_deref().unwrap_or(&param_def.name),
                parse_ts_type(&param_def.ts_type),
            );
            if let Some(doc) = param_def.doc {
                param = param.with_doc(&doc);
            }
            if param_def.optional.unwrap_or(false) {
                param = param.optional();
            }
            op = op.param(param);
        }

        if let Some(return_type) = op_def.return_type {
            op = op.returns(parse_ts_type(&return_type));
        }

        module = module.op(op);
    }

    Ok(module)
}

// ============================================================================
// Extension Setup
// ============================================================================

include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn weld_extension() -> Extension {
    ext_weld_runtime::ext()
}

/// Initialize the weld extension state in the op state
pub fn init_weld_state(state: &mut OpState) {
    state.put(WeldState::new());
}

// Re-export forge_weld for the macros
pub use forge_weld;
