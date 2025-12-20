//! Function and Op definitions
//!
//! This module provides types for representing functions and Deno ops
//! in documentation. Functions are regular TypeScript/JavaScript functions,
//! while Ops are the Rust→TypeScript bridge functions.

use crate::decorators::DecoratorDef;
use crate::params::ParamDef;
use crate::ts_type_params::TsTypeParamDef;
use crate::types::EtchType;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Op attributes from #[weld_op] and #[op2] macros
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpAttrs {
    /// Whether the op is async (returns a Future)
    #[serde(default)]
    pub is_async: bool,

    /// Whether the op uses fast API
    #[serde(default)]
    pub fast: bool,

    /// Whether the op is reentrant
    #[serde(default)]
    pub reentrant: bool,

    /// Additional attributes as key-value pairs
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attrs: IndexMap<String, String>,
}

/// Op definition - a Rust function exposed to TypeScript via Deno
///
/// Ops are the core bridge between Rust and TypeScript in Forge.
/// They're defined in Rust with #[weld_op] and #[op2] macros and
/// exposed as functions in the TypeScript SDK.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpDef {
    /// Original Rust function name (e.g., "op_fs_read_text")
    pub rust_name: String,

    /// TypeScript export name (e.g., "readTextFile")
    pub ts_name: String,

    /// Whether this op is async
    #[serde(default)]
    pub is_async: bool,

    /// Function parameters
    #[serde(default)]
    pub params: Vec<ParamDef>,

    /// Return type as TypeScript string
    pub return_type: String,

    /// Structured return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type_def: Option<EtchType>,

    /// Op attributes from macros
    #[serde(default)]
    pub op_attrs: OpAttrs,

    /// Type parameters (generics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Whether this op throws/can return an error
    #[serde(default)]
    pub can_throw: bool,

    /// Whether this op requires permissions
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub permissions: Option<Vec<String>>,
}

impl OpDef {
    /// Create a new op definition
    pub fn new(
        rust_name: impl Into<String>,
        ts_name: impl Into<String>,
        return_type: impl Into<String>,
    ) -> Self {
        Self {
            rust_name: rust_name.into(),
            ts_name: ts_name.into(),
            is_async: false,
            params: vec![],
            return_type: return_type.into(),
            return_type_def: None,
            op_attrs: OpAttrs::default(),
            type_params: vec![],
            can_throw: false,
            permissions: None,
        }
    }

    /// Mark as async
    pub fn as_async(mut self) -> Self {
        self.is_async = true;
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, param: ParamDef) -> Self {
        self.params.push(param);
        self
    }

    /// Add multiple parameters
    pub fn with_params(mut self, params: Vec<ParamDef>) -> Self {
        self.params = params;
        self
    }

    /// Set op attributes
    pub fn with_attrs(mut self, attrs: OpAttrs) -> Self {
        self.op_attrs = attrs;
        self
    }

    /// Mark that this op can throw
    pub fn with_throws(mut self) -> Self {
        self.can_throw = true;
        self
    }

    /// Set required permissions
    pub fn with_permissions(mut self, perms: Vec<String>) -> Self {
        self.permissions = Some(perms);
        self
    }

    /// Generate TypeScript function signature
    pub fn to_typescript_signature(&self) -> String {
        let type_params = if self.type_params.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = self.type_params.iter().map(|p| p.to_typescript()).collect();
            format!("<{}>", params.join(", "))
        };

        let params: Vec<String> = self.params.iter().map(|p| p.to_typescript()).collect();

        format!(
            "{}{}({}): {}",
            self.ts_name,
            type_params,
            params.join(", "),
            self.return_type
        )
    }
}

impl Default for OpDef {
    fn default() -> Self {
        Self {
            rust_name: String::new(),
            ts_name: String::new(),
            is_async: false,
            params: vec![],
            return_type: "void".to_string(),
            return_type_def: None,
            op_attrs: OpAttrs::default(),
            type_params: vec![],
            can_throw: false,
            permissions: None,
        }
    }
}

/// Function definition - a regular TypeScript/JavaScript function
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDef {
    /// Internal definition name (for default exports)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub def_name: Option<String>,

    /// Function parameters
    #[serde(default)]
    pub params: Vec<ParamDef>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Type parameters (generics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Whether this is an async function
    #[serde(default)]
    pub is_async: bool,

    /// Whether this is a generator function
    #[serde(default)]
    pub is_generator: bool,

    /// Whether this has an explicit return type
    #[serde(default)]
    pub has_body: bool,

    /// Decorators applied to this function
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<DecoratorDef>,

    /// Overloaded signatures (for function overloads)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overloads: Vec<FunctionSignature>,
}

/// A function signature (for overloads)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSignature {
    /// Parameters for this overload
    pub params: Vec<ParamDef>,

    /// Return type for this overload
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Type parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,
}

impl FunctionDef {
    /// Create a new function definition
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the definition name
    pub fn with_def_name(mut self, name: impl Into<String>) -> Self {
        self.def_name = Some(name.into());
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, param: ParamDef) -> Self {
        self.params.push(param);
        self
    }

    /// Set parameters
    pub fn with_params(mut self, params: Vec<ParamDef>) -> Self {
        self.params = params;
        self
    }

    /// Set return type
    pub fn with_return_type(mut self, ty: EtchType) -> Self {
        self.return_type = Some(ty);
        self
    }

    /// Mark as async
    pub fn as_async(mut self) -> Self {
        self.is_async = true;
        self
    }

    /// Mark as generator
    pub fn as_generator(mut self) -> Self {
        self.is_generator = true;
        self
    }

    /// Add type parameters
    pub fn with_type_params(mut self, params: Vec<TsTypeParamDef>) -> Self {
        self.type_params = params;
        self
    }

    /// Add an overload signature
    pub fn with_overload(mut self, sig: FunctionSignature) -> Self {
        self.overloads.push(sig);
        self
    }

    /// Generate TypeScript function signature
    pub fn to_typescript_signature(&self, name: &str) -> String {
        let async_kw = if self.is_async { "async " } else { "" };
        let gen_star = if self.is_generator { "*" } else { "" };

        let type_params = if self.type_params.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = self.type_params.iter().map(|p| p.to_typescript()).collect();
            format!("<{}>", params.join(", "))
        };

        let params: Vec<String> = self.params.iter().map(|p| p.to_typescript()).collect();

        let return_type = self
            .return_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        format!(
            "{}function {}{}{}({}){}",
            async_kw,
            gen_star,
            name,
            type_params,
            params.join(", "),
            return_type
        )
    }
}

impl Default for FunctionDef {
    fn default() -> Self {
        Self {
            def_name: None,
            params: vec![],
            return_type: None,
            type_params: vec![],
            is_async: false,
            is_generator: false,
            has_body: true,
            decorators: vec![],
            overloads: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EtchType;

    #[test]
    fn test_op_def_creation() {
        let op = OpDef::new("op_fs_read_text", "readTextFile", "Promise<string>")
            .as_async()
            .with_param(ParamDef::new("path", EtchType::string()));

        assert_eq!(op.rust_name, "op_fs_read_text");
        assert_eq!(op.ts_name, "readTextFile");
        assert!(op.is_async);
        assert_eq!(op.params.len(), 1);
    }

    #[test]
    fn test_op_typescript_signature() {
        let op = OpDef::new("op_fs_read_text", "readTextFile", "Promise<string>")
            .as_async()
            .with_param(ParamDef::new("path", EtchType::string()));

        assert_eq!(
            op.to_typescript_signature(),
            "readTextFile(path: string): Promise<string>"
        );
    }

    #[test]
    fn test_function_def_creation() {
        let func = FunctionDef::new()
            .as_async()
            .with_param(ParamDef::new("data", EtchType::string()))
            .with_return_type(EtchType::promise(EtchType::void()));

        assert!(func.is_async);
        assert_eq!(func.params.len(), 1);
    }

    #[test]
    fn test_function_typescript_signature() {
        let func = FunctionDef::new()
            .as_async()
            .with_param(ParamDef::new("data", EtchType::string()))
            .with_return_type(EtchType::promise(EtchType::void()));

        assert_eq!(
            func.to_typescript_signature("processData"),
            "async function processData(data: string): Promise<void>"
        );
    }
}
