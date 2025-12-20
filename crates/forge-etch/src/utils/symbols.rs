//! Symbol resolution and tracking
//!
//! This module provides utilities for tracking symbol definitions and references
//! across TypeScript files. It's used to:
//! - Build hyperlinks in documentation
//! - Resolve type references
//! - Track imports and exports

use crate::node::Location;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reference to a symbol
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolRef {
    /// The symbol name
    pub name: String,
    /// The module where the symbol is defined (if known)
    pub module: Option<String>,
    /// The kind of symbol
    pub kind: SymbolKind,
}

impl SymbolRef {
    /// Create a new symbol reference
    pub fn new(name: impl Into<String>, kind: SymbolKind) -> Self {
        Self {
            name: name.into(),
            module: None,
            kind,
        }
    }

    /// Create a symbol reference with a module
    pub fn with_module(
        name: impl Into<String>,
        module: impl Into<String>,
        kind: SymbolKind,
    ) -> Self {
        Self {
            name: name.into(),
            module: Some(module.into()),
            kind,
        }
    }

    /// Create a type reference
    pub fn type_ref(name: impl Into<String>) -> Self {
        Self::new(name, SymbolKind::TypeAlias)
    }

    /// Create a function reference
    pub fn function_ref(name: impl Into<String>) -> Self {
        Self::new(name, SymbolKind::Function)
    }

    /// Create a class reference
    pub fn class_ref(name: impl Into<String>) -> Self {
        Self::new(name, SymbolKind::Class)
    }

    /// Get the fully qualified name
    pub fn qualified_name(&self) -> String {
        match &self.module {
            Some(m) => format!("{}:{}", m, self.name),
            None => self.name.clone(),
        }
    }

    /// Generate a documentation link for this symbol
    pub fn doc_link(&self, base_url: &str) -> String {
        let anchor = self.name.to_lowercase().replace(' ', "-");
        match &self.module {
            Some(m) => format!("{}/{}/#{}", base_url, m.replace(':', "/"), anchor),
            None => format!("{}#{}", base_url, anchor),
        }
    }
}

/// The kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SymbolKind {
    /// A function or method
    Function,
    /// An op (Rust→TypeScript bridge function)
    Op,
    /// A class
    Class,
    /// An interface
    Interface,
    /// A type alias
    TypeAlias,
    /// An enum
    Enum,
    /// A variable (const, let, var)
    Variable,
    /// A namespace or module
    Namespace,
    /// A property
    Property,
    /// A method
    Method,
    /// A parameter
    Parameter,
    /// An import
    Import,
    /// Unknown kind
    Unknown,
}

impl SymbolKind {
    /// Get the display string for this kind
    pub fn display(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Op => "op",
            SymbolKind::Class => "class",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeAlias => "type",
            SymbolKind::Enum => "enum",
            SymbolKind::Variable => "variable",
            SymbolKind::Namespace => "namespace",
            SymbolKind::Property => "property",
            SymbolKind::Method => "method",
            SymbolKind::Parameter => "parameter",
            SymbolKind::Import => "import",
            SymbolKind::Unknown => "unknown",
        }
    }

    /// Get the icon class for documentation
    pub fn icon_class(&self) -> &'static str {
        match self {
            SymbolKind::Function | SymbolKind::Op => "icon-function",
            SymbolKind::Class => "icon-class",
            SymbolKind::Interface => "icon-interface",
            SymbolKind::TypeAlias => "icon-type",
            SymbolKind::Enum => "icon-enum",
            SymbolKind::Variable => "icon-variable",
            SymbolKind::Namespace => "icon-namespace",
            SymbolKind::Property => "icon-property",
            SymbolKind::Method => "icon-method",
            SymbolKind::Parameter => "icon-parameter",
            SymbolKind::Import => "icon-import",
            SymbolKind::Unknown => "icon-unknown",
        }
    }
}

/// A symbol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDef {
    /// The symbol name
    pub name: String,
    /// The kind of symbol
    pub kind: SymbolKind,
    /// The location where the symbol is defined
    pub location: Location,
    /// The module specifier where this symbol is defined
    pub module: String,
    /// Whether this is exported
    pub exported: bool,
    /// Whether this is a default export
    pub is_default: bool,
    /// Documentation summary
    pub doc_summary: Option<String>,
}

impl SymbolDef {
    /// Create a new symbol definition
    pub fn new(
        name: impl Into<String>,
        kind: SymbolKind,
        location: Location,
        module: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            location,
            module: module.into(),
            exported: false,
            is_default: false,
            doc_summary: None,
        }
    }

    /// Mark as exported
    pub fn as_exported(mut self) -> Self {
        self.exported = true;
        self
    }

    /// Mark as default export
    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self.exported = true;
        self
    }

    /// Set documentation summary
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc_summary = Some(doc.into());
        self
    }

    /// Convert to a SymbolRef
    pub fn to_ref(&self) -> SymbolRef {
        SymbolRef {
            name: self.name.clone(),
            module: Some(self.module.clone()),
            kind: self.kind,
        }
    }
}

/// A table of symbol definitions for resolving references
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Symbols by module specifier
    by_module: IndexMap<String, Vec<SymbolDef>>,
    /// All symbols indexed by qualified name
    by_name: HashMap<String, SymbolDef>,
    /// Type references found during parsing (for later resolution)
    type_refs: Vec<(String, Location)>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a symbol definition
    pub fn add(&mut self, symbol: SymbolDef) {
        let qualified = format!("{}:{}", symbol.module, symbol.name);
        self.by_name.insert(qualified, symbol.clone());
        self.by_module
            .entry(symbol.module.clone())
            .or_default()
            .push(symbol);
    }

    /// Record a type reference for later resolution
    pub fn record_type_ref(&mut self, name: impl Into<String>, location: Location) {
        self.type_refs.push((name.into(), location));
    }

    /// Look up a symbol by name in a specific module
    pub fn lookup(&self, module: &str, name: &str) -> Option<&SymbolDef> {
        let qualified = format!("{}:{}", module, name);
        self.by_name.get(&qualified)
    }

    /// Look up a symbol by name across all modules
    pub fn lookup_global(&self, name: &str) -> Option<&SymbolDef> {
        // First try exact match
        for (_, symbols) in &self.by_module {
            for symbol in symbols {
                if symbol.name == name {
                    return Some(symbol);
                }
            }
        }
        None
    }

    /// Get all symbols for a module
    pub fn symbols_for_module(&self, module: &str) -> &[SymbolDef] {
        self.by_module
            .get(module)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all exported symbols for a module
    pub fn exports_for_module(&self, module: &str) -> Vec<&SymbolDef> {
        self.symbols_for_module(module)
            .iter()
            .filter(|s| s.exported)
            .collect()
    }

    /// Get all modules
    pub fn modules(&self) -> impl Iterator<Item = &str> {
        self.by_module.keys().map(|s| s.as_str())
    }

    /// Get all type references
    pub fn type_references(&self) -> &[(String, Location)] {
        &self.type_refs
    }

    /// Resolve a type reference to a symbol definition
    pub fn resolve_type_ref(&self, name: &str) -> Option<&SymbolDef> {
        // Try to find the type in the symbol table
        self.lookup_global(name)
    }

    /// Check if a type is a built-in (doesn't need resolution)
    pub fn is_builtin_type(name: &str) -> bool {
        matches!(
            name,
            "string"
                | "number"
                | "boolean"
                | "void"
                | "null"
                | "undefined"
                | "never"
                | "unknown"
                | "any"
                | "object"
                | "symbol"
                | "bigint"
                | "Array"
                | "Promise"
                | "Map"
                | "Set"
                | "WeakMap"
                | "WeakSet"
                | "Record"
                | "Partial"
                | "Required"
                | "Readonly"
                | "Pick"
                | "Omit"
                | "Exclude"
                | "Extract"
                | "NonNullable"
                | "ReturnType"
                | "InstanceType"
                | "Parameters"
                | "ConstructorParameters"
                | "ThisType"
                | "Awaited"
        )
    }

    /// Get the number of symbols
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_ref() {
        let sym = SymbolRef::with_module("readFile", "runtime:fs", SymbolKind::Function);
        assert_eq!(sym.qualified_name(), "runtime:fs:readFile");
        assert!(sym.doc_link("/docs").contains("runtime/fs"));
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();

        let loc = Location::new("test.ts", 1, 0);
        let sym = SymbolDef::new("readFile", SymbolKind::Function, loc, "runtime:fs")
            .as_exported()
            .with_doc("Reads a file");

        table.add(sym);

        assert_eq!(table.len(), 1);
        assert!(table.lookup("runtime:fs", "readFile").is_some());
        assert!(table.lookup_global("readFile").is_some());
    }

    #[test]
    fn test_builtin_types() {
        assert!(SymbolTable::is_builtin_type("string"));
        assert!(SymbolTable::is_builtin_type("Promise"));
        assert!(!SymbolTable::is_builtin_type("MyCustomType"));
    }
}
