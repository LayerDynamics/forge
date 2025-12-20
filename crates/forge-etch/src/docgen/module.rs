//! Module documentation structure
//!
//! This module provides the ModuleDoc type for representing
//! documentation at the module level.

use crate::js_doc::EtchDoc;
use crate::node::EtchNode;
use serde::{Deserialize, Serialize};

/// Documentation for a single module/file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleDoc {
    /// Module path/specifier
    pub path: String,
    /// Module-level documentation
    pub doc: Option<EtchDoc>,
    /// Symbols defined in this module
    pub symbols: Vec<EtchNode>,
    /// Imports from other modules
    pub imports: Vec<ImportInfo>,
    /// Exports (including re-exports)
    pub exports: Vec<ExportInfo>,
}

impl ModuleDoc {
    /// Create a new module doc
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set module documentation
    pub fn with_doc(mut self, doc: EtchDoc) -> Self {
        self.doc = Some(doc);
        self
    }

    /// Add a symbol
    pub fn add_symbol(&mut self, symbol: EtchNode) {
        self.symbols.push(symbol);
    }

    /// Add an import
    pub fn add_import(&mut self, import: ImportInfo) {
        self.imports.push(import);
    }

    /// Add an export
    pub fn add_export(&mut self, export: ExportInfo) {
        self.exports.push(export);
    }

    /// Get exported symbol names
    pub fn exported_names(&self) -> Vec<&str> {
        self.exports.iter().map(|e| e.name.as_str()).collect()
    }

    /// Check if a symbol is exported
    pub fn is_exported(&self, name: &str) -> bool {
        self.exports
            .iter()
            .any(|e| e.name == name || e.local_name.as_deref() == Some(name))
    }
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    /// Imported name
    pub name: String,
    /// Local alias (if renamed)
    pub alias: Option<String>,
    /// Source module
    pub source: String,
    /// Whether this is a type-only import
    pub type_only: bool,
    /// Whether this is a namespace import (import * as)
    pub namespace: bool,
}

impl ImportInfo {
    /// Create a named import
    pub fn named(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            alias: None,
            source: source.into(),
            type_only: false,
            namespace: false,
        }
    }

    /// Create a namespace import
    pub fn namespace(alias: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: "*".to_string(),
            alias: Some(alias.into()),
            source: source.into(),
            type_only: false,
            namespace: true,
        }
    }

    /// Mark as type-only
    pub fn as_type_only(mut self) -> Self {
        self.type_only = true;
        self
    }

    /// Set alias
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }
}

/// Export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    /// Exported name
    pub name: String,
    /// Local name (if different)
    pub local_name: Option<String>,
    /// Source module (for re-exports)
    pub source: Option<String>,
    /// Whether this is a default export
    pub is_default: bool,
    /// Whether this is a type-only export
    pub type_only: bool,
}

impl ExportInfo {
    /// Create a named export
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            source: None,
            is_default: false,
            type_only: false,
        }
    }

    /// Create a default export
    pub fn default_export(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            source: None,
            is_default: true,
            type_only: false,
        }
    }

    /// Create a re-export
    pub fn re_export(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            source: Some(source.into()),
            is_default: false,
            type_only: false,
        }
    }

    /// Set local name
    pub fn with_local_name(mut self, local: impl Into<String>) -> Self {
        self.local_name = Some(local.into());
        self
    }

    /// Mark as type-only
    pub fn as_type_only(mut self) -> Self {
        self.type_only = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_doc() {
        let mut module = ModuleDoc::new("runtime:fs");
        module.add_export(ExportInfo::named("readFile"));
        module.add_export(ExportInfo::named("writeFile"));

        assert!(module.is_exported("readFile"));
        assert!(module.is_exported("writeFile"));
        assert!(!module.is_exported("deleteFile"));
    }

    #[test]
    fn test_import_info() {
        let import = ImportInfo::named("readFile", "runtime:fs")
            .with_alias("read")
            .as_type_only();

        assert_eq!(import.name, "readFile");
        assert_eq!(import.alias, Some("read".to_string()));
        assert!(import.type_only);
    }

    #[test]
    fn test_export_info() {
        let export = ExportInfo::re_export("readFile", "./fs").with_local_name("internalRead");

        assert_eq!(export.name, "readFile");
        assert_eq!(export.source, Some("./fs".to_string()));
        assert_eq!(export.local_name, Some("internalRead".to_string()));
    }
}
