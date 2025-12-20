//! TypeScript type parameters (generics)
//!
//! This module provides types for representing generic type parameters
//! like `<T>`, `<T extends Foo>`, `<T = DefaultType>`.

use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// TypeScript type parameter definition
///
/// Represents a generic type parameter like:
/// - `T` - simple type parameter
/// - `T extends Foo` - with constraint
/// - `T = string` - with default
/// - `T extends Foo = Bar` - with both
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsTypeParamDef {
    /// Parameter name (e.g., "T", "K", "V")
    pub name: String,

    /// Constraint (extends clause)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub constraint: Option<Box<EtchType>>,

    /// Default value
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default: Option<Box<EtchType>>,

    /// Whether this is a const type parameter (TypeScript 5.0+)
    #[serde(default)]
    pub is_const: bool,

    /// Whether this is an in type parameter (contravariant)
    #[serde(default)]
    pub is_in: bool,

    /// Whether this is an out type parameter (covariant)
    #[serde(default)]
    pub is_out: bool,
}

impl TsTypeParamDef {
    /// Create a new type parameter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraint: None,
            default: None,
            is_const: false,
            is_in: false,
            is_out: false,
        }
    }

    /// Set the constraint (extends clause)
    pub fn with_constraint(mut self, constraint: EtchType) -> Self {
        self.constraint = Some(Box::new(constraint));
        self
    }

    /// Set the default value
    pub fn with_default(mut self, default: EtchType) -> Self {
        self.default = Some(Box::new(default));
        self
    }

    /// Mark as const type parameter
    pub fn as_const(mut self) -> Self {
        self.is_const = true;
        self
    }

    /// Mark as in (contravariant)
    pub fn as_in(mut self) -> Self {
        self.is_in = true;
        self
    }

    /// Mark as out (covariant)
    pub fn as_out(mut self) -> Self {
        self.is_out = true;
        self
    }

    /// Generate TypeScript declaration
    pub fn to_typescript(&self) -> String {
        let mut parts = vec![];

        // Variance modifiers
        if self.is_const {
            parts.push("const".to_string());
        }
        if self.is_in {
            parts.push("in".to_string());
        }
        if self.is_out {
            parts.push("out".to_string());
        }

        // Parameter name
        parts.push(self.name.clone());

        // Constraint
        if let Some(ref constraint) = self.constraint {
            parts.push(format!("extends {}", constraint.to_typescript()));
        }

        // Default
        if let Some(ref default) = self.default {
            parts.push(format!("= {}", default.to_typescript()));
        }

        parts.join(" ")
    }

    /// Check if this has a constraint
    pub fn has_constraint(&self) -> bool {
        self.constraint.is_some()
    }

    /// Check if this has a default
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }

    /// Check if this is a simple type parameter (no constraint or default)
    pub fn is_simple(&self) -> bool {
        self.constraint.is_none() && self.default.is_none()
    }
}

impl Default for TsTypeParamDef {
    fn default() -> Self {
        Self {
            name: "T".to_string(),
            constraint: None,
            default: None,
            is_const: false,
            is_in: false,
            is_out: false,
        }
    }
}

impl std::fmt::Display for TsTypeParamDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_typescript())
    }
}

/// Helper to format a list of type parameters
pub fn format_type_params(params: &[TsTypeParamDef]) -> String {
    if params.is_empty() {
        String::new()
    } else {
        let formatted: Vec<String> = params.iter().map(|p| p.to_typescript()).collect();
        format!("<{}>", formatted.join(", "))
    }
}

/// Common type parameter patterns
impl TsTypeParamDef {
    /// Create a type parameter with an `extends object` constraint
    pub fn object_constrained(name: impl Into<String>) -> Self {
        Self::new(name).with_constraint(EtchType::simple_ref("object"))
    }

    /// Create a type parameter with an `extends string` constraint
    pub fn string_constrained(name: impl Into<String>) -> Self {
        Self::new(name).with_constraint(EtchType::string())
    }

    /// Create a keyof constraint (e.g., `K extends keyof T`)
    pub fn keyof_constrained(name: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(name).with_constraint(EtchType::new(crate::types::EtchTypeKind::TypeOperator {
            operator: crate::types::TypeOperator::KeyOf,
            type_arg: Box::new(EtchType::new(crate::types::EtchTypeKind::TypeParam(
                target.into(),
            ))),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_type_param() {
        let param = TsTypeParamDef::new("T");
        assert_eq!(param.to_typescript(), "T");
        assert!(param.is_simple());
    }

    #[test]
    fn test_constrained_type_param() {
        let param = TsTypeParamDef::new("T").with_constraint(EtchType::simple_ref("Foo"));
        assert_eq!(param.to_typescript(), "T extends Foo");
        assert!(param.has_constraint());
    }

    #[test]
    fn test_default_type_param() {
        let param = TsTypeParamDef::new("T").with_default(EtchType::string());
        assert_eq!(param.to_typescript(), "T = string");
        assert!(param.has_default());
    }

    #[test]
    fn test_full_type_param() {
        let param = TsTypeParamDef::new("T")
            .with_constraint(EtchType::simple_ref("object"))
            .with_default(EtchType::simple_ref("Record<string, unknown>"));

        assert_eq!(
            param.to_typescript(),
            "T extends object = Record<string, unknown>"
        );
    }

    #[test]
    fn test_variance_modifiers() {
        let param = TsTypeParamDef::new("T").as_const();
        assert_eq!(param.to_typescript(), "const T");

        let param = TsTypeParamDef::new("T").as_in();
        assert_eq!(param.to_typescript(), "in T");

        let param = TsTypeParamDef::new("T").as_out();
        assert_eq!(param.to_typescript(), "out T");
    }

    #[test]
    fn test_format_type_params() {
        let params = vec![
            TsTypeParamDef::new("K").with_constraint(EtchType::string()),
            TsTypeParamDef::new("V"),
        ];

        assert_eq!(format_type_params(&params), "<K extends string, V>");
    }

    #[test]
    fn test_format_empty_type_params() {
        let params: Vec<TsTypeParamDef> = vec![];
        assert_eq!(format_type_params(&params), "");
    }

    #[test]
    fn test_keyof_constrained() {
        let param = TsTypeParamDef::keyof_constrained("K", "T");
        assert_eq!(param.to_typescript(), "K extends keyof T");
    }
}
