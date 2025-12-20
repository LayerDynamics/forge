//! Class definitions
//!
//! This module provides types for representing TypeScript classes
//! in documentation, including properties, methods, constructors,
//! and class inheritance.

use crate::decorators::DecoratorDef;
use crate::function::FunctionDef;
use crate::params::ParamDef;
use crate::ts_type_params::TsTypeParamDef;
use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// Accessibility modifier for class members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accessibility {
    /// Public (default)
    #[default]
    Public,
    /// Protected
    Protected,
    /// Private
    Private,
}

impl Accessibility {
    /// Get TypeScript keyword
    pub fn keyword(&self) -> Option<&'static str> {
        match self {
            Accessibility::Public => None, // Default, no keyword needed
            Accessibility::Protected => Some("protected"),
            Accessibility::Private => Some("private"),
        }
    }
}

/// Class property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassPropertyDef {
    /// Property name
    pub name: String,

    /// TypeScript name (for display)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ts_name: Option<String>,

    /// Property type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ts_type: Option<EtchType>,

    /// Whether this is readonly
    #[serde(default)]
    pub readonly: bool,

    /// Whether this is optional
    #[serde(default)]
    pub is_optional: bool,

    /// Accessibility modifier (public, protected, private)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub accessibility: Option<String>,

    /// Whether this is static
    #[serde(default)]
    pub is_static: bool,

    /// Whether this is abstract
    #[serde(default)]
    pub is_abstract: bool,

    /// Whether this is an override
    #[serde(default)]
    pub is_override: bool,

    /// Whether this is a computed property
    #[serde(default)]
    pub computed: bool,

    /// Decorators applied to this property
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<DecoratorDef>,

    /// Property documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<crate::js_doc::EtchDoc>,

    /// Whether this has a default value
    #[serde(default)]
    pub has_default: bool,
}

impl PartialEq for ClassPropertyDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.ts_name == other.ts_name
            && self.ts_type == other.ts_type
            && self.readonly == other.readonly
            && self.is_optional == other.is_optional
            && self.accessibility == other.accessibility
            && self.is_static == other.is_static
            && self.is_abstract == other.is_abstract
            && self.is_override == other.is_override
            && self.computed == other.computed
            && self.decorators == other.decorators
            && self.has_default == other.has_default
    }
}

impl ClassPropertyDef {
    /// Create a new property definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ts_name: None,
            ts_type: None,
            readonly: false,
            is_optional: false,
            accessibility: None,
            is_static: false,
            is_abstract: false,
            is_override: false,
            computed: false,
            decorators: vec![],
            doc: None,
            has_default: false,
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
        self.is_optional = true;
        self
    }

    /// Set accessibility
    pub fn with_accessibility(mut self, acc: impl Into<String>) -> Self {
        self.accessibility = Some(acc.into());
        self
    }

    /// Mark as static
    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    /// Mark as abstract
    pub fn as_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: crate::js_doc::EtchDoc) -> Self {
        self.doc = Some(doc);
        self
    }

    /// Get the effective name
    pub fn effective_name(&self) -> &str {
        self.ts_name.as_deref().unwrap_or(&self.name)
    }

    /// Generate TypeScript declaration
    pub fn to_typescript(&self) -> String {
        let mut parts = vec![];

        if let Some(ref acc) = self.accessibility {
            if acc != "public" {
                parts.push(acc.clone());
            }
        }
        if self.is_static {
            parts.push("static".to_string());
        }
        if self.is_abstract {
            parts.push("abstract".to_string());
        }
        if self.is_override {
            parts.push("override".to_string());
        }
        if self.readonly {
            parts.push("readonly".to_string());
        }

        let name = self.effective_name();
        let opt = if self.is_optional { "?" } else { "" };
        let type_str = self
            .ts_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        parts.push(format!("{}{}{}", name, opt, type_str));

        parts.join(" ")
    }
}

/// Class method definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMethodDef {
    /// Method name
    pub name: String,

    /// Method kind (method, getter, setter)
    #[serde(default)]
    pub kind: MethodKind,

    /// Accessibility modifier (public, protected, private)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub accessibility: Option<String>,

    /// Whether this is static
    #[serde(default)]
    pub is_static: bool,

    /// Whether this is abstract
    #[serde(default)]
    pub is_abstract: bool,

    /// Whether this is an override
    #[serde(default)]
    pub is_override: bool,

    /// Whether this is optional
    #[serde(default)]
    pub is_optional: bool,

    /// Decorators applied to this method
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<DecoratorDef>,

    /// Method documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<crate::js_doc::EtchDoc>,

    /// Type parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Return type
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub return_type: Option<EtchType>,

    /// Method parameters
    #[serde(default)]
    pub params: Vec<ParamDef>,
}

impl PartialEq for ClassMethodDef {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.kind == other.kind
            && self.accessibility == other.accessibility
            && self.is_static == other.is_static
            && self.is_abstract == other.is_abstract
            && self.is_override == other.is_override
            && self.is_optional == other.is_optional
            && self.decorators == other.decorators
            && self.type_params == other.type_params
            && self.return_type == other.return_type
            && self.params == other.params
    }
}

/// Method kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MethodKind {
    /// Normal method
    #[default]
    Method,
    /// Getter (get foo())
    Getter,
    /// Setter (set foo(value))
    Setter,
}

impl ClassMethodDef {
    /// Create a new method definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: MethodKind::Method,
            accessibility: None,
            is_static: false,
            is_abstract: false,
            is_override: false,
            is_optional: false,
            decorators: vec![],
            doc: None,
            type_params: vec![],
            return_type: None,
            params: vec![],
        }
    }

    /// Create from a FunctionDef
    pub fn from_function_def(name: impl Into<String>, func: FunctionDef) -> Self {
        Self {
            name: name.into(),
            kind: MethodKind::Method,
            accessibility: None,
            is_static: false,
            is_abstract: false,
            is_override: false,
            is_optional: false,
            decorators: func.decorators,
            doc: None,
            type_params: func.type_params,
            return_type: func.return_type,
            params: func.params,
        }
    }

    /// Create a getter
    pub fn getter(name: impl Into<String>, return_type: EtchType) -> Self {
        Self {
            name: name.into(),
            kind: MethodKind::Getter,
            accessibility: None,
            is_static: false,
            is_abstract: false,
            is_override: false,
            is_optional: false,
            decorators: vec![],
            doc: None,
            type_params: vec![],
            return_type: Some(return_type),
            params: vec![],
        }
    }

    /// Create a setter
    pub fn setter(name: impl Into<String>, param: ParamDef) -> Self {
        Self {
            name: name.into(),
            kind: MethodKind::Setter,
            accessibility: None,
            is_static: false,
            is_abstract: false,
            is_override: false,
            is_optional: false,
            decorators: vec![],
            doc: None,
            type_params: vec![],
            return_type: None,
            params: vec![param],
        }
    }

    /// Set accessibility
    pub fn with_accessibility(mut self, acc: impl Into<String>) -> Self {
        self.accessibility = Some(acc.into());
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

    /// Mark as static
    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    /// Mark as abstract
    pub fn as_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: crate::js_doc::EtchDoc) -> Self {
        self.doc = Some(doc);
        self
    }

    /// Generate TypeScript signature
    pub fn to_typescript_signature(&self) -> String {
        let mut parts = vec![];

        if let Some(ref acc) = self.accessibility {
            if acc != "public" {
                parts.push(acc.clone());
            }
        }
        if self.is_static {
            parts.push("static".to_string());
        }
        if self.is_abstract {
            parts.push("abstract".to_string());
        }
        if self.is_override {
            parts.push("override".to_string());
        }

        let kind_prefix = match self.kind {
            MethodKind::Method => "",
            MethodKind::Getter => "get ",
            MethodKind::Setter => "set ",
        };

        let type_params_str = if self.type_params.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = self.type_params.iter().map(|p| p.to_typescript()).collect();
            format!("<{}>", params.join(", "))
        };

        let params: Vec<String> = self.params.iter().map(|p| p.to_typescript()).collect();

        let return_type_str = self
            .return_type
            .as_ref()
            .map(|t| format!(": {}", t.to_typescript()))
            .unwrap_or_default();

        parts.push(format!(
            "{}{}{}({}){}",
            kind_prefix,
            self.name,
            type_params_str,
            params.join(", "),
            return_type_str
        ));

        parts.join(" ")
    }
}

/// Constructor definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConstructorDef {
    /// Constructor parameters
    pub params: Vec<ParamDef>,

    /// Documentation
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<crate::js_doc::EtchDoc>,

    /// Accessibility modifier
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub accessibility: Option<String>,

    /// Whether constructor has a body
    #[serde(default)]
    pub has_body: bool,
}

impl PartialEq for ClassConstructorDef {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params
            && self.accessibility == other.accessibility
            && self.has_body == other.has_body
    }
}

/// Constructor parameter (can be a property promotion)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConstructorParam {
    /// Parameter definition
    pub param: ParamDef,

    /// Whether this is a property promotion (has accessibility or readonly)
    #[serde(default)]
    pub is_property: bool,

    /// Accessibility (if property)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub accessibility: Option<Accessibility>,

    /// Whether readonly (if property)
    #[serde(default)]
    pub readonly: bool,
}

impl ClassConstructorDef {
    /// Create a new constructor
    pub fn new(params: Vec<ParamDef>) -> Self {
        Self {
            params,
            doc: None,
            accessibility: None,
            has_body: true,
        }
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: crate::js_doc::EtchDoc) -> Self {
        self.doc = Some(doc);
        self
    }

    /// Set accessibility
    pub fn with_accessibility(mut self, acc: impl Into<String>) -> Self {
        self.accessibility = Some(acc.into());
        self
    }
}

/// Index signature (e.g., [key: string]: any)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassIndexSignature {
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

/// Class definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct ClassDef {
    /// Internal definition name (for default exports)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub def_name: Option<String>,

    /// Constructors (can have multiple overloads)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constructors: Vec<ClassConstructorDef>,

    /// Properties
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<ClassPropertyDef>,

    /// Methods
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<ClassMethodDef>,

    /// Index signatures
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index_signatures: Vec<ClassIndexSignature>,

    /// Type parameters (generics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,

    /// Superclass (extends)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub extends: Option<EtchType>,

    /// Implemented interfaces
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<EtchType>,

    /// Whether this is an abstract class
    #[serde(default)]
    pub is_abstract: bool,

    /// Decorators applied to this class
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decorators: Vec<DecoratorDef>,
}

impl ClassDef {
    /// Create a new class definition
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the definition name
    pub fn with_def_name(mut self, name: impl Into<String>) -> Self {
        self.def_name = Some(name.into());
        self
    }

    /// Add a constructor
    pub fn with_constructor(mut self, ctor: ClassConstructorDef) -> Self {
        self.constructors.push(ctor);
        self
    }

    /// Add a property
    pub fn with_property(mut self, prop: ClassPropertyDef) -> Self {
        self.properties.push(prop);
        self
    }

    /// Add a method
    pub fn with_method(mut self, method: ClassMethodDef) -> Self {
        self.methods.push(method);
        self
    }

    /// Set superclass
    pub fn extends(mut self, superclass: EtchType) -> Self {
        self.extends = Some(superclass);
        self
    }

    /// Add implemented interface
    pub fn implements(mut self, interface: EtchType) -> Self {
        self.implements.push(interface);
        self
    }

    /// Mark as abstract
    pub fn as_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }

    /// Add type parameters
    pub fn with_type_params(mut self, params: Vec<TsTypeParamDef>) -> Self {
        self.type_params = params;
        self
    }

    /// Get all public properties
    pub fn public_properties(&self) -> Vec<&ClassPropertyDef> {
        self.properties
            .iter()
            .filter(|p| p.accessibility.is_none() || p.accessibility.as_deref() == Some("public"))
            .collect()
    }

    /// Get all public methods
    pub fn public_methods(&self) -> Vec<&ClassMethodDef> {
        self.methods
            .iter()
            .filter(|m| m.accessibility.is_none() || m.accessibility.as_deref() == Some("public"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_property() {
        let prop = ClassPropertyDef::new("name")
            .with_type(EtchType::string())
            .as_readonly();

        assert_eq!(prop.to_typescript(), "readonly name: string");
    }

    #[test]
    fn test_class_method() {
        let method = ClassMethodDef::new("setValue")
            .with_param(ParamDef::new("value", EtchType::string()))
            .with_return_type(EtchType::void());

        assert_eq!(
            method.to_typescript_signature(),
            "setValue(value: string): void"
        );
    }

    #[test]
    fn test_getter_setter() {
        let getter = ClassMethodDef::getter("count", EtchType::number());
        assert!(getter.to_typescript_signature().contains("get count"));

        let setter = ClassMethodDef::setter("count", ParamDef::new("value", EtchType::number()));
        assert!(setter.to_typescript_signature().contains("set count"));
    }

    #[test]
    fn test_class_def() {
        let class = ClassDef::new()
            .with_property(ClassPropertyDef::new("name").with_type(EtchType::string()))
            .extends(EtchType::simple_ref("BaseClass"))
            .implements(EtchType::simple_ref("Serializable"));

        assert!(class.extends.is_some());
        assert_eq!(class.implements.len(), 1);
        assert_eq!(class.properties.len(), 1);
    }
}
