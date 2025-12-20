//! Variable definitions
//!
//! This module provides types for representing TypeScript variables
//! and constants in documentation.

use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VariableKind {
    /// var declaration
    Var,
    /// let declaration
    Let,
    /// const declaration
    #[default]
    Const,
}

impl VariableKind {
    /// Get TypeScript keyword
    pub fn keyword(&self) -> &'static str {
        match self {
            VariableKind::Var => "var",
            VariableKind::Let => "let",
            VariableKind::Const => "const",
        }
    }

    /// Check if this is a const
    pub fn is_const(&self) -> bool {
        matches!(self, VariableKind::Const)
    }
}

/// Variable definition
///
/// Represents a TypeScript variable or constant like:
/// ```typescript
/// export const VERSION = "1.0.0";
/// export let counter: number;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDef {
    /// Declaration kind (var, let, const)
    #[serde(default)]
    pub kind: VariableKind,

    /// Variable type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ts_type: Option<EtchType>,
    pub(crate) value: Option<String>,
}

impl VariableDef {
    /// Create a new variable definition
    pub fn new(kind: VariableKind) -> Self {
        Self {
            kind,
            ts_type: None,
            value: None,
        }
    }

    /// Create a const variable
    pub fn const_var() -> Self {
        Self::new(VariableKind::Const)
    }

    /// Create a let variable
    pub fn let_var() -> Self {
        Self::new(VariableKind::Let)
    }

    /// Set the type
    pub fn with_type(mut self, ty: EtchType) -> Self {
        self.ts_type = Some(ty);
        self
    }

    /// Generate TypeScript declaration (without name)
    pub fn to_typescript_keyword(&self) -> &'static str {
        self.kind.keyword()
    }

    /// Generate TypeScript declaration
    pub fn to_typescript(&self, name: &str) -> String {
        let type_str = self
            .ts_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        format!("{} {}{}", self.kind.keyword(), name, type_str)
    }

    /// Check if this is a const
    pub fn is_const(&self) -> bool {
        self.kind.is_const()
    }
}

impl Default for VariableDef {
    fn default() -> Self {
        Self {
            kind: VariableKind::Const,
            ts_type: None,
            value: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_variable() {
        let var = VariableDef::const_var().with_type(EtchType::string());

        assert!(var.is_const());
        assert_eq!(var.to_typescript("VERSION"), "const VERSION: string");
    }

    #[test]
    fn test_let_variable() {
        let var = VariableDef::let_var().with_type(EtchType::number());

        assert!(!var.is_const());
        assert_eq!(var.to_typescript("counter"), "let counter: number");
    }

    #[test]
    fn test_variable_without_type() {
        let var = VariableDef::const_var();

        assert_eq!(var.to_typescript("value"), "const value");
    }

    #[test]
    fn test_variable_kind() {
        assert_eq!(VariableKind::Var.keyword(), "var");
        assert_eq!(VariableKind::Let.keyword(), "let");
        assert_eq!(VariableKind::Const.keyword(), "const");
    }
}
