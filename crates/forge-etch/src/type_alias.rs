//! Type alias definitions
//!
//! This module provides types for representing TypeScript type aliases
//! in documentation.

use crate::ts_type_params::TsTypeParamDef;
use crate::types::EtchType;
use serde::{Deserialize, Serialize};

/// Type alias definition
///
/// Represents a TypeScript type alias like:
/// ```typescript
/// type StringOrNumber = string | number;
/// type Callback<T> = (value: T) => void;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeAliasDef {
    /// The type that this alias refers to
    pub ts_type: EtchType,

    /// Type parameters (generics)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<TsTypeParamDef>,
}

impl TypeAliasDef {
    /// Create a new type alias definition
    pub fn new(ts_type: EtchType) -> Self {
        Self {
            ts_type,
            type_params: vec![],
        }
    }

    /// Add type parameters
    pub fn with_type_params(mut self, params: Vec<TsTypeParamDef>) -> Self {
        self.type_params = params;
        self
    }

    /// Add a single type parameter
    pub fn with_type_param(mut self, param: TsTypeParamDef) -> Self {
        self.type_params.push(param);
        self
    }

    /// Generate TypeScript declaration (without the name)
    pub fn to_typescript_rhs(&self) -> String {
        self.ts_type.to_typescript()
    }

    /// Generate full TypeScript declaration
    pub fn to_typescript(&self, name: &str) -> String {
        let type_params = if self.type_params.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = self.type_params.iter().map(|p| p.to_typescript()).collect();
            format!("<{}>", params.join(", "))
        };

        format!(
            "type {}{} = {}",
            name,
            type_params,
            self.ts_type.to_typescript()
        )
    }

    /// Check if this is a simple alias (no type params)
    pub fn is_simple(&self) -> bool {
        self.type_params.is_empty()
    }

    /// Check if the underlying type is a union
    pub fn is_union(&self) -> bool {
        matches!(self.ts_type.kind, crate::types::EtchTypeKind::Union(_))
    }

    /// Check if the underlying type is an intersection
    pub fn is_intersection(&self) -> bool {
        matches!(
            self.ts_type.kind,
            crate::types::EtchTypeKind::Intersection(_)
        )
    }

    /// Check if the underlying type is a function
    pub fn is_function(&self) -> bool {
        matches!(self.ts_type.kind, crate::types::EtchTypeKind::Function(_))
    }
}

impl Default for TypeAliasDef {
    fn default() -> Self {
        Self {
            ts_type: EtchType::unknown(),
            type_params: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_type_alias() {
        let alias = TypeAliasDef::new(EtchType::union(vec![
            EtchType::string(),
            EtchType::number(),
        ]));

        assert_eq!(
            alias.to_typescript("StringOrNumber"),
            "type StringOrNumber = string | number"
        );
    }

    #[test]
    fn test_generic_type_alias() {
        let alias = TypeAliasDef::new(EtchType::array(EtchType::new(
            crate::types::EtchTypeKind::TypeParam("T".to_string()),
        )))
        .with_type_param(TsTypeParamDef::new("T"));

        assert_eq!(alias.to_typescript("List"), "type List<T> = T[]");
    }

    #[test]
    fn test_type_alias_with_constraint() {
        let alias = TypeAliasDef::new(EtchType::type_ref(
            "Record",
            vec![
                EtchType::new(crate::types::EtchTypeKind::TypeParam("K".to_string())),
                EtchType::new(crate::types::EtchTypeKind::TypeParam("V".to_string())),
            ],
        ))
        .with_type_params(vec![
            TsTypeParamDef::new("K").with_constraint(EtchType::string()),
            TsTypeParamDef::new("V"),
        ]);

        let ts = alias.to_typescript("StringKeyedRecord");
        assert!(ts.contains("K extends string"));
    }

    #[test]
    fn test_is_helpers() {
        let union_alias = TypeAliasDef::new(EtchType::union(vec![
            EtchType::string(),
            EtchType::number(),
        ]));
        assert!(union_alias.is_union());
        assert!(!union_alias.is_intersection());

        let simple_alias = TypeAliasDef::new(EtchType::string());
        assert!(simple_alias.is_simple());
    }
}
