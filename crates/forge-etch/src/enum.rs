//! Enum definitions
//!
//! This module provides types for representing TypeScript enums
//! in documentation, supporting both numeric and string enums,
//! as well as const enums.

use crate::js_doc::EtchDoc;
use serde::{Deserialize, Serialize};

/// Enum variant/member definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumMemberDef {
    /// Member name
    pub name: String,

    /// Initializer value (if any)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub init: Option<EnumMemberValue>,

    /// Member documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,
}

/// Value of an enum member
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum EnumMemberValue {
    /// String value
    #[serde(rename = "string")]
    String { value: String },

    /// Number value
    #[serde(rename = "number")]
    Number { value: f64 },

    /// BigInt value
    #[serde(rename = "bigint")]
    BigInt { value: String },

    /// Computed/expression value
    #[serde(rename = "computed")]
    Computed { repr: String },
}

impl EnumMemberValue {
    /// Create a string value
    pub fn string(s: impl Into<String>) -> Self {
        EnumMemberValue::String { value: s.into() }
    }

    /// Create a number value
    pub fn number(n: f64) -> Self {
        EnumMemberValue::Number { value: n }
    }

    /// Create an integer value
    pub fn integer(n: i64) -> Self {
        EnumMemberValue::Number { value: n as f64 }
    }

    /// Create a computed value
    pub fn computed(repr: impl Into<String>) -> Self {
        EnumMemberValue::Computed { repr: repr.into() }
    }

    /// Get TypeScript representation
    pub fn to_typescript(&self) -> String {
        match self {
            EnumMemberValue::String { value } => format!("\"{}\"", value),
            EnumMemberValue::Number { value } => {
                if value.fract() == 0.0 {
                    format!("{}", *value as i64)
                } else {
                    value.to_string()
                }
            }
            EnumMemberValue::BigInt { value } => format!("{}n", value),
            EnumMemberValue::Computed { repr } => repr.clone(),
        }
    }
}

impl EnumMemberDef {
    /// Create a new enum member
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            init: None,
            doc: None,
        }
    }

    /// Set the value
    pub fn with_value(mut self, value: EnumMemberValue) -> Self {
        self.init = Some(value);
        self
    }

    /// Set a string value
    pub fn with_string_value(mut self, value: impl Into<String>) -> Self {
        self.init = Some(EnumMemberValue::string(value));
        self
    }

    /// Set a number value
    pub fn with_number_value(mut self, value: f64) -> Self {
        self.init = Some(EnumMemberValue::number(value));
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Generate TypeScript declaration
    pub fn to_typescript(&self) -> String {
        match &self.init {
            Some(value) => format!("{} = {}", self.name, value.to_typescript()),
            None => self.name.clone(),
        }
    }
}

/// Enum definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct EnumDef {
    /// Enum members/variants
    #[serde(default)]
    pub members: Vec<EnumMemberDef>,

    /// Whether this is a const enum
    #[serde(default)]
    pub is_const: bool,

    /// Whether this is a declare enum (ambient)
    #[serde(default)]
    pub is_declare: bool,
}

impl EnumDef {
    /// Create a new enum definition
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a member
    pub fn with_member(mut self, member: EnumMemberDef) -> Self {
        self.members.push(member);
        self
    }

    /// Add multiple members
    pub fn with_members(mut self, members: Vec<EnumMemberDef>) -> Self {
        self.members = members;
        self
    }

    /// Mark as const enum
    pub fn as_const(mut self) -> Self {
        self.is_const = true;
        self
    }

    /// Mark as declare enum
    pub fn as_declare(mut self) -> Self {
        self.is_declare = true;
        self
    }

    /// Check if this is a string enum
    pub fn is_string_enum(&self) -> bool {
        self.members
            .iter()
            .all(|m| matches!(m.init.as_ref(), Some(EnumMemberValue::String { .. }) | None))
    }

    /// Check if this is a numeric enum
    pub fn is_numeric_enum(&self) -> bool {
        self.members
            .iter()
            .all(|m| matches!(m.init.as_ref(), Some(EnumMemberValue::Number { .. }) | None))
    }

    /// Check if this is a heterogeneous enum (mixed types)
    pub fn is_heterogeneous(&self) -> bool {
        !self.is_string_enum() && !self.is_numeric_enum()
    }

    /// Get member by name
    pub fn get_member(&self, name: &str) -> Option<&EnumMemberDef> {
        self.members.iter().find(|m| m.name == name)
    }
}

/// Rust enum representation (from forge-weld)
///
/// This represents Rust enums that are exposed to TypeScript.
/// They're typically converted to TypeScript union types or
/// discriminated unions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustEnumDef {
    /// Rust name
    pub rust_name: String,

    /// TypeScript name
    pub ts_name: String,

    /// Enum variants
    pub variants: Vec<RustEnumVariant>,

    /// Whether this is a simple enum (no data)
    #[serde(default)]
    pub is_simple: bool,

    /// Documentation
    #[serde(default)]
    pub doc: EtchDoc,
}

/// A variant of a Rust enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustEnumVariant {
    /// Variant name
    pub name: String,

    /// TypeScript name (often the same)
    pub ts_name: String,

    /// Variant kind
    pub kind: RustEnumVariantKind,

    /// Documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,
}

/// Kind of Rust enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RustEnumVariantKind {
    /// Unit variant (no data)
    Unit,

    /// Tuple variant (unnamed fields)
    Tuple { fields: Vec<String> },

    /// Struct variant (named fields)
    Struct { fields: Vec<RustEnumField> },
}

/// Field of a struct enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustEnumField {
    /// Field name
    pub name: String,

    /// TypeScript type
    pub ts_type: String,
}

impl RustEnumDef {
    /// Create a new Rust enum definition
    pub fn new(rust_name: impl Into<String>, ts_name: impl Into<String>) -> Self {
        Self {
            rust_name: rust_name.into(),
            ts_name: ts_name.into(),
            variants: vec![],
            is_simple: true,
            doc: EtchDoc::default(),
        }
    }

    /// Add a variant
    pub fn with_variant(mut self, variant: RustEnumVariant) -> Self {
        // Check if still simple
        if !matches!(variant.kind, RustEnumVariantKind::Unit) {
            self.is_simple = false;
        }
        self.variants.push(variant);
        self
    }

    /// Generate TypeScript type
    pub fn to_typescript(&self) -> String {
        if self.is_simple {
            // Simple enum -> string literal union
            let variants: Vec<String> = self
                .variants
                .iter()
                .map(|v| format!("\"{}\"", v.ts_name))
                .collect();
            variants.join(" | ")
        } else {
            // Complex enum -> discriminated union
            let variants: Vec<String> = self
                .variants
                .iter()
                .map(|v| match &v.kind {
                    RustEnumVariantKind::Unit => {
                        format!("{{ type: \"{}\" }}", v.ts_name)
                    }
                    RustEnumVariantKind::Tuple { fields } => {
                        let field_types = fields.join(", ");
                        format!("{{ type: \"{}\", value: [{}] }}", v.ts_name, field_types)
                    }
                    RustEnumVariantKind::Struct { fields } => {
                        let field_strs: Vec<String> = fields
                            .iter()
                            .map(|f| format!("{}: {}", f.name, f.ts_type))
                            .collect();
                        format!("{{ type: \"{}\", {} }}", v.ts_name, field_strs.join(", "))
                    }
                })
                .collect();
            variants.join(" | ")
        }
    }
}

impl RustEnumVariant {
    /// Create a unit variant
    pub fn unit(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            ts_name: name.clone(),
            name,
            kind: RustEnumVariantKind::Unit,
            doc: None,
        }
    }

    /// Create a tuple variant
    pub fn tuple(name: impl Into<String>, fields: Vec<String>) -> Self {
        let name = name.into();
        Self {
            ts_name: name.clone(),
            name,
            kind: RustEnumVariantKind::Tuple { fields },
            doc: None,
        }
    }

    /// Create a struct variant
    pub fn with_struct(name: impl Into<String>, fields: Vec<RustEnumField>) -> Self {
        let name = name.into();
        Self {
            ts_name: name.clone(),
            name,
            kind: RustEnumVariantKind::Struct { fields },
            doc: None,
        }
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_member() {
        let member = EnumMemberDef::new("Red").with_string_value("red");
        assert_eq!(member.to_typescript(), "Red = \"red\"");

        let member = EnumMemberDef::new("Zero").with_number_value(0.0);
        assert_eq!(member.to_typescript(), "Zero = 0");
    }

    #[test]
    fn test_string_enum() {
        let enum_def = EnumDef::new()
            .with_member(EnumMemberDef::new("Red").with_string_value("red"))
            .with_member(EnumMemberDef::new("Green").with_string_value("green"))
            .with_member(EnumMemberDef::new("Blue").with_string_value("blue"));

        assert!(enum_def.is_string_enum());
        assert!(!enum_def.is_numeric_enum());
        assert_eq!(enum_def.members.len(), 3);
    }

    #[test]
    fn test_numeric_enum() {
        let enum_def = EnumDef::new()
            .with_member(EnumMemberDef::new("First"))
            .with_member(EnumMemberDef::new("Second"))
            .with_member(EnumMemberDef::new("Third"));

        // No explicit values = numeric enum
        assert!(enum_def.is_numeric_enum());
    }

    #[test]
    fn test_rust_enum_simple() {
        let enum_def = RustEnumDef::new("Color", "Color")
            .with_variant(RustEnumVariant::unit("Red"))
            .with_variant(RustEnumVariant::unit("Green"))
            .with_variant(RustEnumVariant::unit("Blue"));

        assert!(enum_def.is_simple);
        assert_eq!(enum_def.to_typescript(), "\"Red\" | \"Green\" | \"Blue\"");
    }

    #[test]
    fn test_rust_enum_complex() {
        let enum_def = RustEnumDef::new("Message", "Message")
            .with_variant(RustEnumVariant::unit("Quit"))
            .with_variant(RustEnumVariant::tuple(
                "Move",
                vec!["number".to_string(), "number".to_string()],
            ))
            .with_variant(RustEnumVariant::with_struct(
                "Write",
                vec![RustEnumField {
                    name: "content".to_string(),
                    ts_type: "string".to_string(),
                }],
            ));

        assert!(!enum_def.is_simple);
        let ts = enum_def.to_typescript();
        assert!(ts.contains("type: \"Quit\""));
        assert!(ts.contains("type: \"Move\""));
        assert!(ts.contains("type: \"Write\""));
    }
}
