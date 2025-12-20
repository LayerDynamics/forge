//! Visibility and export handling
//!
//! This module provides types for tracking symbol visibility
//! (public, private, internal) in documentation.

use serde::{Deserialize, Serialize};

/// Symbol visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    /// Public export (available to users)
    #[default]
    Public,

    /// Private (not exported)
    Private,

    /// Internal (exported but marked @internal)
    Internal,

    /// Declared (ambient declaration)
    Declare,
}

impl Visibility {
    /// Check if this is publicly visible
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    /// Check if this should be included in documentation
    pub fn should_document(&self) -> bool {
        matches!(self, Visibility::Public | Visibility::Declare)
    }

    /// Get display string
    pub fn display(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Internal => "internal",
            Visibility::Declare => "declare",
        }
    }

    /// Get CSS class for styling
    pub fn css_class(&self) -> &'static str {
        match self {
            Visibility::Public => "visibility-public",
            Visibility::Private => "visibility-private",
            Visibility::Internal => "visibility-internal",
            Visibility::Declare => "visibility-declare",
        }
    }
}

/// Declaration kind (how the symbol was declared)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclarationKind {
    /// Normal export
    #[default]
    Export,

    /// Default export
    DefaultExport,

    /// Re-export from another module
    ReExport,

    /// Ambient declaration (declare)
    Ambient,

    /// Local (not exported)
    Local,
}

impl DeclarationKind {
    /// Check if this is an export
    pub fn is_export(&self) -> bool {
        matches!(
            self,
            DeclarationKind::Export | DeclarationKind::DefaultExport | DeclarationKind::ReExport
        )
    }

    /// Check if this is the default export
    pub fn is_default(&self) -> bool {
        matches!(self, DeclarationKind::DefaultExport)
    }
}

/// Export information for a symbol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportInfo {
    /// The exported name
    pub name: String,

    /// The local name (if different from exported)
    pub local_name: Option<String>,

    /// Declaration kind
    pub kind: DeclarationKind,

    /// Source module (for re-exports)
    pub source: Option<String>,
}

impl ExportInfo {
    /// Create a simple export
    pub fn export(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            kind: DeclarationKind::Export,
            source: None,
        }
    }

    /// Create a default export
    pub fn default_export(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            kind: DeclarationKind::DefaultExport,
            source: None,
        }
    }

    /// Create a re-export
    pub fn re_export(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: None,
            kind: DeclarationKind::ReExport,
            source: Some(source.into()),
        }
    }

    /// Create a renamed export
    pub fn renamed(name: impl Into<String>, local_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            local_name: Some(local_name.into()),
            kind: DeclarationKind::Export,
            source: None,
        }
    }

    /// Get the actual name to use in code
    pub fn code_name(&self) -> &str {
        self.local_name.as_deref().unwrap_or(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility() {
        assert!(Visibility::Public.is_public());
        assert!(Visibility::Public.should_document());
        assert!(!Visibility::Private.should_document());
        assert!(!Visibility::Internal.should_document());
        assert!(Visibility::Declare.should_document());
    }

    #[test]
    fn test_declaration_kind() {
        assert!(DeclarationKind::Export.is_export());
        assert!(DeclarationKind::DefaultExport.is_export());
        assert!(DeclarationKind::DefaultExport.is_default());
        assert!(!DeclarationKind::Local.is_export());
    }

    #[test]
    fn test_export_info() {
        let export = ExportInfo::export("readFile");
        assert_eq!(export.name, "readFile");
        assert_eq!(export.code_name(), "readFile");

        let renamed = ExportInfo::renamed("readFile", "internalReadFile");
        assert_eq!(renamed.name, "readFile");
        assert_eq!(renamed.code_name(), "internalReadFile");
    }
}
