//! Parameter definition types
//!
//! This module provides types for representing function/op parameters
//! in documentation.

use crate::decorators::DecoratorDef;
use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// Parameter definition for functions and ops
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamDef {
    /// Parameter name (Rust style, e.g., "file_path")
    pub name: String,

    /// TypeScript type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ts_type: Option<EtchType>,

    /// Whether this parameter is optional
    #[serde(default)]
    pub optional: bool,

    /// Default value (as string representation)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default: Option<String>,

    /// Parameter documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,

    /// Whether this is a rest parameter (...args)
    #[serde(default)]
    pub rest: bool,

    /// Decorators applied to this parameter
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<DecoratorDef>,
}

impl ParamDef {
    /// Create a new parameter definition
    pub fn new(name: impl Into<String>, param_type: EtchType) -> Self {
        Self {
            name: name.into(),
            ts_type: Some(param_type),
            optional: false,
            default: None,
            doc: None,
            rest: false,
            decorators: vec![],
        }
    }

    /// Create a parameter with just a name
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ts_type: None,
            optional: false,
            default: None,
            doc: None,
            rest: false,
            decorators: vec![],
        }
    }

    /// Set the TypeScript type
    pub fn with_type(mut self, ty: EtchType) -> Self {
        self.ts_type = Some(ty);
        self
    }

    /// Mark as optional
    pub fn as_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self.optional = true; // Default implies optional
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Mark as rest parameter
    pub fn as_rest(mut self) -> Self {
        self.rest = true;
        self
    }

    /// Get TypeScript parameter declaration
    pub fn to_typescript(&self) -> String {
        let rest = if self.rest { "..." } else { "" };
        let optional = if self.optional && !self.rest { "?" } else { "" };
        let type_str = self
            .ts_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_else(|| ": any".to_string());
        format!("{}{}{}{}", rest, self.name, optional, type_str)
    }

    /// Get TypeScript parameter declaration with default
    pub fn to_typescript_with_default(&self) -> String {
        if let Some(ref default) = self.default {
            let type_str = self
                .ts_type
                .as_ref()
                .map(|t| t.to_typescript())
                .unwrap_or_else(|| "any".to_string());
            format!("{}: {} = {}", self.name, type_str, default)
        } else {
            self.to_typescript()
        }
    }
}

/// Convert snake_case to camelCase
#[allow(dead_code)]
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_def_creation() {
        let param = ParamDef::new("file_path", EtchType::string());
        assert_eq!(param.name, "file_path");
        assert!(!param.optional);
    }

    #[test]
    fn test_param_to_typescript() {
        let param = ParamDef::new("file_path", EtchType::string());
        assert_eq!(param.to_typescript(), "file_path: string");

        let optional = ParamDef::new("options", EtchType::simple_ref("Options")).as_optional();
        assert_eq!(optional.to_typescript(), "options?: Options");

        let rest = ParamDef::new("args", EtchType::array(EtchType::string())).as_rest();
        assert_eq!(rest.to_typescript(), "...args: string[]");
    }

    #[test]
    fn test_param_with_default() {
        let param = ParamDef::new("encoding", EtchType::string()).with_default("\"utf-8\"");
        assert_eq!(
            param.to_typescript_with_default(),
            "encoding: string = \"utf-8\""
        );
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("file_path"), "filePath");
        assert_eq!(to_camel_case("read_text_file"), "readTextFile");
        assert_eq!(to_camel_case("already"), "already");
    }
}
