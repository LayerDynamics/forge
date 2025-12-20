//! TypeScript type utilities
//!
//! This module provides utilities for working with TypeScript types,
//! including type formatting, type guards, and common type patterns.

use crate::types::{EtchType, EtchTypeKind};
use serde::{Deserialize, Serialize};

// Re-export type literal types for use in parser
pub use crate::types::{
    TypeLiteralMethod as TsTypeLiteralMethod, TypeLiteralProperty as TsTypeLiteralProperty,
};

/// TypeScript type literal kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TsTypeLiteralKind {
    /// Number literal (e.g., 42)
    Number,
    /// String literal (e.g., "hello")
    String,
    /// Boolean literal (true/false)
    Boolean,
    /// BigInt literal (e.g., 42n)
    BigInt,
    /// Template literal
    Template,
}

/// Result of analyzing a type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeAnalysis {
    /// Whether the type is nullable (includes null)
    pub is_nullable: bool,

    /// Whether the type is optional (includes undefined)
    pub is_optional: bool,

    /// Whether the type is a Promise
    pub is_async: bool,

    /// Whether the type is an array
    pub is_array: bool,

    /// Whether the type is a tuple
    pub is_tuple: bool,

    /// Whether the type is a function
    pub is_function: bool,

    /// Whether the type is a union
    pub is_union: bool,

    /// Whether the type is an intersection
    pub is_intersection: bool,

    /// Whether the type is generic
    pub is_generic: bool,

    /// Whether the type is a primitive
    pub is_primitive: bool,

    /// Type complexity score (for rendering decisions)
    pub complexity: usize,
}

impl TypeAnalysis {
    /// Analyze a type
    pub fn from_type(ty: &EtchType) -> Self {
        let mut analysis = Self {
            is_nullable: ty.nullable,
            is_optional: ty.optional,
            is_async: false,
            is_array: false,
            is_tuple: false,
            is_function: false,
            is_union: false,
            is_intersection: false,
            is_generic: false,
            is_primitive: false,
            complexity: 1,
        };

        analysis.analyze_kind(&ty.kind);
        analysis
    }

    fn analyze_kind(&mut self, kind: &EtchTypeKind) {
        match kind {
            EtchTypeKind::Primitive(_) => {
                self.is_primitive = true;
            }
            EtchTypeKind::Array(inner) => {
                self.is_array = true;
                self.complexity += Self::from_type(inner).complexity;
            }
            EtchTypeKind::Tuple(elements) => {
                self.is_tuple = true;
                self.complexity += elements.len();
                for elem in elements {
                    self.complexity += Self::from_type(elem).complexity;
                }
            }
            EtchTypeKind::Union(types) => {
                self.is_union = true;
                self.complexity += types.len();
                // Check for nullable/optional in union
                for t in types {
                    if matches!(
                        t.kind,
                        EtchTypeKind::Primitive(crate::types::EtchPrimitive::Null)
                    ) {
                        self.is_nullable = true;
                    }
                    if matches!(
                        t.kind,
                        EtchTypeKind::Primitive(crate::types::EtchPrimitive::Undefined)
                    ) {
                        self.is_optional = true;
                    }
                }
            }
            EtchTypeKind::Intersection(types) => {
                self.is_intersection = true;
                self.complexity += types.len();
            }
            EtchTypeKind::Function(_) => {
                self.is_function = true;
                self.complexity += 3;
            }
            EtchTypeKind::Promise(_) => {
                self.is_async = true;
                self.complexity += 1;
            }
            EtchTypeKind::TypeRef { type_params, .. } => {
                if !type_params.is_empty() {
                    self.is_generic = true;
                    self.complexity += type_params.len();
                }
            }
            EtchTypeKind::Conditional { .. } => {
                self.complexity += 5;
            }
            EtchTypeKind::Mapped { .. } => {
                self.complexity += 4;
            }
            _ => {}
        }
    }

    /// Check if this is a simple type (low complexity)
    pub fn is_simple(&self) -> bool {
        self.complexity <= 2
    }

    /// Check if this needs parentheses in certain contexts
    pub fn needs_parens_in_array(&self) -> bool {
        self.is_union || self.is_intersection || self.is_function
    }
}

/// Common TypeScript type patterns
pub mod patterns {
    use super::*;

    /// Create a nullable type (T | null)
    pub fn nullable(ty: EtchType) -> EtchType {
        ty.as_nullable()
    }

    /// Create an optional type (T | undefined)
    pub fn optional(ty: EtchType) -> EtchType {
        ty.as_optional()
    }

    /// Create a maybe type (T | null | undefined)
    pub fn maybe(ty: EtchType) -> EtchType {
        ty.as_nullable().as_optional()
    }

    /// Create a readonly array type (readonly T[])
    pub fn readonly_array(element: EtchType) -> EtchType {
        EtchType::new(EtchTypeKind::TypeOperator {
            operator: crate::types::TypeOperator::Readonly,
            type_arg: Box::new(EtchType::array(element)),
        })
    }

    /// Create a Partial<T> type
    pub fn partial(ty: EtchType) -> EtchType {
        EtchType::type_ref("Partial", vec![ty])
    }

    /// Create a Required<T> type
    pub fn required(ty: EtchType) -> EtchType {
        EtchType::type_ref("Required", vec![ty])
    }

    /// Create a Readonly<T> type
    pub fn readonly(ty: EtchType) -> EtchType {
        EtchType::type_ref("Readonly", vec![ty])
    }

    /// Create a Pick<T, K> type
    pub fn pick(ty: EtchType, keys: EtchType) -> EtchType {
        EtchType::type_ref("Pick", vec![ty, keys])
    }

    /// Create an Omit<T, K> type
    pub fn omit(ty: EtchType, keys: EtchType) -> EtchType {
        EtchType::type_ref("Omit", vec![ty, keys])
    }

    /// Create an Extract<T, U> type
    pub fn extract(ty: EtchType, union: EtchType) -> EtchType {
        EtchType::type_ref("Extract", vec![ty, union])
    }

    /// Create an Exclude<T, U> type
    pub fn exclude(ty: EtchType, union: EtchType) -> EtchType {
        EtchType::type_ref("Exclude", vec![ty, union])
    }

    /// Create a NonNullable<T> type
    pub fn non_nullable(ty: EtchType) -> EtchType {
        EtchType::type_ref("NonNullable", vec![ty])
    }

    /// Create a ReturnType<T> type
    pub fn return_type(fn_ty: EtchType) -> EtchType {
        EtchType::type_ref("ReturnType", vec![fn_ty])
    }

    /// Create a Parameters<T> type
    pub fn parameters(fn_ty: EtchType) -> EtchType {
        EtchType::type_ref("Parameters", vec![fn_ty])
    }

    /// Create an Awaited<T> type (TypeScript 4.5+)
    pub fn awaited(ty: EtchType) -> EtchType {
        EtchType::type_ref("Awaited", vec![ty])
    }
}

/// Type simplification utilities
pub mod simplify {
    use super::*;

    /// Flatten nested unions
    pub fn flatten_union(types: Vec<EtchType>) -> Vec<EtchType> {
        let mut result = vec![];
        for ty in types {
            if let EtchTypeKind::Union(inner) = &ty.kind {
                result.extend(flatten_union(inner.clone()));
            } else {
                result.push(ty);
            }
        }
        result
    }

    /// Flatten nested intersections
    pub fn flatten_intersection(types: Vec<EtchType>) -> Vec<EtchType> {
        let mut result = vec![];
        for ty in types {
            if let EtchTypeKind::Intersection(inner) = &ty.kind {
                result.extend(flatten_intersection(inner.clone()));
            } else {
                result.push(ty);
            }
        }
        result
    }

    /// Remove duplicate types from a union
    pub fn dedupe_union(types: Vec<EtchType>) -> Vec<EtchType> {
        let mut seen = std::collections::HashSet::new();
        let mut result = vec![];
        for ty in types {
            let key = ty.to_typescript();
            if seen.insert(key) {
                result.push(ty);
            }
        }
        result
    }

    /// Extract null/undefined from a union and return (base_type, is_nullable, is_optional)
    pub fn extract_optionality(ty: &EtchType) -> (Option<EtchType>, bool, bool) {
        if let EtchTypeKind::Union(types) = &ty.kind {
            let mut is_nullable = ty.nullable;
            let mut is_optional = ty.optional;
            let mut other_types = vec![];

            for t in types {
                match &t.kind {
                    EtchTypeKind::Primitive(crate::types::EtchPrimitive::Null) => {
                        is_nullable = true;
                    }
                    EtchTypeKind::Primitive(crate::types::EtchPrimitive::Undefined) => {
                        is_optional = true;
                    }
                    _ => {
                        other_types.push(t.clone());
                    }
                }
            }

            let base = match other_types.len() {
                0 => None,
                1 => Some(other_types.into_iter().next().unwrap()),
                _ => Some(EtchType::union(other_types)),
            };

            (base, is_nullable, is_optional)
        } else {
            (Some(ty.clone()), ty.nullable, ty.optional)
        }
    }
}

/// Type display options
#[derive(Debug, Clone, Default)]
pub struct TypeDisplayOptions {
    /// Maximum depth for nested types
    pub max_depth: Option<usize>,

    /// Whether to use short form (e.g., `string[]` vs `Array<string>`)
    pub prefer_short_form: bool,

    /// Whether to include documentation
    pub include_doc: bool,

    /// Whether to use single quotes for string literals
    pub single_quotes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EtchPrimitive;

    #[test]
    fn test_type_analysis() {
        let simple = EtchType::string();
        let analysis = TypeAnalysis::from_type(&simple);
        assert!(analysis.is_primitive);
        assert!(analysis.is_simple());

        let complex = EtchType::union(vec![
            EtchType::string(),
            EtchType::number(),
            EtchType::new(EtchTypeKind::Primitive(EtchPrimitive::Null)),
        ]);
        let analysis = TypeAnalysis::from_type(&complex);
        assert!(analysis.is_union);
        assert!(analysis.is_nullable);
    }

    #[test]
    fn test_patterns() {
        let ty = patterns::partial(EtchType::simple_ref("User"));
        assert_eq!(ty.to_typescript(), "Partial<User>");

        let ty = patterns::readonly_array(EtchType::string());
        assert_eq!(ty.to_typescript(), "readonly string[]");
    }

    #[test]
    fn test_flatten_union() {
        let nested = vec![
            EtchType::string(),
            EtchType::union(vec![EtchType::number(), EtchType::boolean()]),
        ];
        let flattened = simplify::flatten_union(nested);
        assert_eq!(flattened.len(), 3);
    }

    #[test]
    fn test_extract_optionality() {
        let ty = EtchType::union(vec![
            EtchType::string(),
            EtchType::new(EtchTypeKind::Primitive(EtchPrimitive::Null)),
            EtchType::new(EtchTypeKind::Primitive(EtchPrimitive::Undefined)),
        ]);

        let (base, nullable, optional) = simplify::extract_optionality(&ty);
        assert!(nullable);
        assert!(optional);
        assert!(base.is_some());
        assert_eq!(base.unwrap().to_typescript(), "string");
    }
}
