//! Extension documentation structure
//!
//! This module provides the ExtensionDoc type which represents
//! complete documentation for a Forge extension.

use crate::js_doc::EtchDoc;
use crate::node::{EtchNode, EtchNodeDef, EtchNodeKind};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Complete documentation for a Forge extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDoc {
    /// Internal name (e.g., "host_fs")
    pub name: String,
    /// Module specifier (e.g., "runtime:fs")
    pub specifier: String,
    /// Display title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// All documented nodes
    pub nodes: Vec<EtchNode>,
    /// Module-level documentation
    pub module_doc: Option<EtchDoc>,
}

impl ExtensionDoc {
    /// Create a new extension doc
    pub fn new(
        name: impl Into<String>,
        specifier: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            specifier: specifier.into(),
            title: title.into(),
            description: None,
            nodes: vec![],
            module_doc: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set nodes
    pub fn with_nodes(mut self, nodes: Vec<EtchNode>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Set module doc
    pub fn with_module_doc(mut self, doc: EtchDoc) -> Self {
        self.module_doc = Some(doc);
        self
    }

    /// Get nodes grouped by kind
    pub fn nodes_by_kind(&self) -> IndexMap<EtchNodeKind, Vec<&EtchNode>> {
        let mut result: IndexMap<EtchNodeKind, Vec<&EtchNode>> = IndexMap::new();

        for node in &self.nodes {
            let kind = node.kind();
            result.entry(kind).or_default().push(node);
        }

        // Sort each group alphabetically
        for nodes in result.values_mut() {
            nodes.sort_by(|a, b| a.name.cmp(&b.name));
        }

        result
    }

    /// Get all functions (including ops)
    pub fn functions(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::Function { .. } | EtchNodeDef::Op { .. }))
            .collect()
    }

    /// Get all ops
    pub fn ops(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::Op { .. }))
            .collect()
    }

    /// Get all classes
    pub fn classes(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::Class { .. }))
            .collect()
    }

    /// Get all interfaces
    pub fn interfaces(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| {
                matches!(
                    n.def,
                    EtchNodeDef::Interface { .. } | EtchNodeDef::Struct { .. }
                )
            })
            .collect()
    }

    /// Get all type aliases
    pub fn type_aliases(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::TypeAlias { .. }))
            .collect()
    }

    /// Get all enums
    pub fn enums(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::Enum { .. }))
            .collect()
    }

    /// Get all variables
    pub fn variables(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n.def, EtchNodeDef::Variable { .. }))
            .collect()
    }

    /// Get a node by name
    pub fn get_node(&self, name: &str) -> Option<&EtchNode> {
        self.nodes.iter().find(|n| n.name == name)
    }

    /// Check if this extension has any documented symbols
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the number of documented symbols
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Get the effective description (from module_doc or description field)
    pub fn effective_description(&self) -> Option<&str> {
        self.module_doc
            .as_ref()
            .and_then(|d| d.description.as_deref())
            .or(self.description.as_deref())
    }

    /// Get categories from node @category tags
    pub fn categories(&self) -> Vec<String> {
        let mut categories = std::collections::HashSet::new();

        for node in &self.nodes {
            if let Some(cat) = node.doc.category() {
                categories.insert(cat.to_string());
            }
        }

        let mut result: Vec<_> = categories.into_iter().collect();
        result.sort();
        result
    }

    /// Get nodes by category
    pub fn nodes_by_category(&self) -> IndexMap<String, Vec<&EtchNode>> {
        let mut result: IndexMap<String, Vec<&EtchNode>> = IndexMap::new();

        // Add "Uncategorized" for nodes without a category
        for node in &self.nodes {
            let category = node
                .doc
                .category()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Uncategorized".to_string());
            result.entry(category).or_default().push(node);
        }

        // Sort nodes within each category
        for nodes in result.values_mut() {
            nodes.sort_by(|a, b| a.name.cmp(&b.name));
        }

        result
    }

    /// Generate a slug for URLs
    pub fn slug(&self) -> String {
        self.specifier
            .replace(":", "-")
            .replace("/", "-")
            .to_lowercase()
    }

    /// Get deprecated nodes
    pub fn deprecated_nodes(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| n.doc.is_deprecated())
            .collect()
    }

    /// Get experimental nodes
    pub fn experimental_nodes(&self) -> Vec<&EtchNode> {
        self.nodes
            .iter()
            .filter(|n| n.doc.is_experimental())
            .collect()
    }
}

/// Summary statistics for an extension
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionStats {
    /// Number of functions
    pub functions: usize,
    /// Number of ops
    pub ops: usize,
    /// Number of classes
    pub classes: usize,
    /// Number of interfaces
    pub interfaces: usize,
    /// Number of type aliases
    pub type_aliases: usize,
    /// Number of enums
    pub enums: usize,
    /// Number of variables
    pub variables: usize,
    /// Number of deprecated items
    pub deprecated: usize,
    /// Number of experimental items
    pub experimental: usize,
}

impl ExtensionStats {
    /// Calculate stats from an extension doc
    pub fn from_extension(ext: &ExtensionDoc) -> Self {
        Self {
            functions: ext.functions().len(),
            ops: ext.ops().len(),
            classes: ext.classes().len(),
            interfaces: ext.interfaces().len(),
            type_aliases: ext.type_aliases().len(),
            enums: ext.enums().len(),
            variables: ext.variables().len(),
            deprecated: ext.deprecated_nodes().len(),
            experimental: ext.experimental_nodes().len(),
        }
    }

    /// Total number of symbols
    pub fn total(&self) -> usize {
        self.functions
            + self.classes
            + self.interfaces
            + self.type_aliases
            + self.enums
            + self.variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Location;
    use crate::visibility::Visibility;

    fn make_node(name: &str, def: EtchNodeDef) -> EtchNode {
        EtchNode {
            name: name.to_string(),
            is_default: None,
            location: Location::unknown(),
            visibility: Visibility::Public,
            doc: EtchDoc::default(),
            def,
            module: None,
        }
    }

    #[test]
    fn test_extension_doc() {
        let mut ext = ExtensionDoc::new("host_fs", "runtime:fs", "File System");

        let func_def = crate::function::FunctionDef::default();
        ext.nodes.push(make_node(
            "readFile",
            EtchNodeDef::Function {
                function_def: func_def,
            },
        ));

        let iface_def = crate::interface::InterfaceDef::default();
        ext.nodes.push(make_node(
            "FileStat",
            EtchNodeDef::Interface {
                interface_def: iface_def,
            },
        ));

        assert_eq!(ext.len(), 2);
        assert_eq!(ext.functions().len(), 1);
        assert_eq!(ext.interfaces().len(), 1);
        assert_eq!(ext.slug(), "runtime-fs");
    }

    #[test]
    fn test_extension_stats() {
        let ext = ExtensionDoc::new("host_fs", "runtime:fs", "File System");
        let stats = ExtensionStats::from_extension(&ext);
        assert_eq!(stats.total(), 0);
    }
}
