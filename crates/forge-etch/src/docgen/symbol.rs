//! Symbol documentation structure
//!
//! This module provides the SymbolDoc type for representing
//! documentation for individual symbols.

use crate::js_doc::EtchDoc;
use crate::node::{EtchNode, EtchNodeKind, Location};
use serde::{Deserialize, Serialize};

/// Documentation for a single symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDoc {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: EtchNodeKind,
    /// Full documentation
    pub doc: EtchDoc,
    /// Symbol location
    pub location: Location,
    /// TypeScript signature
    pub signature: Option<String>,
    /// Examples from documentation
    pub examples: Vec<String>,
    /// Related symbols
    pub see_also: Vec<String>,
    /// Whether deprecated
    pub deprecated: bool,
    /// Deprecation message
    pub deprecation_message: Option<String>,
    /// Whether experimental
    pub experimental: bool,
    /// Category (from @category tag)
    pub category: Option<String>,
    /// Since version (from @since tag)
    pub since: Option<String>,
}

impl SymbolDoc {
    /// Create from an EtchNode
    pub fn from_node(node: &EtchNode) -> Self {
        Self {
            name: node.name.clone(),
            kind: node.kind(),
            doc: node.doc.clone(),
            location: node.location.clone(),
            signature: Some(node.to_typescript_signature()),
            examples: node
                .doc
                .examples()
                .filter_map(|t| {
                    if let crate::js_doc::JsDocTag::Example { doc, .. } = t {
                        Some(doc.clone())
                    } else {
                        None
                    }
                })
                .collect(),
            see_also: node.doc.see_also().map(|s| s.to_string()).collect(),
            deprecated: node.doc.is_deprecated(),
            deprecation_message: node.doc.deprecated().and_then(|t| {
                if let crate::js_doc::JsDocTag::Deprecated { doc } = t {
                    doc.clone()
                } else {
                    None
                }
            }),
            experimental: node.doc.is_experimental(),
            category: node.doc.category().map(|s| s.to_string()),
            since: node.doc.since().map(|s| s.to_string()),
        }
    }

    /// Get display name (formatted for documentation)
    pub fn display_name(&self) -> String {
        match self.kind {
            EtchNodeKind::Function | EtchNodeKind::Op => format!("{}()", self.name),
            EtchNodeKind::Class => format!("class {}", self.name),
            EtchNodeKind::Interface | EtchNodeKind::Struct => format!("interface {}", self.name),
            EtchNodeKind::Enum => format!("enum {}", self.name),
            EtchNodeKind::TypeAlias => format!("type {}", self.name),
            _ => self.name.clone(),
        }
    }

    /// Get anchor ID for HTML/markdown
    pub fn anchor_id(&self) -> String {
        self.name.to_lowercase().replace(' ', "-")
    }

    /// Get badge text (e.g., "deprecated", "experimental")
    pub fn badges(&self) -> Vec<SymbolBadge> {
        let mut badges = Vec::new();

        if self.deprecated {
            badges.push(SymbolBadge {
                text: "deprecated".to_string(),
                kind: BadgeKind::Deprecated,
            });
        }

        if self.experimental {
            badges.push(SymbolBadge {
                text: "experimental".to_string(),
                kind: BadgeKind::Experimental,
            });
        }

        badges
    }
}

/// A badge to display on a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolBadge {
    /// Badge text
    pub text: String,
    /// Badge kind
    pub kind: BadgeKind,
}

/// Kind of badge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeKind {
    /// Deprecated symbol
    Deprecated,
    /// Experimental symbol
    Experimental,
    /// New symbol
    New,
    /// Beta symbol
    Beta,
    /// Internal symbol
    Internal,
}

impl BadgeKind {
    /// Get CSS class for styling
    pub fn css_class(&self) -> &'static str {
        match self {
            BadgeKind::Deprecated => "badge-deprecated",
            BadgeKind::Experimental => "badge-experimental",
            BadgeKind::New => "badge-new",
            BadgeKind::Beta => "badge-beta",
            BadgeKind::Internal => "badge-internal",
        }
    }

    /// Get display text
    pub fn display(&self) -> &'static str {
        match self {
            BadgeKind::Deprecated => "Deprecated",
            BadgeKind::Experimental => "Experimental",
            BadgeKind::New => "New",
            BadgeKind::Beta => "Beta",
            BadgeKind::Internal => "Internal",
        }
    }
}

/// Summary of a symbol for index/search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSummary {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: EtchNodeKind,
    /// Short description (first sentence)
    pub description: Option<String>,
    /// Module where defined
    pub module: Option<String>,
    /// URL path
    pub url: String,
}

impl SymbolSummary {
    /// Create from a node
    pub fn from_node(node: &EtchNode, base_url: &str) -> Self {
        Self {
            name: node.name.clone(),
            kind: node.kind(),
            description: node.doc.summary(),
            module: node.module.clone(),
            url: format!("{}#{}", base_url, node.name.to_lowercase()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::EtchNodeDef;
    use crate::visibility::Visibility;

    #[test]
    fn test_symbol_doc() {
        let node = EtchNode {
            name: "readFile".to_string(),
            is_default: None,
            location: Location::unknown(),
            visibility: Visibility::Public,
            doc: EtchDoc::default(),
            def: EtchNodeDef::Function {
                function_def: crate::function::FunctionDef::default(),
            },
            module: None,
        };

        let sym_doc = SymbolDoc::from_node(&node);
        assert_eq!(sym_doc.name, "readFile");
        assert_eq!(sym_doc.kind, EtchNodeKind::Function);
        assert_eq!(sym_doc.display_name(), "readFile()");
    }

    #[test]
    fn test_badge_kind() {
        assert_eq!(BadgeKind::Deprecated.css_class(), "badge-deprecated");
        assert_eq!(BadgeKind::Experimental.display(), "Experimental");
    }
}
