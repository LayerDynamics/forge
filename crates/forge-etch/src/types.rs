//! Type system for documentation
//!
//! This module provides `EtchType`, an extended type representation that
//! includes documentation metadata. It can convert from forge-weld's `WeldType`
//! while adding doc-specific information.

use forge_weld::ir::{WeldPrimitive, WeldType};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Primitive types in the documentation type system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EtchPrimitive {
    /// string type
    String,
    /// number type (for all numeric types that fit in JS number)
    Number,
    /// bigint type (for i64/u64)
    BigInt,
    /// boolean type
    Boolean,
    /// void/undefined type
    Void,
    /// null type
    Null,
    /// undefined type
    Undefined,
    /// never type
    Never,
    /// any type
    Any,
    /// unknown type
    Unknown,
    /// symbol type
    Symbol,
    /// object type (non-primitive)
    Object,
}

impl EtchPrimitive {
    /// Convert to TypeScript type string
    pub fn to_typescript(&self) -> &'static str {
        match self {
            EtchPrimitive::String => "string",
            EtchPrimitive::Number => "number",
            EtchPrimitive::BigInt => "bigint",
            EtchPrimitive::Boolean => "boolean",
            EtchPrimitive::Void => "void",
            EtchPrimitive::Null => "null",
            EtchPrimitive::Undefined => "undefined",
            EtchPrimitive::Never => "never",
            EtchPrimitive::Any => "any",
            EtchPrimitive::Unknown => "unknown",
            EtchPrimitive::Symbol => "symbol",
            EtchPrimitive::Object => "object",
        }
    }

    /// Parse from TypeScript type string
    pub fn from_typescript(s: &str) -> Option<Self> {
        match s {
            "string" => Some(EtchPrimitive::String),
            "number" => Some(EtchPrimitive::Number),
            "bigint" => Some(EtchPrimitive::BigInt),
            "boolean" => Some(EtchPrimitive::Boolean),
            "void" => Some(EtchPrimitive::Void),
            "null" => Some(EtchPrimitive::Null),
            "undefined" => Some(EtchPrimitive::Undefined),
            "never" => Some(EtchPrimitive::Never),
            "any" => Some(EtchPrimitive::Any),
            "unknown" => Some(EtchPrimitive::Unknown),
            "symbol" => Some(EtchPrimitive::Symbol),
            "object" => Some(EtchPrimitive::Object),
            _ => None,
        }
    }
}

impl fmt::Display for EtchPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_typescript())
    }
}

/// Convert from WeldPrimitive
impl From<&WeldPrimitive> for EtchPrimitive {
    fn from(weld: &WeldPrimitive) -> Self {
        match weld {
            WeldPrimitive::U8
            | WeldPrimitive::U16
            | WeldPrimitive::U32
            | WeldPrimitive::I8
            | WeldPrimitive::I16
            | WeldPrimitive::I32
            | WeldPrimitive::F32
            | WeldPrimitive::F64
            | WeldPrimitive::Usize
            | WeldPrimitive::Isize => EtchPrimitive::Number,
            WeldPrimitive::U64 | WeldPrimitive::I64 => EtchPrimitive::BigInt,
            WeldPrimitive::Bool => EtchPrimitive::Boolean,
            WeldPrimitive::String | WeldPrimitive::Str | WeldPrimitive::Char => {
                EtchPrimitive::String
            }
            WeldPrimitive::Unit => EtchPrimitive::Void,
        }
    }
}

/// Literal type values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EtchLiteral {
    /// String literal (e.g., "hello")
    String(String),
    /// Number literal (e.g., 42)
    Number(f64),
    /// BigInt literal (e.g., 42n)
    BigInt(i64),
    /// Boolean literal (true/false)
    Boolean(bool),
    /// Template literal type
    Template(Vec<TemplatePart>),
}

/// Part of a template literal type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemplatePart {
    /// Static string part
    String(String),
    /// Type substitution part
    Type(Box<EtchType>),
}

impl EtchLiteral {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> String {
        match self {
            EtchLiteral::String(s) => format!("\"{}\"", s),
            EtchLiteral::Number(n) => n.to_string(),
            EtchLiteral::BigInt(n) => format!("{}n", n),
            EtchLiteral::Boolean(b) => b.to_string(),
            EtchLiteral::Template(parts) => {
                let mut result = String::from("`");
                for part in parts {
                    match part {
                        TemplatePart::String(s) => result.push_str(s),
                        TemplatePart::Type(t) => {
                            result.push_str("${");
                            result.push_str(&t.to_typescript());
                            result.push('}');
                        }
                    }
                }
                result.push('`');
                result
            }
        }
    }
}

/// Function type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTypeDef {
    /// Type parameters
    pub type_params: Vec<TypeParamDef>,
    /// Parameters
    pub params: Vec<FunctionTypeParam>,
    /// Return type
    pub return_type: Box<EtchType>,
    /// Whether this is a constructor signature
    pub is_constructor: bool,
}

/// Function type parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTypeParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: EtchType,
    /// Whether optional
    pub optional: bool,
}

/// Type parameter definition (generic)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeParamDef {
    /// Parameter name (e.g., "T")
    pub name: String,
    /// Constraint (extends clause)
    pub constraint: Option<Box<EtchType>>,
    /// Default value
    pub default: Option<Box<EtchType>>,
}

/// Complex type kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EtchTypeKind {
    /// Primitive type
    Primitive(EtchPrimitive),

    /// Array type (T[])
    Array(Box<EtchType>),

    /// Tuple type ([A, B, C])
    Tuple(Vec<EtchType>),

    /// Object/Record type (Record<K, V>)
    Object {
        key: Box<EtchType>,
        value: Box<EtchType>,
    },

    /// Union type (A | B | C)
    Union(Vec<EtchType>),

    /// Intersection type (A & B & C)
    Intersection(Vec<EtchType>),

    /// Type reference (MyType or MyType<T, U>)
    TypeRef {
        name: String,
        type_params: Vec<EtchType>,
    },

    /// Literal type
    Literal(EtchLiteral),

    /// Function type ((a: A) => B)
    Function(Box<FunctionTypeDef>),

    /// Promise<T>
    Promise(Box<EtchType>),

    /// Set<T>
    Set(Box<EtchType>),

    /// Map<K, V>
    Map {
        key: Box<EtchType>,
        value: Box<EtchType>,
    },

    /// Uint8Array (special case for Vec<u8>)
    Uint8Array,

    /// Type parameter reference (T in generic context)
    TypeParam(String),

    /// Conditional type (T extends U ? X : Y)
    Conditional {
        check_type: Box<EtchType>,
        extends_type: Box<EtchType>,
        true_type: Box<EtchType>,
        false_type: Box<EtchType>,
    },

    /// Indexed access type (T[K])
    IndexedAccess {
        obj_type: Box<EtchType>,
        index_type: Box<EtchType>,
    },

    /// Mapped type ({ [K in keyof T]: ... })
    Mapped {
        type_param: String,
        name_type: Option<Box<EtchType>>,
        value_type: Box<EtchType>,
        optional: Option<bool>,
        readonly: Option<bool>,
        template: Option<Box<EtchType>>,
        constraint: Option<Box<EtchType>>,
    },

    /// Type operator (keyof T, typeof x, readonly T[])
    TypeOperator {
        operator: TypeOperator,
        type_arg: Box<EtchType>,
    },

    /// Infer type (infer T)
    Infer(String),

    /// Parenthesized type
    Parenthesized(Box<EtchType>),

    /// This type
    This,

    /// Constructor type (new (...) => T)
    Constructor {
        params: Vec<crate::params::ParamDef>,
        return_type: Box<EtchType>,
        type_params: Vec<crate::ts_type_params::TsTypeParamDef>,
    },

    /// Type literal ({ prop: T, method(): R })
    TypeLiteral {
        properties: Vec<TypeLiteralProperty>,
        methods: Vec<TypeLiteralMethod>,
    },

    /// Optional type (T?)
    Optional(Box<EtchType>),

    /// Rest type (...T)
    Rest(Box<EtchType>),

    /// Type query (typeof x)
    TypeQuery(String),

    /// Import type (import("module").Type)
    Import {
        arg: String,
        qualifier: Option<String>,
    },

    /// Type predicate (x is T)
    TypePredicate {
        param_name: String,
        ts_type: Option<Box<EtchType>>,
        asserts: bool,
    },

    /// Unknown/any fallback
    #[default]
    Unknown,
}

/// Property in a type literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeLiteralProperty {
    pub name: String,
    pub ts_type: Option<EtchType>,
    pub optional: bool,
    pub readonly: bool,
}

/// Method in a type literal
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeLiteralMethod {
    pub name: String,
    pub params: Vec<crate::params::ParamDef>,
    pub return_type: Option<EtchType>,
    pub optional: bool,
}

/// Type operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TypeOperator {
    /// keyof T
    KeyOf,
    /// typeof x
    TypeOf,
    /// readonly T[]
    Readonly,
    /// unique symbol
    Unique,
}

impl TypeOperator {
    /// Get operator keyword
    pub fn keyword(&self) -> &'static str {
        match self {
            TypeOperator::KeyOf => "keyof",
            TypeOperator::TypeOf => "typeof",
            TypeOperator::Readonly => "readonly",
            TypeOperator::Unique => "unique",
        }
    }
}

/// Extended type with documentation metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EtchType {
    /// The underlying type representation
    pub kind: EtchTypeKind,

    /// Optional documentation for this type usage
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub doc: Option<String>,

    /// Whether this type is optional (from context, not T | undefined)
    #[serde(default)]
    pub optional: bool,

    /// Whether this is nullable (T | null)
    #[serde(default)]
    pub nullable: bool,
}

impl EtchType {
    /// Create a new type
    pub fn new(kind: EtchTypeKind) -> Self {
        Self {
            kind,
            doc: None,
            optional: false,
            nullable: false,
        }
    }

    /// Create a primitive type
    pub fn primitive(p: EtchPrimitive) -> Self {
        Self::new(EtchTypeKind::Primitive(p))
    }

    /// Create a string type
    pub fn string() -> Self {
        Self::primitive(EtchPrimitive::String)
    }

    /// Create a number type
    pub fn number() -> Self {
        Self::primitive(EtchPrimitive::Number)
    }

    /// Create a boolean type
    pub fn boolean() -> Self {
        Self::primitive(EtchPrimitive::Boolean)
    }

    /// Create a void type
    pub fn void() -> Self {
        Self::primitive(EtchPrimitive::Void)
    }

    /// Create an any type
    pub fn any() -> Self {
        Self::primitive(EtchPrimitive::Any)
    }

    /// Create an unknown type
    pub fn unknown() -> Self {
        Self::primitive(EtchPrimitive::Unknown)
    }

    /// Create a never type
    pub fn never() -> Self {
        Self::primitive(EtchPrimitive::Never)
    }

    /// Create an array type
    pub fn array(element: EtchType) -> Self {
        Self::new(EtchTypeKind::Array(Box::new(element)))
    }

    /// Create a tuple type
    pub fn tuple(elements: Vec<EtchType>) -> Self {
        Self::new(EtchTypeKind::Tuple(elements))
    }

    /// Create a union type
    pub fn union(types: Vec<EtchType>) -> Self {
        Self::new(EtchTypeKind::Union(types))
    }

    /// Create an intersection type
    pub fn intersection(types: Vec<EtchType>) -> Self {
        Self::new(EtchTypeKind::Intersection(types))
    }

    /// Create a type reference
    pub fn type_ref(name: impl Into<String>, type_params: Vec<EtchType>) -> Self {
        Self::new(EtchTypeKind::TypeRef {
            name: name.into(),
            type_params,
        })
    }

    /// Create a simple type reference (no type params)
    pub fn simple_ref(name: impl Into<String>) -> Self {
        Self::type_ref(name, vec![])
    }

    /// Create a Promise type
    pub fn promise(inner: EtchType) -> Self {
        Self::new(EtchTypeKind::Promise(Box::new(inner)))
    }

    /// Create a Record type
    pub fn record(key: EtchType, value: EtchType) -> Self {
        Self::new(EtchTypeKind::Object {
            key: Box::new(key),
            value: Box::new(value),
        })
    }

    /// Create a Set type
    pub fn set(element: EtchType) -> Self {
        Self::new(EtchTypeKind::Set(Box::new(element)))
    }

    /// Create a Map type
    pub fn map(key: EtchType, value: EtchType) -> Self {
        Self::new(EtchTypeKind::Map {
            key: Box::new(key),
            value: Box::new(value),
        })
    }

    /// Create a Uint8Array type
    pub fn uint8_array() -> Self {
        Self::new(EtchTypeKind::Uint8Array)
    }

    /// Create a function type
    pub fn function(func: FunctionTypeDef) -> Self {
        Self::new(EtchTypeKind::Function(Box::new(func)))
    }

    /// Create a literal type
    pub fn literal(lit: EtchLiteral) -> Self {
        Self::new(EtchTypeKind::Literal(lit))
    }

    /// Mark as optional
    pub fn as_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark as nullable
    pub fn as_nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Add documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Convert to TypeScript type string
    pub fn to_typescript(&self) -> String {
        let base = self.kind_to_typescript();

        if self.nullable && self.optional {
            format!("{} | null | undefined", base)
        } else if self.nullable {
            format!("{} | null", base)
        } else if self.optional {
            format!("{} | undefined", base)
        } else {
            base
        }
    }

    /// Convert kind to TypeScript (without nullable/optional)
    fn kind_to_typescript(&self) -> String {
        match &self.kind {
            EtchTypeKind::Primitive(p) => p.to_typescript().to_string(),
            EtchTypeKind::Array(inner) => {
                let inner_ts = inner.to_typescript();
                // Wrap complex types in parens for array notation
                if matches!(
                    inner.kind,
                    EtchTypeKind::Union(_)
                        | EtchTypeKind::Intersection(_)
                        | EtchTypeKind::Function(_)
                ) {
                    format!("({})[]", inner_ts)
                } else {
                    format!("{}[]", inner_ts)
                }
            }
            EtchTypeKind::Tuple(elements) => {
                let types: Vec<String> = elements.iter().map(|t| t.to_typescript()).collect();
                format!("[{}]", types.join(", "))
            }
            EtchTypeKind::Object { key, value } => {
                format!("Record<{}, {}>", key.to_typescript(), value.to_typescript())
            }
            EtchTypeKind::Union(types) => {
                let types: Vec<String> = types.iter().map(|t| t.to_typescript()).collect();
                types.join(" | ")
            }
            EtchTypeKind::Intersection(types) => {
                let types: Vec<String> = types.iter().map(|t| t.to_typescript()).collect();
                types.join(" & ")
            }
            EtchTypeKind::TypeRef { name, type_params } => {
                if type_params.is_empty() {
                    name.clone()
                } else {
                    let params: Vec<String> =
                        type_params.iter().map(|t| t.to_typescript()).collect();
                    format!("{}<{}>", name, params.join(", "))
                }
            }
            EtchTypeKind::Literal(lit) => lit.to_typescript(),
            EtchTypeKind::Function(func) => {
                let params: Vec<String> = func
                    .params
                    .iter()
                    .map(|p| {
                        let opt = if p.optional { "?" } else { "" };
                        format!("{}{}: {}", p.name, opt, p.param_type.to_typescript())
                    })
                    .collect();

                if func.is_constructor {
                    format!(
                        "new ({}) => {}",
                        params.join(", "),
                        func.return_type.to_typescript()
                    )
                } else {
                    format!(
                        "({}) => {}",
                        params.join(", "),
                        func.return_type.to_typescript()
                    )
                }
            }
            EtchTypeKind::Promise(inner) => {
                format!("Promise<{}>", inner.to_typescript())
            }
            EtchTypeKind::Set(inner) => {
                format!("Set<{}>", inner.to_typescript())
            }
            EtchTypeKind::Map { key, value } => {
                format!("Map<{}, {}>", key.to_typescript(), value.to_typescript())
            }
            EtchTypeKind::Uint8Array => "Uint8Array".to_string(),
            EtchTypeKind::TypeParam(name) => name.clone(),
            EtchTypeKind::Conditional {
                check_type,
                extends_type,
                true_type,
                false_type,
            } => {
                format!(
                    "{} extends {} ? {} : {}",
                    check_type.to_typescript(),
                    extends_type.to_typescript(),
                    true_type.to_typescript(),
                    false_type.to_typescript()
                )
            }
            EtchTypeKind::IndexedAccess {
                obj_type,
                index_type,
            } => {
                format!(
                    "{}[{}]",
                    obj_type.to_typescript(),
                    index_type.to_typescript()
                )
            }
            EtchTypeKind::Mapped {
                type_param,
                value_type,
                optional,
                readonly,
                ..
            } => {
                let readonly_str = if *readonly == Some(true) {
                    "readonly "
                } else {
                    ""
                };
                let optional_str = match optional {
                    Some(true) => "?",
                    Some(false) => "-?",
                    None => "",
                };
                format!(
                    "{{ {}[{} in keyof T]{}: {} }}",
                    readonly_str,
                    type_param,
                    optional_str,
                    value_type.to_typescript()
                )
            }
            EtchTypeKind::TypeOperator { operator, type_arg } => {
                format!("{} {}", operator.keyword(), type_arg.to_typescript())
            }
            EtchTypeKind::Infer(name) => format!("infer {}", name),
            EtchTypeKind::Parenthesized(inner) => format!("({})", inner.to_typescript()),
            EtchTypeKind::This => "this".to_string(),
            EtchTypeKind::Constructor {
                params,
                return_type,
                type_params,
            } => {
                let type_params_str = if type_params.is_empty() {
                    String::new()
                } else {
                    let tps: Vec<String> = type_params.iter().map(|p| p.to_typescript()).collect();
                    format!("<{}>", tps.join(", "))
                };
                let params_str: Vec<String> = params.iter().map(|p| p.to_typescript()).collect();
                format!(
                    "new {}({}) => {}",
                    type_params_str,
                    params_str.join(", "),
                    return_type.to_typescript()
                )
            }
            EtchTypeKind::TypeLiteral {
                properties,
                methods,
            } => {
                let mut parts = Vec::new();
                for prop in properties {
                    let readonly = if prop.readonly { "readonly " } else { "" };
                    let opt = if prop.optional { "?" } else { "" };
                    let ty = prop
                        .ts_type
                        .as_ref()
                        .map(|t| format!(": {}", t.to_typescript()))
                        .unwrap_or_default();
                    parts.push(format!("{}{}{}{}", readonly, prop.name, opt, ty));
                }
                for method in methods {
                    let opt = if method.optional { "?" } else { "" };
                    let method_params: Vec<String> =
                        method.params.iter().map(|p| p.to_typescript()).collect();
                    let ret = method
                        .return_type
                        .as_ref()
                        .map(|t| format!(": {}", t.to_typescript()))
                        .unwrap_or_default();
                    parts.push(format!(
                        "{}{}({}){}",
                        method.name,
                        opt,
                        method_params.join(", "),
                        ret
                    ));
                }
                format!("{{ {} }}", parts.join("; "))
            }
            EtchTypeKind::Optional(inner) => format!("{}?", inner.to_typescript()),
            EtchTypeKind::Rest(inner) => format!("...{}", inner.to_typescript()),
            EtchTypeKind::TypeQuery(name) => format!("typeof {}", name),
            EtchTypeKind::Import { arg, qualifier } => match qualifier {
                Some(q) => format!("import(\"{}\").{}", arg, q),
                None => format!("import(\"{}\")", arg),
            },
            EtchTypeKind::TypePredicate {
                param_name,
                ts_type,
                asserts,
            } => match (asserts, ts_type) {
                (true, Some(ty)) => format!("asserts {} is {}", param_name, ty.to_typescript()),
                (true, None) => format!("asserts {}", param_name),
                (false, Some(ty)) => format!("{} is {}", param_name, ty.to_typescript()),
                (false, None) => param_name.clone(),
            },
            EtchTypeKind::Unknown => "unknown".to_string(),
        }
    }

    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, EtchTypeKind::Primitive(_))
    }

    /// Check if this is a Promise type
    pub fn is_promise(&self) -> bool {
        matches!(self.kind, EtchTypeKind::Promise(_))
    }

    /// Get the inner type of a Promise
    pub fn promise_inner(&self) -> Option<&EtchType> {
        match &self.kind {
            EtchTypeKind::Promise(inner) => Some(inner),
            _ => None,
        }
    }

    /// Get the type name if this is a named type reference
    pub fn type_name(&self) -> Option<String> {
        match &self.kind {
            EtchTypeKind::TypeRef { name, .. } => Some(name.clone()),
            EtchTypeKind::Primitive(p) => Some(p.to_typescript().to_string()),
            _ => None,
        }
    }

    /// Collect all referenced type names in this type
    pub fn referenced_types(&self) -> Vec<String> {
        let mut refs = Vec::new();
        self.collect_referenced_types(&mut refs);
        refs
    }

    /// Helper to recursively collect referenced types
    fn collect_referenced_types(&self, refs: &mut Vec<String>) {
        match &self.kind {
            EtchTypeKind::TypeRef { name, type_params } => {
                refs.push(name.clone());
                for tp in type_params {
                    tp.collect_referenced_types(refs);
                }
            }
            EtchTypeKind::Array(inner) => inner.collect_referenced_types(refs),
            EtchTypeKind::Tuple(elements) => {
                for e in elements {
                    e.collect_referenced_types(refs);
                }
            }
            EtchTypeKind::Object { key, value } => {
                key.collect_referenced_types(refs);
                value.collect_referenced_types(refs);
            }
            EtchTypeKind::Union(types) | EtchTypeKind::Intersection(types) => {
                for t in types {
                    t.collect_referenced_types(refs);
                }
            }
            EtchTypeKind::Function(func) => {
                for p in &func.params {
                    p.param_type.collect_referenced_types(refs);
                }
                func.return_type.collect_referenced_types(refs);
            }
            EtchTypeKind::Promise(inner) | EtchTypeKind::Set(inner) => {
                inner.collect_referenced_types(refs);
            }
            EtchTypeKind::Map { key, value } => {
                key.collect_referenced_types(refs);
                value.collect_referenced_types(refs);
            }
            EtchTypeKind::Conditional {
                check_type,
                extends_type,
                true_type,
                false_type,
            } => {
                check_type.collect_referenced_types(refs);
                extends_type.collect_referenced_types(refs);
                true_type.collect_referenced_types(refs);
                false_type.collect_referenced_types(refs);
            }
            EtchTypeKind::IndexedAccess {
                obj_type,
                index_type,
            } => {
                obj_type.collect_referenced_types(refs);
                index_type.collect_referenced_types(refs);
            }
            EtchTypeKind::Mapped {
                value_type,
                template,
                constraint,
                ..
            } => {
                value_type.collect_referenced_types(refs);
                if let Some(t) = template {
                    t.collect_referenced_types(refs);
                }
                if let Some(c) = constraint {
                    c.collect_referenced_types(refs);
                }
            }
            EtchTypeKind::TypeOperator { type_arg, .. } => {
                type_arg.collect_referenced_types(refs);
            }
            EtchTypeKind::Parenthesized(inner)
            | EtchTypeKind::Optional(inner)
            | EtchTypeKind::Rest(inner) => {
                inner.collect_referenced_types(refs);
            }
            EtchTypeKind::Constructor {
                params,
                return_type,
                ..
            } => {
                for p in params {
                    if let Some(ty) = &p.ts_type {
                        ty.collect_referenced_types(refs);
                    }
                }
                return_type.collect_referenced_types(refs);
            }
            EtchTypeKind::TypeLiteral {
                properties,
                methods,
            } => {
                for prop in properties {
                    if let Some(ty) = &prop.ts_type {
                        ty.collect_referenced_types(refs);
                    }
                }
                for method in methods {
                    for p in &method.params {
                        if let Some(ty) = &p.ts_type {
                            ty.collect_referenced_types(refs);
                        }
                    }
                    if let Some(ret) = &method.return_type {
                        ret.collect_referenced_types(refs);
                    }
                }
            }
            EtchTypeKind::TypePredicate { ts_type, .. } => {
                if let Some(ty) = ts_type {
                    ty.collect_referenced_types(refs);
                }
            }
            // Leaf nodes that don't contain type references
            EtchTypeKind::Primitive(_)
            | EtchTypeKind::Literal(_)
            | EtchTypeKind::Uint8Array
            | EtchTypeKind::TypeParam(_)
            | EtchTypeKind::Infer(_)
            | EtchTypeKind::This
            | EtchTypeKind::TypeQuery(_)
            | EtchTypeKind::Import { .. }
            | EtchTypeKind::Unknown => {}
        }
    }
}

impl fmt::Display for EtchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_typescript())
    }
}

/// Convert from WeldType to EtchType
impl From<&WeldType> for EtchType {
    fn from(weld: &WeldType) -> Self {
        match weld {
            WeldType::Primitive(p) => EtchType::primitive(EtchPrimitive::from(p)),
            WeldType::Option(inner) => {
                let mut etch = EtchType::from(inner.as_ref());
                etch.nullable = true;
                etch
            }
            WeldType::Vec(inner) => EtchType::array(EtchType::from(inner.as_ref())),
            WeldType::Bytes => EtchType::uint8_array(),
            WeldType::Result { ok, .. } => EtchType::promise(EtchType::from(ok.as_ref())),
            WeldType::HashMap { key, value } | WeldType::BTreeMap { key, value } => {
                EtchType::record(EtchType::from(key.as_ref()), EtchType::from(value.as_ref()))
            }
            WeldType::HashSet(inner) | WeldType::BTreeSet(inner) => {
                EtchType::set(EtchType::from(inner.as_ref()))
            }
            WeldType::Tuple(elements) => {
                EtchType::tuple(elements.iter().map(EtchType::from).collect())
            }
            WeldType::Array { element, .. } => EtchType::array(EtchType::from(element.as_ref())),
            WeldType::Generic { base, params } => {
                EtchType::type_ref(base.clone(), params.iter().map(EtchType::from).collect())
            }
            WeldType::Struct(name) | WeldType::Enum(name) => EtchType::simple_ref(name.clone()),
            WeldType::JsonValue => EtchType::unknown(),
            WeldType::OpState => EtchType::unknown(), // Internal, filtered out
            WeldType::Box(inner)
            | WeldType::Arc(inner)
            | WeldType::Rc(inner)
            | WeldType::RefCell(inner)
            | WeldType::Mutex(inner)
            | WeldType::RwLock(inner)
            | WeldType::Reference { inner, .. }
            | WeldType::Pointer { inner, .. } => EtchType::from(inner.as_ref()),
            WeldType::Never => EtchType::never(),
            WeldType::Unknown => EtchType::unknown(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_to_typescript() {
        assert_eq!(EtchPrimitive::String.to_typescript(), "string");
        assert_eq!(EtchPrimitive::Number.to_typescript(), "number");
        assert_eq!(EtchPrimitive::Boolean.to_typescript(), "boolean");
    }

    #[test]
    fn test_etch_type_to_typescript() {
        // Simple types
        assert_eq!(EtchType::string().to_typescript(), "string");
        assert_eq!(EtchType::number().to_typescript(), "number");

        // Array
        assert_eq!(
            EtchType::array(EtchType::string()).to_typescript(),
            "string[]"
        );

        // Tuple
        assert_eq!(
            EtchType::tuple(vec![EtchType::string(), EtchType::number()]).to_typescript(),
            "[string, number]"
        );

        // Union
        assert_eq!(
            EtchType::union(vec![EtchType::string(), EtchType::number()]).to_typescript(),
            "string | number"
        );

        // Promise
        assert_eq!(
            EtchType::promise(EtchType::string()).to_typescript(),
            "Promise<string>"
        );

        // Record
        assert_eq!(
            EtchType::record(EtchType::string(), EtchType::number()).to_typescript(),
            "Record<string, number>"
        );

        // Nullable
        assert_eq!(
            EtchType::string().as_nullable().to_typescript(),
            "string | null"
        );

        // Optional
        assert_eq!(
            EtchType::string().as_optional().to_typescript(),
            "string | undefined"
        );

        // Both nullable and optional
        assert_eq!(
            EtchType::string()
                .as_nullable()
                .as_optional()
                .to_typescript(),
            "string | null | undefined"
        );
    }

    #[test]
    fn test_weld_type_conversion() {
        // String
        let weld = WeldType::Primitive(WeldPrimitive::String);
        assert_eq!(EtchType::from(&weld).to_typescript(), "string");

        // Vec<string>
        let weld = WeldType::Vec(Box::new(WeldType::Primitive(WeldPrimitive::String)));
        assert_eq!(EtchType::from(&weld).to_typescript(), "string[]");

        // Option<string>
        let weld = WeldType::Option(Box::new(WeldType::Primitive(WeldPrimitive::String)));
        assert_eq!(EtchType::from(&weld).to_typescript(), "string | null");

        // Result<string, Error>
        let weld = WeldType::Result {
            ok: Box::new(WeldType::Primitive(WeldPrimitive::String)),
            err: Box::new(WeldType::Struct("Error".to_string())),
        };
        assert_eq!(EtchType::from(&weld).to_typescript(), "Promise<string>");

        // Vec<u8> -> Uint8Array
        let weld = WeldType::Bytes;
        assert_eq!(EtchType::from(&weld).to_typescript(), "Uint8Array");
    }

    #[test]
    fn test_complex_array_type() {
        // (string | number)[]
        let union = EtchType::union(vec![EtchType::string(), EtchType::number()]);
        let array = EtchType::array(union);
        assert_eq!(array.to_typescript(), "(string | number)[]");
    }
}
