//! Type conversion helpers for docgen
//!
//! This module provides additional type conversion utilities
//! for documentation generation.

use crate::types::{EtchType, EtchTypeKind};
use serde::{Deserialize, Serialize};

/// Rendered type information for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedType {
    /// Plain text representation
    pub text: String,
    /// HTML representation with links
    pub html: String,
    /// Whether this is a primitive type
    pub is_primitive: bool,
    /// Referenced type names (for linking)
    pub references: Vec<String>,
}

impl RenderedType {
    /// Create from an EtchType
    pub fn from_type(ty: &EtchType, link_base: Option<&str>) -> Self {
        let text = ty.to_typescript();
        let references = collect_referenced_types(ty);
        let html = if let Some(base) = link_base {
            render_type_html(ty, base)
        } else {
            html_escape::encode_text(&text).to_string()
        };

        Self {
            text,
            html,
            is_primitive: ty.is_primitive(),
            references,
        }
    }
}

/// Collect all referenced type names from a type
pub fn collect_referenced_types(ty: &EtchType) -> Vec<String> {
    let mut refs = Vec::new();
    collect_refs_recursive(&ty.kind, &mut refs);
    refs.sort();
    refs.dedup();
    refs
}

fn collect_refs_recursive(kind: &EtchTypeKind, refs: &mut Vec<String>) {
    match kind {
        EtchTypeKind::TypeRef { name, type_params } => {
            refs.push(name.clone());
            for p in type_params {
                collect_refs_recursive(&p.kind, refs);
            }
        }
        EtchTypeKind::Array(inner) => collect_refs_recursive(&inner.kind, refs),
        EtchTypeKind::Tuple(types)
        | EtchTypeKind::Union(types)
        | EtchTypeKind::Intersection(types) => {
            for t in types {
                collect_refs_recursive(&t.kind, refs);
            }
        }
        EtchTypeKind::Function(func_def) => {
            for p in &func_def.params {
                collect_refs_recursive(&p.param_type.kind, refs);
            }
            collect_refs_recursive(&func_def.return_type.kind, refs);
        }
        EtchTypeKind::Optional(inner) | EtchTypeKind::Promise(inner) => {
            collect_refs_recursive(&inner.kind, refs);
        }
        _ => {}
    }
}

/// Render a type as HTML with links
fn render_type_html(ty: &EtchType, link_base: &str) -> String {
    match &ty.kind {
        EtchTypeKind::Primitive(p) => {
            format!("<span class=\"type-primitive\">{}</span>", p)
        }
        EtchTypeKind::TypeRef { name, type_params } => {
            let link = if is_builtin_type(name) {
                format!("<span class=\"type-builtin\">{}</span>", name)
            } else {
                format!(
                    "<a href=\"{}#{}\" class=\"type-link\">{}</a>",
                    link_base,
                    name.to_lowercase(),
                    name
                )
            };

            if type_params.is_empty() {
                link
            } else {
                let params: Vec<String> = type_params
                    .iter()
                    .map(|p| render_type_html(p, link_base))
                    .collect();
                format!("{}&lt;{}&gt;", link, params.join(", "))
            }
        }
        EtchTypeKind::Array(inner) => {
            format!(
                "{}<span class=\"type-bracket\">[]</span>",
                render_type_html(inner, link_base)
            )
        }
        EtchTypeKind::Tuple(types) => {
            let inner: Vec<String> = types
                .iter()
                .map(|t| render_type_html(t, link_base))
                .collect();
            format!("[{}]", inner.join(", "))
        }
        EtchTypeKind::Union(types) => {
            let inner: Vec<String> = types
                .iter()
                .map(|t| render_type_html(t, link_base))
                .collect();
            inner.join(" | ")
        }
        EtchTypeKind::Intersection(types) => {
            let inner: Vec<String> = types
                .iter()
                .map(|t| render_type_html(t, link_base))
                .collect();
            inner.join(" &amp; ")
        }
        EtchTypeKind::Function(func_def) => {
            let param_strs: Vec<String> = func_def
                .params
                .iter()
                .map(|p| {
                    let ty_str = render_type_html(&p.param_type, link_base);
                    format!("{}: {}", p.name, ty_str)
                })
                .collect();
            format!(
                "({}) =&gt; {}",
                param_strs.join(", "),
                render_type_html(&func_def.return_type, link_base)
            )
        }
        EtchTypeKind::Literal(lit) => {
            format!(
                "<span class=\"type-literal\">{}</span>",
                lit.to_typescript()
            )
        }
        EtchTypeKind::Optional(inner) => {
            format!("{}?", render_type_html(inner, link_base))
        }
        EtchTypeKind::Promise(inner) => {
            format!(
                "<span class=\"type-builtin\">Promise</span>&lt;{}&gt;",
                render_type_html(inner, link_base)
            )
        }
        _ => html_escape::encode_text(&ty.to_typescript()).to_string(),
    }
}

/// Check if a type name is a built-in TypeScript type
fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "Array"
            | "Promise"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "Record"
            | "Partial"
            | "Required"
            | "Readonly"
            | "Pick"
            | "Omit"
            | "Exclude"
            | "Extract"
            | "NonNullable"
            | "ReturnType"
            | "InstanceType"
            | "Parameters"
            | "ConstructorParameters"
            | "ThisType"
            | "Awaited"
            | "Uint8Array"
            | "Int8Array"
            | "Uint16Array"
            | "Int16Array"
            | "Uint32Array"
            | "Int32Array"
            | "Float32Array"
            | "Float64Array"
            | "BigInt64Array"
            | "BigUint64Array"
            | "ArrayBuffer"
            | "SharedArrayBuffer"
            | "DataView"
            | "Error"
            | "Date"
            | "RegExp"
            | "JSON"
    )
}

/// Type complexity score (for documentation ordering)
pub fn type_complexity(ty: &EtchType) -> usize {
    match &ty.kind {
        EtchTypeKind::Primitive(_) => 1,
        EtchTypeKind::Literal(_) => 1,
        EtchTypeKind::TypeRef { type_params, .. } => {
            1 + type_params.iter().map(type_complexity).sum::<usize>()
        }
        EtchTypeKind::Array(inner) => 1 + type_complexity(inner),
        EtchTypeKind::Tuple(types) => 1 + types.iter().map(type_complexity).sum::<usize>(),
        EtchTypeKind::Union(types) => types.iter().map(type_complexity).sum::<usize>(),
        EtchTypeKind::Intersection(types) => types.iter().map(type_complexity).sum::<usize>(),
        EtchTypeKind::Function(func_def) => {
            2 + func_def.params.len() + type_complexity(&func_def.return_type)
        }
        EtchTypeKind::Conditional { .. } => 5,
        EtchTypeKind::Mapped { .. } => 4,
        EtchTypeKind::TypeLiteral {
            properties,
            methods,
        } => 3 + properties.len() + methods.len(),
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendered_type() {
        let ty = EtchType::string();
        let rendered = RenderedType::from_type(&ty, None);
        assert_eq!(rendered.text, "string");
        assert!(rendered.is_primitive);
    }

    #[test]
    fn test_type_complexity() {
        let simple = EtchType::string();
        let complex = EtchType::new(EtchTypeKind::Union(vec![
            EtchType::string(),
            EtchType::number(),
            EtchType::boolean(),
        ]));

        assert!(type_complexity(&simple) < type_complexity(&complex));
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin_type("Promise"));
        assert!(is_builtin_type("Array"));
        assert!(!is_builtin_type("MyCustomType"));
    }
}
