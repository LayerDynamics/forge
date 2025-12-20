//! Rust/forge-weld extraction utilities
//!
//! This module provides utilities for extracting documentation
//! from forge-weld IR (intermediate representation).

use crate::node::EtchNode;
use crate::parser::weld_module_to_nodes;
use forge_weld::ir::{OpSymbol, WeldEnum, WeldModule, WeldStruct};

/// forge-weld documentation extractor
///
/// Extracts documentation from forge-weld IR types.
pub struct WeldExtractor {
    /// Whether to include private ops
    include_private: bool,
}

impl Default for WeldExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl WeldExtractor {
    /// Create a new extractor
    pub fn new() -> Self {
        Self {
            include_private: false,
        }
    }

    /// Include private ops
    pub fn include_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Extract documentation from a WeldModule
    pub fn extract_module(&self, module: &WeldModule) -> Vec<EtchNode> {
        weld_module_to_nodes(module)
    }

    /// Extract documentation from ops only
    pub fn extract_ops(&self, ops: &[OpSymbol], specifier: &str) -> Vec<EtchNode> {
        ops.iter()
            .map(|op| crate::parser::op_symbol_to_node_pub(op, specifier))
            .collect()
    }

    /// Extract documentation from structs only
    pub fn extract_structs(&self, structs: &[WeldStruct], specifier: &str) -> Vec<EtchNode> {
        structs
            .iter()
            .map(|s| crate::parser::weld_struct_to_node_pub(s, specifier))
            .collect()
    }

    /// Extract documentation from enums only
    pub fn extract_enums(&self, enums: &[WeldEnum], specifier: &str) -> Vec<EtchNode> {
        enums
            .iter()
            .map(|e| crate::parser::weld_enum_to_node_pub(e, specifier))
            .collect()
    }
}

/// Get type information from a WeldModule
pub fn get_type_exports(module: &WeldModule) -> Vec<TypeExport> {
    let mut exports = Vec::new();

    for s in &module.structs {
        exports.push(TypeExport {
            name: s.ts_name.clone(),
            kind: TypeExportKind::Interface,
            doc: s.doc.clone(),
        });
    }

    for e in &module.enums {
        exports.push(TypeExport {
            name: e.ts_name.clone(),
            kind: TypeExportKind::Enum,
            doc: e.doc.clone(),
        });
    }

    exports
}

/// A type export from a module
#[derive(Debug, Clone)]
pub struct TypeExport {
    /// The exported name
    pub name: String,
    /// The kind of type
    pub kind: TypeExportKind,
    /// Documentation
    pub doc: Option<String>,
}

/// Kind of type export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeExportKind {
    /// An interface/struct
    Interface,
    /// An enum
    Enum,
    /// A type alias
    TypeAlias,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weld_extractor() {
        let extractor = WeldExtractor::new();
        // Would need a mock WeldModule to test fully
        assert!(!extractor.include_private);
    }
}
