//! Interface definitions
//!
//! This module provides types for representing TypeScript interfaces
//! in documentation, including properties, methods, call signatures,
//! and interface extension.

use crate::params::ParamDef;
use crate::ts_type_params::TsTypeParamDef;
use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// Interface property definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfacePropertyDef {
    /// Property name
    pub name: String,

    /// Property type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ts_type: Option<EtchType>,

    /// Whether this is readonly
    #[serde(default)]
    pub readonly: bool,

    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,

    /// Whether this is a computed property
    #[serde(default)]
    pub computed: bool,

    /// Property documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,
}

impl InterfacePropertyDef {
    /// Create a new property definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ts_type: None,
            readonly: false,
            optional: false,
            computed: false,
            doc: None,
        }
    }

    /// Set the type
    pub fn with_type(mut self, ty: EtchType) -> Self {
        self.ts_type = Some(ty);
        self
    }

    /// Mark as readonly
    pub fn as_readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Mark as optional
    pub fn as_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Generate TypeScript declaration
    pub fn to_typescript(&self) -> String {
        let readonly = if self.readonly { "readonly " } else { "" };
        let opt = if self.optional { "?" } else { "" };
        let type_str = self
            .ts_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        format!("{}{}{}{}", readonly, self.name, opt, type_str)
    }
}

/// Interface method definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceMethodDef {
    /// Method name
    pub name: String,

    /// Method parameters
    #[serde(default)]
    pub params: Vec<ParamDef>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Type parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,

    /// Method documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,
}

impl InterfaceMethodDef {
    /// Create a new method definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: vec![],
            return_type: None,
            type_params: vec![],
            optional: false,
            doc: None,
        }
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

    /// Mark as optional
    pub fn as_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Generate TypeScript signature
    pub fn to_typescript(&self) -> String {
        let type_params = if self.type_params.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = self.type_params.iter().map(|p| p.to_typescript()).collect();
            format!("<{}>", params.join(", "))
        };

        let params: Vec<String> = self.params.iter().map(|p| p.to_typescript()).collect();
        let opt = if self.optional { "?" } else { "" };

        let return_type = self
            .return_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        format!(
            "{}{}{}({}){}",
            self.name,
            opt,
            type_params,
            params.join(", "),
            return_type
        )
    }
}

/// Call signature (for callable interfaces)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceCallSignature {
    /// Parameters
    pub params: Vec<ParamDef>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Type parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,
}

impl InterfaceCallSignature {
    /// Create a new call signature
    pub fn new(params: Vec<ParamDef>, return_type: Option<EtchType>) -> Self {
        Self {
            params,
            return_type,
            type_params: vec![],
            doc: None,
        }
    }

    /// Generate TypeScript signature
    pub fn to_typescript(&self) -> String {
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

        format!("{}({}){})", type_params, params.join(", "), return_type)
    }
}

/// Constructor signature (for newable interfaces)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConstructSignature {
    /// Parameters
    pub params: Vec<ParamDef>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Type parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,
}

impl InterfaceConstructSignature {
    /// Create a new construct signature
    pub fn new(params: Vec<ParamDef>, return_type: Option<EtchType>) -> Self {
        Self {
            params,
            return_type,
            type_params: vec![],
        }
    }

    /// Generate TypeScript signature
    pub fn to_typescript(&self) -> String {
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

        format!("new {}({}){})", type_params, params.join(", "), return_type)
    }
}

/// Index signature (e.g., [key: string]: any)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceIndexSignature {
    /// Index parameter name
    pub param_name: String,

    /// Index type (usually string or number)
    pub index_type: EtchType,

    /// Value type
    pub value_type: EtchType,

    /// Whether this is readonly
    #[serde(default)]
    pub readonly: bool,
}

impl InterfaceIndexSignature {
    /// Create a new index signature
    pub fn new(param_name: impl Into<String>, index_type: EtchType, value_type: EtchType) -> Self {
        Self {
            param_name: param_name.into(),
            index_type,
            value_type,
            readonly: false,
        }
    }

    /// Mark as readonly
    pub fn as_readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Mark as readonly if condition is true
    pub fn as_readonly_if(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Generate TypeScript signature
    pub fn to_typescript(&self) -> String {
        let readonly = if self.readonly { "readonly " } else { "" };
        format!(
            "{}[{}: {}]: {}",
            readonly,
            self.param_name,
            self.index_type.to_typescript(),
            self.value_type.to_typescript()
        )
    }
}

/// Interface definition
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceDef {
    /// Internal definition name (for default exports)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub def_name: Option<String>,

    /// Properties
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<InterfacePropertyDef>,

    /// Methods
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<InterfaceMethodDef>,

    /// Call signatures (for callable interfaces)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call_signatures: Vec<InterfaceCallSignature>,

    /// Construct signatures (for newable interfaces)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub construct_signatures: Vec<InterfaceConstructSignature>,

    /// Index signatures
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index_signatures: Vec<InterfaceIndexSignature>,

    /// Type parameters (generics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Extended interfaces
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<EtchType>,
}

impl InterfaceDef {
    /// Create a new interface definition
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the definition name
    pub fn with_def_name(mut self, name: impl Into<String>) -> Self {
        self.def_name = Some(name.into());
        self
    }

    /// Add a property
    pub fn with_property(mut self, prop: InterfacePropertyDef) -> Self {
        self.properties.push(prop);
        self
    }

    /// Add a method
    pub fn with_method(mut self, method: InterfaceMethodDef) -> Self {
        self.methods.push(method);
        self
    }

    /// Add a call signature
    pub fn with_call_signature(mut self, sig: InterfaceCallSignature) -> Self {
        self.call_signatures.push(sig);
        self
    }

    /// Add a construct signature
    pub fn with_construct_signature(mut self, sig: InterfaceConstructSignature) -> Self {
        self.construct_signatures.push(sig);
        self
    }

    /// Add an index signature
    pub fn with_index_signature(mut self, sig: InterfaceIndexSignature) -> Self {
        self.index_signatures.push(sig);
        self
    }

    /// Add extended interface
    pub fn extends(mut self, interface: EtchType) -> Self {
        self.extends.push(interface);
        self
    }

    /// Add type parameters
    pub fn with_type_params(mut self, params: Vec<TsTypeParamDef>) -> Self {
        self.type_params = params;
        self
    }

    /// Check if interface is empty
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
            && self.methods.is_empty()
            && self.call_signatures.is_empty()
            && self.construct_signatures.is_empty()
            && self.index_signatures.is_empty()
    }

    /// Get all member count
    pub fn member_count(&self) -> usize {
        self.properties.len()
            + self.methods.len()
            + self.call_signatures.len()
            + self.construct_signatures.len()
            + self.index_signatures.len()
    }
}

// Type aliases for backwards compatibility with parser.rs
pub type CallSignatureDef = InterfaceCallSignature;
pub type ConstructSignatureDef = InterfaceConstructSignature;
pub type IndexSignatureDef = InterfaceIndexSignature;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interface_property() {
        let prop = InterfacePropertyDef::new("name")
            .with_type(EtchType::string())
            .as_readonly();

        assert_eq!(prop.to_typescript(), "readonly name: string");
    }

    #[test]
    fn test_interface_property_optional() {
        let prop = InterfacePropertyDef::new("age")
            .with_type(EtchType::number())
            .as_optional();

        assert_eq!(prop.to_typescript(), "age?: number");
    }

    #[test]
    fn test_interface_method() {
        let method = InterfaceMethodDef::new("greet")
            .with_param(ParamDef::new("name", EtchType::string()))
            .with_return_type(EtchType::string());

        assert_eq!(method.to_typescript(), "greet(name: string): string");
    }

    #[test]
    fn test_index_signature() {
        let sig = InterfaceIndexSignature::new("key", EtchType::string(), EtchType::unknown());

        assert_eq!(sig.to_typescript(), "[key: string]: unknown");
    }

    #[test]
    fn test_interface_def() {
        let iface = InterfaceDef::new()
            .with_property(InterfacePropertyDef::new("name").with_type(EtchType::string()))
            .with_method(InterfaceMethodDef::new("getName").with_return_type(EtchType::string()))
            .extends(EtchType::simple_ref("Serializable"));

        assert_eq!(iface.properties.len(), 1);
        assert_eq!(iface.methods.len(), 1);
        assert_eq!(iface.extends.len(), 1);
    }
}
