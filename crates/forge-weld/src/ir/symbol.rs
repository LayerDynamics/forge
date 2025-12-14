//! Symbol metadata for ops and structs
//!
//! This module provides metadata structures for representing Rust ops
//! and structs that will be exposed to TypeScript.

use crate::ir::WeldType;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Parameter attribute from deno_core
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ParamAttr {
    /// No special attribute
    #[default]
    None,
    /// #[string] - string parameter
    String,
    /// #[serde] - serde-serialized parameter
    Serde,
    /// #[buffer] - buffer parameter
    Buffer,
    /// #[global] - global parameter
    Global,
    /// #[state] - state parameter
    State,
}

impl FromStr for ParamAttr {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "string" => ParamAttr::String,
            "serde" => ParamAttr::Serde,
            "buffer" => ParamAttr::Buffer,
            "global" => ParamAttr::Global,
            "state" => ParamAttr::State,
            _ => ParamAttr::None,
        })
    }
}

/// Op2 macro attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Op2Attrs {
    /// Whether this is a fast op
    pub is_fast: bool,
    /// Whether the return is a string (#[string] on return)
    pub string_return: bool,
    /// Whether the return is serde-serialized (#[serde] on return)
    pub serde_return: bool,
    /// Whether this is an async op
    pub is_async: bool,
    /// Optional rename for the op
    pub rename: Option<String>,
}

/// Parameter metadata for an op function
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OpParam {
    /// Parameter name in Rust
    pub rust_name: String,
    /// Parameter name in TypeScript (camelCase)
    pub ts_name: String,
    /// Parameter type
    pub ty: WeldType,
    /// Parameter attribute
    pub attr: ParamAttr,
    /// Documentation comments
    pub doc: Option<String>,
    /// Whether this parameter is optional (has default)
    pub optional: bool,
}

impl OpParam {
    /// Create a new parameter
    pub fn new(rust_name: impl Into<String>, ty: WeldType) -> Self {
        let rust_name = rust_name.into();
        let ts_name = to_camel_case(&rust_name);
        Self {
            rust_name,
            ts_name,
            ty,
            attr: ParamAttr::None,
            doc: None,
            optional: false,
        }
    }

    /// Set the TypeScript name
    pub fn with_ts_name(mut self, name: impl Into<String>) -> Self {
        self.ts_name = name.into();
        self
    }

    /// Set the parameter attribute
    pub fn with_attr(mut self, attr: ParamAttr) -> Self {
        self.attr = attr;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Check if this is the OpState parameter
    pub fn is_op_state(&self) -> bool {
        self.ty.is_op_state()
    }

    /// Get TypeScript parameter declaration
    pub fn to_typescript_param(&self) -> String {
        let optional = if self.optional { "?" } else { "" };
        format!("{}{}: {}", self.ts_name, optional, self.ty.to_typescript())
    }
}

/// Metadata for a single op function
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpSymbol {
    /// Rust function name (e.g., "op_fs_read_text")
    pub rust_name: String,
    /// TypeScript export name (e.g., "readTextFile")
    pub ts_name: String,
    /// Whether the op is async
    pub is_async: bool,
    /// Parameters (excluding OpState)
    pub params: Vec<OpParam>,
    /// Return type
    pub return_type: WeldType,
    /// Documentation comments
    pub doc: Option<String>,
    /// Op2 attributes
    pub op2_attrs: Op2Attrs,
    /// Module this op belongs to
    pub module: Option<String>,
}

impl OpSymbol {
    /// Create a new op symbol
    pub fn new(rust_name: impl Into<String>, ts_name: impl Into<String>) -> Self {
        Self {
            rust_name: rust_name.into(),
            ts_name: ts_name.into(),
            is_async: false,
            params: Vec::new(),
            return_type: WeldType::void(),
            doc: None,
            op2_attrs: Op2Attrs::default(),
            module: None,
        }
    }

    /// Create from just the Rust name (auto-generate TS name)
    pub fn from_rust_name(rust_name: impl Into<String>) -> Self {
        let rust_name = rust_name.into();
        let ts_name = op_name_to_ts(&rust_name);
        Self::new(rust_name, ts_name)
    }

    /// Mark as async
    pub fn async_op(mut self) -> Self {
        self.is_async = true;
        self
    }

    /// Add a parameter
    pub fn param(mut self, param: OpParam) -> Self {
        self.params.push(param);
        self
    }

    /// Set parameters
    pub fn with_params(mut self, params: Vec<OpParam>) -> Self {
        self.params = params;
        self
    }

    /// Set return type
    pub fn returns(mut self, ty: WeldType) -> Self {
        self.return_type = ty;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set op2 attributes
    pub fn with_op2_attrs(mut self, attrs: Op2Attrs) -> Self {
        self.op2_attrs = attrs;
        self
    }

    /// Set module
    pub fn in_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Set TypeScript name
    pub fn ts_name(mut self, name: impl Into<String>) -> Self {
        self.ts_name = name.into();
        self
    }

    /// Get visible parameters (excludes OpState)
    pub fn visible_params(&self) -> impl Iterator<Item = &OpParam> {
        self.params.iter().filter(|p| !p.is_op_state())
    }

    /// Get TypeScript return type
    pub fn ts_return_type(&self) -> String {
        if self.is_async {
            if self.return_type.is_async_result() {
                // Result<T, E> already produces Promise<T>
                self.return_type.to_typescript()
            } else {
                format!("Promise<{}>", self.return_type.to_typescript())
            }
        } else {
            self.return_type.to_typescript()
        }
    }

    /// Get TypeScript function signature
    pub fn to_typescript_signature(&self) -> String {
        let params: Vec<String> = self
            .visible_params()
            .map(|p| p.to_typescript_param())
            .collect();
        let async_kw = if self.is_async { "async " } else { "" };
        format!(
            "{}function {}({}): {}",
            async_kw,
            self.ts_name,
            params.join(", "),
            self.ts_return_type()
        )
    }
}

/// Field metadata for a struct
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructField {
    /// Rust field name
    pub rust_name: String,
    /// TypeScript field name (camelCase)
    pub ts_name: String,
    /// Field type
    pub ty: WeldType,
    /// Whether this field is optional (Option<T> or #[serde(default)])
    pub optional: bool,
    /// Documentation comments
    pub doc: Option<String>,
    /// Whether this field is readonly
    pub readonly: bool,
}

impl StructField {
    /// Create a new field
    pub fn new(rust_name: impl Into<String>, ty: WeldType) -> Self {
        let rust_name = rust_name.into();
        let ts_name = to_camel_case(&rust_name);

        // Auto-detect optional from Option type
        let optional = matches!(&ty, WeldType::Option(_));

        Self {
            rust_name,
            ts_name,
            ty,
            optional,
            doc: None,
            readonly: false,
        }
    }

    /// Set the TypeScript name
    pub fn with_ts_name(mut self, name: impl Into<String>) -> Self {
        self.ts_name = name.into();
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark as readonly
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Get TypeScript field declaration
    pub fn to_typescript_field(&self) -> String {
        let readonly = if self.readonly { "readonly " } else { "" };
        let optional = if self.optional { "?" } else { "" };

        // For Option<T>, unwrap to T in the type (optional marker handles nullability)
        let ty = match &self.ty {
            WeldType::Option(inner) => inner.to_typescript(),
            _ => self.ty.to_typescript(),
        };

        format!("{}{}{}: {}", readonly, self.ts_name, optional, ty)
    }
}

/// Metadata for a struct exposed to TypeScript
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeldStruct {
    /// Rust struct name
    pub rust_name: String,
    /// TypeScript interface name
    pub ts_name: String,
    /// Fields
    pub fields: Vec<StructField>,
    /// Documentation comments
    pub doc: Option<String>,
    /// Whether to generate as a type alias instead of interface
    pub is_type_alias: bool,
    /// Generic type parameters
    pub type_params: Vec<String>,
}

impl WeldStruct {
    /// Create a new struct
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            rust_name: name.clone(),
            ts_name: name,
            fields: Vec::new(),
            doc: None,
            is_type_alias: false,
            type_params: Vec::new(),
        }
    }

    /// Set TypeScript name
    pub fn with_ts_name(mut self, name: impl Into<String>) -> Self {
        self.ts_name = name.into();
        self
    }

    /// Add a field
    pub fn field(mut self, field: StructField) -> Self {
        self.fields.push(field);
        self
    }

    /// Set fields
    pub fn with_fields(mut self, fields: Vec<StructField>) -> Self {
        self.fields = fields;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Mark as type alias
    pub fn as_type_alias(mut self) -> Self {
        self.is_type_alias = true;
        self
    }

    /// Add type parameters
    pub fn with_type_params(mut self, params: Vec<String>) -> Self {
        self.type_params = params;
        self
    }

    /// Get TypeScript interface declaration
    pub fn to_typescript_interface(&self) -> String {
        let mut output = String::new();

        // Doc comment
        if let Some(doc) = &self.doc {
            output.push_str(&format!("/** {} */\n", doc));
        }

        // Type parameters
        let type_params = if self.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.type_params.join(", "))
        };

        if self.is_type_alias {
            // Type alias
            if self.fields.len() == 1 {
                output.push_str(&format!(
                    "export type {}{} = {};\n",
                    self.ts_name,
                    type_params,
                    self.fields[0].ty.to_typescript()
                ));
            }
        } else {
            // Interface
            output.push_str(&format!(
                "export interface {}{} {{\n",
                self.ts_name, type_params
            ));

            for field in &self.fields {
                if let Some(doc) = &field.doc {
                    output.push_str(&format!("  /** {} */\n", doc));
                }
                output.push_str(&format!("  {};\n", field.to_typescript_field()));
            }

            output.push_str("}\n");
        }

        output
    }
}

/// Metadata for an enum exposed to TypeScript
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeldEnum {
    /// Rust enum name
    pub rust_name: String,
    /// TypeScript type name
    pub ts_name: String,
    /// Variants
    pub variants: Vec<EnumVariant>,
    /// Documentation comments
    pub doc: Option<String>,
}

/// Enum variant metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,
    /// Variant value (for string enums)
    pub value: Option<String>,
    /// Associated data (for ADT enums)
    pub data: Option<WeldType>,
    /// Documentation
    pub doc: Option<String>,
}

impl WeldEnum {
    /// Create a new enum
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            rust_name: name.clone(),
            ts_name: name,
            variants: Vec::new(),
            doc: None,
        }
    }

    /// Add a variant
    pub fn variant(mut self, variant: EnumVariant) -> Self {
        self.variants.push(variant);
        self
    }

    /// Get TypeScript type declaration
    pub fn to_typescript_type(&self) -> String {
        let mut output = String::new();

        if let Some(doc) = &self.doc {
            output.push_str(&format!("/** {} */\n", doc));
        }

        // Check if this is a simple string enum
        let is_string_enum = self.variants.iter().all(|v| v.data.is_none());

        if is_string_enum {
            // String literal union type
            let variants: Vec<String> = self
                .variants
                .iter()
                .map(|v| {
                    let value = v.value.as_ref().unwrap_or(&v.name);
                    format!("\"{}\"", value)
                })
                .collect();
            output.push_str(&format!(
                "export type {} = {};\n",
                self.ts_name,
                variants.join(" | ")
            ));
        } else {
            // Discriminated union for ADT enums
            let variants: Vec<String> = self
                .variants
                .iter()
                .map(|v| {
                    if let Some(data) = &v.data {
                        format!("{{ type: \"{}\"; data: {} }}", v.name, data.to_typescript())
                    } else {
                        format!("{{ type: \"{}\" }}", v.name)
                    }
                })
                .collect();
            output.push_str(&format!(
                "export type {} = {};\n",
                self.ts_name,
                variants.join(" | ")
            ));
        }

        output
    }
}

// Helper functions

/// Convert snake_case to camelCase
pub fn to_camel_case(s: &str) -> String {
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

/// Convert op name (op_fs_read_text) to TypeScript name (readText)
pub fn op_name_to_ts(op_name: &str) -> String {
    // Remove op_ prefix and module prefix (e.g., op_fs_read_text -> read_text)
    let name = op_name.strip_prefix("op_").unwrap_or(op_name);

    // Find second underscore (after module name) and take rest
    let parts: Vec<&str> = name.splitn(2, '_').collect();
    let base_name = if parts.len() > 1 { parts[1] } else { name };

    to_camel_case(base_name)
}

/// Convert PascalCase to camelCase
pub fn to_lower_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("read_text_file"), "readTextFile");
        assert_eq!(to_camel_case("is_file"), "isFile");
        assert_eq!(to_camel_case("already_camel"), "alreadyCamel");
    }

    #[test]
    fn test_op_name_to_ts() {
        assert_eq!(op_name_to_ts("op_fs_read_text"), "readText");
        assert_eq!(op_name_to_ts("op_net_fetch"), "fetch");
        assert_eq!(op_name_to_ts("op_sys_clipboard_read"), "clipboardRead");
    }

    #[test]
    fn test_op_symbol() {
        let op = OpSymbol::from_rust_name("op_fs_read_text")
            .async_op()
            .param(OpParam::new("path", WeldType::string()))
            .returns(WeldType::result(
                WeldType::string(),
                WeldType::struct_ref("FsError"),
            ));

        assert_eq!(op.ts_name, "readText");
        assert!(op.is_async);
        assert_eq!(op.ts_return_type(), "Promise<string>");
    }

    #[test]
    fn test_struct_field() {
        let field = StructField::new("is_file", WeldType::bool());
        assert_eq!(field.ts_name, "isFile");
        assert_eq!(field.to_typescript_field(), "isFile: boolean");
    }

    #[test]
    fn test_weld_struct() {
        let s = WeldStruct::new("FileStat")
            .field(StructField::new("is_file", WeldType::bool()))
            .field(StructField::new(
                "size",
                WeldType::Primitive(crate::ir::WeldPrimitive::U64),
            ))
            .with_doc("File metadata");

        let ts = s.to_typescript_interface();
        assert!(ts.contains("export interface FileStat"));
        assert!(ts.contains("isFile: boolean"));
        assert!(ts.contains("size: bigint"));
    }
}
