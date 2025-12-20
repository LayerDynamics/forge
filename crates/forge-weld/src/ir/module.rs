//! Module metadata for Forge extensions
//!
//! This module provides the WeldModule structure that represents
//! an entire Forge extension module with its ops, structs, and configuration.

use crate::ir::{ExtensibilityConfig, OpSymbol, WeldEnum, WeldStruct};
use serde::{Deserialize, Serialize};

/// Metadata for an entire extension module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeldModule {
    /// Internal module name (e.g., "host_fs")
    pub name: String,

    /// Host specifier for imports (e.g., "runtime:fs")
    pub specifier: String,

    /// All ops in this module
    pub ops: Vec<OpSymbol>,

    /// All structs in this module
    pub structs: Vec<WeldStruct>,

    /// All enums in this module
    pub enums: Vec<WeldEnum>,

    /// ESM entry point path (e.g., "ext:runtime_fs/init.js")
    pub esm_entry_point: String,

    /// Module documentation
    pub doc: Option<String>,

    /// Error type name (if module has a custom error type)
    pub error_type: Option<String>,

    /// Error code range start (for error code allocation)
    pub error_code_start: Option<u32>,

    /// Extensibility configuration (hooks, handlers, config options)
    #[serde(default)]
    pub extensibility: ExtensibilityConfig,
}

impl WeldModule {
    /// Create a new module
    pub fn new(name: impl Into<String>, specifier: impl Into<String>) -> Self {
        let name = name.into();
        let esm_entry_point = format!("ext:{}/init.js", name);

        Self {
            name,
            specifier: specifier.into(),
            ops: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            esm_entry_point,
            doc: None,
            error_type: None,
            error_code_start: None,
            extensibility: ExtensibilityConfig::default(),
        }
    }

    /// Create a host module (e.g., "host_fs" -> "runtime:fs")
    pub fn host(name: &str) -> Self {
        let module_name = format!("host_{}", name);
        let specifier = format!("runtime:{}", name);
        Self::new(module_name, specifier)
    }

    /// Set ESM entry point
    pub fn with_esm_entry_point(mut self, path: impl Into<String>) -> Self {
        self.esm_entry_point = path.into();
        self
    }

    /// Add an op
    pub fn op(mut self, op: OpSymbol) -> Self {
        self.ops.push(op);
        self
    }

    /// Add ops
    pub fn with_ops(mut self, ops: Vec<OpSymbol>) -> Self {
        self.ops = ops;
        self
    }

    /// Add a struct
    pub fn struct_def(mut self, s: WeldStruct) -> Self {
        self.structs.push(s);
        self
    }

    /// Add structs
    pub fn with_structs(mut self, structs: Vec<WeldStruct>) -> Self {
        self.structs = structs;
        self
    }

    /// Add an enum
    pub fn enum_def(mut self, e: WeldEnum) -> Self {
        self.enums.push(e);
        self
    }

    /// Add enums
    pub fn with_enums(mut self, enums: Vec<WeldEnum>) -> Self {
        self.enums = enums;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set error type
    pub fn with_error_type(mut self, error_type: impl Into<String>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }

    /// Set error code start
    pub fn with_error_code_start(mut self, start: u32) -> Self {
        self.error_code_start = Some(start);
        self
    }

    /// Enable full extensibility (hooks + handlers)
    pub fn with_extensibility(mut self) -> Self {
        self.extensibility = ExtensibilityConfig::new();
        self
    }

    /// Set extensibility configuration
    pub fn with_extensibility_config(mut self, config: ExtensibilityConfig) -> Self {
        self.extensibility = config;
        self
    }

    /// Enable hooks only
    pub fn with_hooks(mut self) -> Self {
        self.extensibility.hooks_enabled = true;
        self
    }

    /// Enable handlers only
    pub fn with_handlers(mut self) -> Self {
        self.extensibility.handlers_enabled = true;
        self
    }

    /// Set specific ops that support hooks
    pub fn with_hookable_ops(mut self, ops: &[&str]) -> Self {
        self.extensibility.hookable_ops = ops.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Get all op names for extension! macro
    pub fn op_names(&self) -> Vec<&str> {
        self.ops.iter().map(|op| op.rust_name.as_str()).collect()
    }

    /// Get the Deno.core.ops declaration for TypeScript
    pub fn deno_core_ops_declaration(&self) -> String {
        let mut output = String::new();

        output.push_str("declare const Deno: {\n");
        output.push_str("  core: {\n");
        output.push_str("    ops: {\n");

        for op in &self.ops {
            // Build parameter list
            let params: Vec<String> = op
                .visible_params()
                .map(|p| format!("{}: {}", p.ts_name, p.ty.to_typescript()))
                .collect();

            // Determine return type
            let return_type = op.ts_return_type();

            output.push_str(&format!(
                "      {}({}): {};\n",
                op.rust_name,
                params.join(", "),
                return_type
            ));
        }

        output.push_str("    };\n");
        output.push_str("  };\n");
        output.push_str("};\n");

        output
    }

    /// Validate the module configuration
    pub fn validate(&self) -> Result<(), ModuleValidationError> {
        if self.name.is_empty() {
            return Err(ModuleValidationError::EmptyName);
        }

        if self.specifier.is_empty() {
            return Err(ModuleValidationError::EmptySpecifier);
        }

        // Check for duplicate op names
        let mut seen_ops = std::collections::HashSet::new();
        for op in &self.ops {
            if !seen_ops.insert(&op.rust_name) {
                return Err(ModuleValidationError::DuplicateOp(op.rust_name.clone()));
            }
        }

        // Check for duplicate struct names
        let mut seen_structs = std::collections::HashSet::new();
        for s in &self.structs {
            if !seen_structs.insert(&s.rust_name) {
                return Err(ModuleValidationError::DuplicateStruct(s.rust_name.clone()));
            }
        }

        Ok(())
    }
}

/// Errors that can occur during module validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleValidationError {
    /// Module name is empty
    EmptyName,
    /// Specifier is empty
    EmptySpecifier,
    /// Duplicate op name
    DuplicateOp(String),
    /// Duplicate struct name
    DuplicateStruct(String),
}

impl std::fmt::Display for ModuleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleValidationError::EmptyName => write!(f, "module name cannot be empty"),
            ModuleValidationError::EmptySpecifier => write!(f, "module specifier cannot be empty"),
            ModuleValidationError::DuplicateOp(name) => write!(f, "duplicate op: {}", name),
            ModuleValidationError::DuplicateStruct(name) => write!(f, "duplicate struct: {}", name),
        }
    }
}

impl std::error::Error for ModuleValidationError {}

/// Builder for constructing WeldModule from discovered ops
#[derive(Debug, Default)]
pub struct ModuleBuilder {
    name: Option<String>,
    specifier: Option<String>,
    ops: Vec<OpSymbol>,
    structs: Vec<WeldStruct>,
    enums: Vec<WeldEnum>,
    doc: Option<String>,
    error_type: Option<String>,
    error_code_start: Option<u32>,
}

impl ModuleBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set module name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set specifier
    pub fn specifier(mut self, specifier: impl Into<String>) -> Self {
        self.specifier = Some(specifier.into());
        self
    }

    /// Add an op
    pub fn op(mut self, op: OpSymbol) -> Self {
        self.ops.push(op);
        self
    }

    /// Add a struct
    pub fn struct_def(mut self, s: WeldStruct) -> Self {
        self.structs.push(s);
        self
    }

    /// Add an enum
    pub fn enum_def(mut self, e: WeldEnum) -> Self {
        self.enums.push(e);
        self
    }

    /// Set documentation
    pub fn doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set error type
    pub fn error_type(mut self, error_type: impl Into<String>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }

    /// Set error code start
    pub fn error_code_start(mut self, start: u32) -> Self {
        self.error_code_start = Some(start);
        self
    }

    /// Build the module
    pub fn build(self) -> Result<WeldModule, ModuleValidationError> {
        let name = self.name.ok_or(ModuleValidationError::EmptyName)?;
        let specifier = self
            .specifier
            .ok_or(ModuleValidationError::EmptySpecifier)?;

        let mut module = WeldModule::new(name, specifier)
            .with_ops(self.ops)
            .with_structs(self.structs)
            .with_enums(self.enums);

        if let Some(doc) = self.doc {
            module = module.with_doc(doc);
        }

        if let Some(error_type) = self.error_type {
            module = module.with_error_type(error_type);
        }

        if let Some(start) = self.error_code_start {
            module = module.with_error_code_start(start);
        }

        module.validate()?;
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OpParam, WeldType};

    #[test]
    fn test_module_creation() {
        let module = WeldModule::host("fs")
            .op(OpSymbol::from_rust_name("op_fs_read_text")
                .async_op()
                .param(OpParam::new("path", WeldType::string()))
                .returns(WeldType::result(
                    WeldType::string(),
                    WeldType::struct_ref("FsError"),
                )))
            .with_doc("Filesystem operations");

        assert_eq!(module.name, "host_fs");
        assert_eq!(module.specifier, "runtime:fs");
        assert_eq!(module.esm_entry_point, "ext:host_fs/init.js");
        assert_eq!(module.ops.len(), 1);
    }

    #[test]
    fn test_deno_core_ops_declaration() {
        let module = WeldModule::host("fs").op(OpSymbol::from_rust_name("op_fs_read_text")
            .async_op()
            .param(OpParam::new("path", WeldType::string()))
            .returns(WeldType::result(
                WeldType::string(),
                WeldType::struct_ref("FsError"),
            )));

        let decl = module.deno_core_ops_declaration();
        assert!(decl.contains("op_fs_read_text(path: string): Promise<string>"));
    }

    #[test]
    fn test_module_validation() {
        let module = WeldModule::new("", "runtime:test");
        assert!(module.validate().is_err());

        let module = WeldModule::new("test", "");
        assert!(module.validate().is_err());

        let module = WeldModule::new("test", "runtime:test")
            .op(OpSymbol::from_rust_name("op_test"))
            .op(OpSymbol::from_rust_name("op_test")); // Duplicate
        assert!(module.validate().is_err());
    }
}
