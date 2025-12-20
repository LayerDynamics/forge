//! Core documentation node types
//!
//! This module provides `EtchNode`, the central data structure representing
//! any documented item in the Forge framework. Similar to deno_doc's DocNode,
//! but tailored for Forge's dual Rust/TypeScript architecture.

use crate::class::ClassDef;
use crate::function::{FunctionDef, OpDef};
use crate::interface::InterfaceDef;
use crate::js_doc::EtchDoc;
use crate::r#enum::EnumDef;
use crate::type_alias::TypeAliasDef;
use crate::variable::VariableDef;
use crate::visibility::Visibility;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Source location for a documented item
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    /// Source filename
    pub filename: String,
    /// 1-indexed line number
    pub line: usize,
    /// 0-indexed column number
    pub col: usize,
    /// Byte offset in source (optional)
    #[serde(default)]
    pub byte_index: usize,
}

impl Location {
    /// Create a new location
    pub fn new(filename: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            filename: filename.into(),
            line,
            col,
            byte_index: 0,
        }
    }

    /// Create an unknown/unset location
    pub fn unknown() -> Self {
        Self {
            filename: String::new(),
            line: 0,
            col: 0,
            byte_index: 0,
        }
    }

    /// Create location with byte index
    pub fn with_byte_index(mut self, index: usize) -> Self {
        self.byte_index = index;
        self
    }

    /// Check if this location is unknown/unset (default values)
    pub fn is_unknown(&self) -> bool {
        self.filename.is_empty() && self.line == 0 && self.col == 0
    }
}

impl std::cmp::Ord for Location {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.filename.cmp(&other.filename) {
            std::cmp::Ordering::Equal => match self.line.cmp(&other.line) {
                std::cmp::Ordering::Equal => self.col.cmp(&other.col),
                ord => ord,
            },
            ord => ord,
        }
    }
}

impl std::cmp::PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Kind of documented node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EtchNodeKind {
    /// Deno op function (Rust → TypeScript bridge)
    Op,
    /// Regular TypeScript/JavaScript function
    Function,
    /// TypeScript class
    Class,
    /// TypeScript interface
    Interface,
    /// Rust struct exposed to TypeScript
    Struct,
    /// Rust/TypeScript enum
    Enum,
    /// Type alias
    TypeAlias,
    /// Variable or constant
    Variable,
    /// Namespace (module-like grouping)
    Namespace,
    /// Module documentation
    Module,
    /// Import statement
    Import,
    /// Reference to another symbol
    Reference,
}

impl EtchNodeKind {
    /// Get display name for this kind
    pub fn display_name(&self) -> &'static str {
        match self {
            EtchNodeKind::Op => "Op",
            EtchNodeKind::Function => "Function",
            EtchNodeKind::Class => "Class",
            EtchNodeKind::Interface => "Interface",
            EtchNodeKind::Struct => "Struct",
            EtchNodeKind::Enum => "Enum",
            EtchNodeKind::TypeAlias => "Type Alias",
            EtchNodeKind::Variable => "Variable",
            EtchNodeKind::Namespace => "Namespace",
            EtchNodeKind::Module => "Module",
            EtchNodeKind::Import => "Import",
            EtchNodeKind::Reference => "Reference",
        }
    }

    /// Get CSS class name for styling
    pub fn css_class(&self) -> &'static str {
        match self {
            EtchNodeKind::Op => "kind-op",
            EtchNodeKind::Function => "kind-function",
            EtchNodeKind::Class => "kind-class",
            EtchNodeKind::Interface => "kind-interface",
            EtchNodeKind::Struct => "kind-struct",
            EtchNodeKind::Enum => "kind-enum",
            EtchNodeKind::TypeAlias => "kind-type-alias",
            EtchNodeKind::Variable => "kind-variable",
            EtchNodeKind::Namespace => "kind-namespace",
            EtchNodeKind::Module => "kind-module",
            EtchNodeKind::Import => "kind-import",
            EtchNodeKind::Reference => "kind-reference",
        }
    }

    /// Get icon character for this kind
    pub fn icon(&self) -> &'static str {
        match self {
            EtchNodeKind::Op => "⚡",
            EtchNodeKind::Function => "ƒ",
            EtchNodeKind::Class => "C",
            EtchNodeKind::Interface => "I",
            EtchNodeKind::Struct => "S",
            EtchNodeKind::Enum => "E",
            EtchNodeKind::TypeAlias => "T",
            EtchNodeKind::Variable => "V",
            EtchNodeKind::Namespace => "N",
            EtchNodeKind::Module => "M",
            EtchNodeKind::Import => "→",
            EtchNodeKind::Reference => "↗",
        }
    }
}

/// Namespace definition containing child nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDef {
    /// Child elements in this namespace
    pub elements: Vec<Arc<EtchNode>>,
}

/// Module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDef {
    /// Module specifier (e.g., "runtime:fs")
    pub specifier: String,
    /// Module name (e.g., "host_fs")
    pub name: String,
    /// Child elements in this module
    pub elements: Vec<Arc<EtchNode>>,
}

/// Struct definition (from Rust via forge-weld)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Rust struct name
    pub rust_name: String,
    /// TypeScript interface name
    pub ts_name: String,
    /// Fields
    pub fields: Vec<StructFieldDef>,
    /// Type parameters
    pub type_params: Vec<String>,
}

/// Struct field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructFieldDef {
    /// Field name
    pub name: String,
    /// TypeScript name (camelCase)
    pub ts_name: String,
    /// Field type as TypeScript string
    pub ts_type: String,
    /// Whether field is optional
    pub optional: bool,
    /// Whether field is readonly
    pub readonly: bool,
    /// Field documentation
    pub doc: Option<String>,
}

/// Import definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDef {
    /// Source module
    pub src: String,
    /// Imported name (None for namespace import)
    pub imported: Option<String>,
}

/// Reference to another symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceDef {
    /// Target location
    pub target: Location,
}

/// Specific definition for each node kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EtchNodeDef {
    /// Deno op function
    #[serde(rename_all = "camelCase")]
    Op { op_def: OpDef },

    /// Regular function
    #[serde(rename_all = "camelCase")]
    Function { function_def: FunctionDef },

    /// Class
    #[serde(rename_all = "camelCase")]
    Class { class_def: ClassDef },

    /// Interface
    #[serde(rename_all = "camelCase")]
    Interface { interface_def: InterfaceDef },

    /// Struct (from Rust)
    #[serde(rename_all = "camelCase")]
    Struct { struct_def: StructDef },

    /// Enum
    #[serde(rename_all = "camelCase")]
    Enum { enum_def: EnumDef },

    /// Type alias
    #[serde(rename_all = "camelCase")]
    TypeAlias { type_alias_def: TypeAliasDef },

    /// Variable
    #[serde(rename_all = "camelCase")]
    Variable { variable_def: VariableDef },

    /// Namespace
    #[serde(rename_all = "camelCase")]
    Namespace { namespace_def: NamespaceDef },

    /// Module
    #[serde(rename_all = "camelCase")]
    Module { module_def: ModuleDef },

    /// Import
    #[serde(rename_all = "camelCase")]
    Import { import_def: ImportDef },

    /// Reference
    #[serde(rename_all = "camelCase")]
    Reference { reference_def: ReferenceDef },

    /// Module-level documentation (no definition)
    ModuleDoc,
}

/// Core documentation node - represents any documented item
///
/// This is the central type in forge-etch, analogous to deno_doc's DocNode.
/// Every documented symbol (function, class, interface, etc.) is represented
/// as an EtchNode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtchNode {
    /// Symbol name
    pub name: String,

    /// Whether this is the default export
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_default: Option<bool>,

    /// Source location
    pub location: Location,

    /// Visibility (public, private, internal)
    pub visibility: Visibility,

    /// Documentation (JSDoc + description)
    #[serde(skip_serializing_if = "EtchDoc::is_empty", default)]
    pub doc: EtchDoc,

    /// Node-specific definition
    #[serde(flatten)]
    pub def: EtchNodeDef,

    /// Module this belongs to (e.g., "runtime:fs")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub module: Option<String>,
}

impl Default for EtchNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_default: None,
            location: Location::default(),
            visibility: Visibility::Public,
            doc: EtchDoc::default(),
            def: EtchNodeDef::ModuleDoc,
            module: None,
        }
    }
}

impl EtchNode {
    /// Create a new module documentation node
    pub fn module_doc(location: Location, doc: EtchDoc) -> Self {
        Self {
            name: String::new(),
            is_default: None,
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::ModuleDoc,
            module: None,
        }
    }

    /// Create an op node
    pub fn op(name: impl Into<String>, location: Location, doc: EtchDoc, op_def: OpDef) -> Self {
        Self {
            name: name.into(),
            is_default: Some(false),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Op { op_def },
            module: None,
        }
    }

    /// Create a function node
    pub fn function(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        function_def: FunctionDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Function { function_def },
            module: None,
        }
    }

    /// Create a class node
    pub fn class(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        class_def: ClassDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Class { class_def },
            module: None,
        }
    }

    /// Create an interface node
    pub fn interface(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        interface_def: InterfaceDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Interface { interface_def },
            module: None,
        }
    }

    /// Create a struct node (from Rust)
    pub fn rust_struct(
        name: impl Into<String>,
        location: Location,
        doc: EtchDoc,
        struct_def: StructDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(false),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Struct { struct_def },
            module: None,
        }
    }

    /// Create an enum node
    pub fn r#enum(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        enum_def: EnumDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Enum { enum_def },
            module: None,
        }
    }

    /// Create a type alias node
    pub fn type_alias(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        type_alias_def: TypeAliasDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::TypeAlias { type_alias_def },
            module: None,
        }
    }

    /// Create a variable node
    pub fn variable(
        name: impl Into<String>,
        is_default: bool,
        location: Location,
        doc: EtchDoc,
        variable_def: VariableDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(is_default),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Variable { variable_def },
            module: None,
        }
    }

    /// Create a namespace node
    pub fn namespace(
        name: impl Into<String>,
        location: Location,
        doc: EtchDoc,
        namespace_def: NamespaceDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: Some(false),
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Namespace { namespace_def },
            module: None,
        }
    }

    /// Create a module node
    pub fn module(
        name: impl Into<String>,
        location: Location,
        doc: EtchDoc,
        module_def: ModuleDef,
    ) -> Self {
        Self {
            name: name.into(),
            is_default: None,
            location,
            visibility: Visibility::Public,
            doc,
            def: EtchNodeDef::Module { module_def },
            module: None,
        }
    }

    /// Set the module this node belongs to
    pub fn in_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Get the kind of this node
    pub fn kind(&self) -> EtchNodeKind {
        match &self.def {
            EtchNodeDef::Op { .. } => EtchNodeKind::Op,
            EtchNodeDef::Function { .. } => EtchNodeKind::Function,
            EtchNodeDef::Class { .. } => EtchNodeKind::Class,
            EtchNodeDef::Interface { .. } => EtchNodeKind::Interface,
            EtchNodeDef::Struct { .. } => EtchNodeKind::Struct,
            EtchNodeDef::Enum { .. } => EtchNodeKind::Enum,
            EtchNodeDef::TypeAlias { .. } => EtchNodeKind::TypeAlias,
            EtchNodeDef::Variable { .. } => EtchNodeKind::Variable,
            EtchNodeDef::Namespace { .. } => EtchNodeKind::Namespace,
            EtchNodeDef::Module { .. } => EtchNodeKind::Module,
            EtchNodeDef::Import { .. } => EtchNodeKind::Import,
            EtchNodeDef::Reference { .. } => EtchNodeKind::Reference,
            EtchNodeDef::ModuleDoc => EtchNodeKind::Module,
        }
    }

    /// Get display name (considers default export names)
    pub fn get_name(&self) -> &str {
        // For certain types, check for internal def_name
        match &self.def {
            EtchNodeDef::Class { class_def } => class_def.def_name.as_deref().unwrap_or(&self.name),
            EtchNodeDef::Function { function_def } => {
                function_def.def_name.as_deref().unwrap_or(&self.name)
            }
            EtchNodeDef::Interface { interface_def } => {
                interface_def.def_name.as_deref().unwrap_or(&self.name)
            }
            _ => &self.name,
        }
    }

    /// Get the op definition if this is an op node
    pub fn op_def(&self) -> Option<&OpDef> {
        match &self.def {
            EtchNodeDef::Op { op_def } => Some(op_def),
            _ => None,
        }
    }

    /// Get the function definition if this is a function node
    pub fn function_def(&self) -> Option<&FunctionDef> {
        match &self.def {
            EtchNodeDef::Function { function_def } => Some(function_def),
            _ => None,
        }
    }

    /// Get the class definition if this is a class node
    pub fn class_def(&self) -> Option<&ClassDef> {
        match &self.def {
            EtchNodeDef::Class { class_def } => Some(class_def),
            _ => None,
        }
    }

    /// Get the interface definition if this is an interface node
    pub fn interface_def(&self) -> Option<&InterfaceDef> {
        match &self.def {
            EtchNodeDef::Interface { interface_def } => Some(interface_def),
            _ => None,
        }
    }

    /// Get the struct definition if this is a struct node
    pub fn struct_def(&self) -> Option<&StructDef> {
        match &self.def {
            EtchNodeDef::Struct { struct_def } => Some(struct_def),
            _ => None,
        }
    }

    /// Get the enum definition if this is an enum node
    pub fn enum_def(&self) -> Option<&EnumDef> {
        match &self.def {
            EtchNodeDef::Enum { enum_def } => Some(enum_def),
            _ => None,
        }
    }

    /// Get the type alias definition if this is a type alias node
    pub fn type_alias_def(&self) -> Option<&TypeAliasDef> {
        match &self.def {
            EtchNodeDef::TypeAlias { type_alias_def } => Some(type_alias_def),
            _ => None,
        }
    }

    /// Get the variable definition if this is a variable node
    pub fn variable_def(&self) -> Option<&VariableDef> {
        match &self.def {
            EtchNodeDef::Variable { variable_def } => Some(variable_def),
            _ => None,
        }
    }

    /// Get the namespace definition if this is a namespace node
    pub fn namespace_def(&self) -> Option<&NamespaceDef> {
        match &self.def {
            EtchNodeDef::Namespace { namespace_def } => Some(namespace_def),
            _ => None,
        }
    }

    /// Get the module definition if this is a module node
    pub fn module_def(&self) -> Option<&ModuleDef> {
        match &self.def {
            EtchNodeDef::Module { module_def } => Some(module_def),
            _ => None,
        }
    }

    /// Check if this node has documentation
    pub fn has_doc(&self) -> bool {
        !self.doc.is_empty()
    }

    /// Get short description (first sentence of doc)
    pub fn short_description(&self) -> Option<&str> {
        self.doc.description.as_ref().map(|d| {
            // Take first sentence or first 100 chars
            if let Some(period_idx) = d.find('.') {
                &d[..=period_idx]
            } else if d.len() > 100 {
                &d[..100]
            } else {
                d.as_str()
            }
        })
    }

    /// Generate TypeScript signature for this node
    pub fn to_typescript_signature(&self) -> String {
        match &self.def {
            EtchNodeDef::Op { op_def } => {
                let async_prefix = if op_def.is_async { "async " } else { "" };
                format!(
                    "{}function {}",
                    async_prefix,
                    op_def.to_typescript_signature()
                )
            }
            EtchNodeDef::Function { function_def } => {
                function_def.to_typescript_signature(&self.name)
            }
            EtchNodeDef::Class { class_def } => {
                let type_params = if class_def.type_params.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        class_def
                            .type_params
                            .iter()
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!("class {}{}", self.name, type_params)
            }
            EtchNodeDef::Interface { .. } => {
                format!("interface {}", self.name)
            }
            EtchNodeDef::Struct { .. } => {
                format!("interface {}", self.name)
            }
            EtchNodeDef::Enum { enum_def } => {
                let const_prefix = if enum_def.is_const { "const " } else { "" };
                format!("{}enum {}", const_prefix, self.name)
            }
            EtchNodeDef::TypeAlias { type_alias_def } => type_alias_def.to_typescript(&self.name),
            EtchNodeDef::Variable { variable_def } => variable_def.to_typescript(&self.name),
            EtchNodeDef::Namespace { .. } => {
                format!("namespace {}", self.name)
            }
            EtchNodeDef::Module { .. } => {
                format!("module \"{}\"", self.name)
            }
            EtchNodeDef::Import { .. } | EtchNodeDef::Reference { .. } | EtchNodeDef::ModuleDoc => {
                self.name.clone()
            }
        }
    }

    /// Generate TypeScript signature for this node, returning None for nodes without signatures
    pub fn to_typescript_signature_opt(&self) -> Option<String> {
        match &self.def {
            EtchNodeDef::Import { .. } | EtchNodeDef::Reference { .. } | EtchNodeDef::ModuleDoc => {
                None
            }
            _ => Some(self.to_typescript_signature()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::OpDef;

    #[test]
    fn test_location_ordering() {
        let loc1 = Location::new("a.ts", 1, 0);
        let loc2 = Location::new("a.ts", 2, 0);
        let loc3 = Location::new("b.ts", 1, 0);

        assert!(loc1 < loc2);
        assert!(loc2 < loc3);
    }

    #[test]
    fn test_etch_node_kind() {
        assert_eq!(EtchNodeKind::Op.display_name(), "Op");
        assert_eq!(EtchNodeKind::Function.css_class(), "kind-function");
        assert_eq!(EtchNodeKind::Class.icon(), "C");
    }

    #[test]
    fn test_etch_node_creation() {
        let op = OpDef {
            rust_name: "op_fs_read_text".to_string(),
            ts_name: "readTextFile".to_string(),
            is_async: true,
            params: vec![],
            return_type: "Promise<string>".to_string(),
            op_attrs: Default::default(),
            return_type_def: None,
            type_params: vec![],
            can_throw: false,
            permissions: None,
        };

        let node = EtchNode::op(
            "readTextFile",
            Location::new("fs.ts", 10, 0),
            EtchDoc::default(),
            op,
        );

        assert_eq!(node.name, "readTextFile");
        assert_eq!(node.kind(), EtchNodeKind::Op);
        assert!(node.op_def().is_some());
    }
}
