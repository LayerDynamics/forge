//! Runtime Rust source parsing into [`EtchNode`]s.
//!
//! Most Rust API metadata in forge-etch is collected at *build time* from the
//! forge-weld inventory (see [`crate::docgen::rust`]). This module covers the
//! complementary *runtime* case: parsing a raw Rust source string (or file)
//! with `syn` and extracting its top-level public surface — functions, structs,
//! and enums — into the same [`EtchNode`] model the rest of forge-etch uses, so
//! the result can be merged with TypeScript nodes via
//! [`crate::parser::merge_nodes`].

use crate::diagnostics::{EtchError, EtchResult};
use crate::function::FunctionDef;
use crate::js_doc::EtchDoc;
use crate::node::{EtchNode, EtchNodeDef, Location, StructDef, StructFieldDef};
use crate::params::ParamDef;
use crate::r#enum::{EnumDef, EnumMemberDef};
use crate::types::EtchType;
use crate::visibility::Visibility;

/// Parse a Rust source string into [`EtchNode`]s.
///
/// Extracts top-level `fn`, `struct`, and `enum` items (public or not — the
/// `visibility` field records which). Returns [`EtchError::RustParse`] if the
/// source is not valid Rust.
pub fn parse_rust_source(source: &str) -> EtchResult<Vec<EtchNode>> {
    let file = syn::parse_file(source).map_err(|e| EtchError::RustParse(e.to_string()))?;
    Ok(file.items.iter().filter_map(item_to_node).collect())
}

/// Read a Rust source file and parse it via [`parse_rust_source`].
pub fn parse_rust_file(path: impl AsRef<std::path::Path>) -> EtchResult<Vec<EtchNode>> {
    let source = std::fs::read_to_string(path)?;
    parse_rust_source(&source)
}

/// Map a single top-level item to an [`EtchNode`], or `None` for kinds we do
/// not document at runtime (impls, traits, uses, etc.).
fn item_to_node(item: &syn::Item) -> Option<EtchNode> {
    match item {
        syn::Item::Fn(f) => Some(function_node(f)),
        syn::Item::Struct(s) => Some(struct_node(s)),
        syn::Item::Enum(e) => Some(enum_node(e)),
        _ => None,
    }
}

fn function_node(f: &syn::ItemFn) -> EtchNode {
    let params = f.sig.inputs.iter().filter_map(fn_param).collect();
    let return_type = match &f.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(EtchType::type_ref(type_to_string(ty), vec![])),
    };

    let function_def = FunctionDef {
        def_name: None,
        params,
        return_type,
        type_params: vec![],
        is_async: f.sig.asyncness.is_some(),
        is_generator: false,
        has_body: true,
        decorators: vec![],
        overloads: vec![],
    };

    EtchNode {
        name: f.sig.ident.to_string(),
        is_default: None,
        location: Location::default(),
        visibility: visibility_of(&f.vis),
        doc: extract_doc(&f.attrs),
        def: EtchNodeDef::Function { function_def },
        module: None,
    }
}

fn fn_param(arg: &syn::FnArg) -> Option<ParamDef> {
    match arg {
        // `self` receivers are not documented parameters.
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat) => {
            let name = match &*pat.pat {
                syn::Pat::Ident(id) => id.ident.to_string(),
                _ => "_".to_string(),
            };
            Some(ParamDef {
                name,
                ts_type: Some(EtchType::type_ref(type_to_string(&pat.ty), vec![])),
                optional: is_option_type(&pat.ty),
                default: None,
                doc: None,
                rest: false,
                decorators: vec![],
            })
        }
    }
}

fn struct_node(s: &syn::ItemStruct) -> EtchNode {
    let fields = s
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = field
                .ident
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| i.to_string());
            StructFieldDef {
                ts_name: to_camel_case(&name),
                ts_type: type_to_string(&field.ty),
                optional: is_option_type(&field.ty),
                readonly: false,
                doc: extract_doc(&field.attrs).description,
                name,
            }
        })
        .collect();

    let struct_def = StructDef {
        rust_name: s.ident.to_string(),
        ts_name: s.ident.to_string(),
        fields,
        type_params: generic_param_names(&s.generics),
    };

    EtchNode {
        name: s.ident.to_string(),
        is_default: None,
        location: Location::default(),
        visibility: visibility_of(&s.vis),
        doc: extract_doc(&s.attrs),
        def: EtchNodeDef::Struct { struct_def },
        module: None,
    }
}

fn enum_node(e: &syn::ItemEnum) -> EtchNode {
    let members = e
        .variants
        .iter()
        .map(|v| EnumMemberDef {
            name: v.ident.to_string(),
            init: None,
            doc: extract_doc(&v.attrs).description,
        })
        .collect();

    let enum_def = EnumDef {
        members,
        is_const: false,
        is_declare: false,
    };

    EtchNode {
        name: e.ident.to_string(),
        is_default: None,
        location: Location::default(),
        visibility: visibility_of(&e.vis),
        doc: extract_doc(&e.attrs),
        def: EtchNodeDef::Enum { enum_def },
        module: None,
    }
}

/// `pub` maps to [`Visibility::Public`]; anything else is [`Visibility::Private`].
fn visibility_of(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        _ => Visibility::Private,
    }
}

/// Is the type an `Option<...>` (so the field/param is "optional")?
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(path) = ty {
        if let Some(seg) = path.path.segments.last() {
            return seg.ident == "Option";
        }
    }
    false
}

/// Names of the type generics (e.g. `T`, `K`) declared on an item.
fn generic_param_names(generics: &syn::Generics) -> Vec<String> {
    generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect()
}

/// Render a `syn::Type` to a tidy Rust type string (e.g. `Vec<String>`).
fn type_to_string(ty: &syn::Type) -> String {
    normalize_type(&quote::quote!(#ty).to_string())
}

/// `quote` stringifies tokens with spaces around punctuation
/// (`Vec < String >`); collapse the spaces that matter for type readability.
fn normalize_type(s: &str) -> String {
    let mut out = s.to_string();
    for (from, to) in [
        (" <", "<"),
        ("< ", "<"),
        (" >", ">"),
        (" ::", "::"),
        (":: ", "::"),
        (" ,", ","),
        ("& ", "&"),
    ] {
        out = out.replace(from, to);
    }
    out
}

/// Collect `///` / `#[doc = "..."]` lines into an [`EtchDoc`] description.
fn extract_doc(attrs: &[syn::Attribute]) -> EtchDoc {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
            {
                let raw = s.value();
                // `///` doc comments carry a leading space; drop just that one.
                lines.push(raw.strip_prefix(' ').unwrap_or(&raw).to_string());
            }
        }
    }

    let description = if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n").trim().to_string())
    };

    EtchDoc {
        description,
        tags: vec![],
    }
}

/// Convert `snake_case` to `camelCase` for the TypeScript-facing field name.
fn to_camel_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper_next = false;
    for ch in s.chars() {
        if ch == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // M3 regression: runtime Rust parsing used to be unimplemented (the op
    // returned an error and merge_nodes discarded rust_source). It must now
    // produce real EtchNodes.

    #[test]
    fn parses_pub_fn_with_doc_params_and_return() {
        let src = "/// Greets a name.\npub fn greet(name: String, times: u32) -> String { name }";
        let nodes = parse_rust_source(src).unwrap();
        assert_eq!(nodes.len(), 1);
        let node = &nodes[0];
        assert_eq!(node.name, "greet");
        assert!(matches!(node.visibility, Visibility::Public));
        assert_eq!(node.doc.description.as_deref(), Some("Greets a name."));

        let EtchNodeDef::Function { function_def } = &node.def else {
            panic!("expected a function node");
        };
        assert_eq!(function_def.params.len(), 2);
        assert_eq!(function_def.params[0].name, "name");
        assert_eq!(function_def.params[1].name, "times");
        assert!(function_def.return_type.is_some());
        assert!(!function_def.is_async);
    }

    #[test]
    fn parses_struct_fields_and_optional() {
        let src = "pub struct Config { pub host: String, pub port: Option<u16> }";
        let nodes = parse_rust_source(src).unwrap();
        let node = nodes.iter().find(|n| n.name == "Config").unwrap();
        let EtchNodeDef::Struct { struct_def } = &node.def else {
            panic!("expected a struct node");
        };
        assert_eq!(struct_def.fields.len(), 2);
        assert_eq!(struct_def.fields[0].name, "host");
        assert_eq!(struct_def.fields[0].ts_type, "String");
        assert!(!struct_def.fields[0].optional);
        assert_eq!(struct_def.fields[1].name, "port");
        assert!(struct_def.fields[1].optional);
    }

    #[test]
    fn parses_enum_variants() {
        let nodes = parse_rust_source("pub enum Mode { Fast, Slow }").unwrap();
        let EtchNodeDef::Enum { enum_def } = &nodes[0].def else {
            panic!("expected an enum node");
        };
        assert_eq!(enum_def.members.len(), 2);
        assert_eq!(enum_def.members[0].name, "Fast");
        assert_eq!(enum_def.members[1].name, "Slow");
    }

    #[test]
    fn records_async_and_private_visibility() {
        let nodes = parse_rust_source("async fn worker() {}").unwrap();
        assert!(matches!(nodes[0].visibility, Visibility::Private));
        let EtchNodeDef::Function { function_def } = &nodes[0].def else {
            panic!("expected a function node");
        };
        assert!(function_def.is_async);
    }

    #[test]
    fn renders_generic_types_tidily() {
        let nodes = parse_rust_source("pub fn ids() -> Vec<Option<u64>> { vec![] }").unwrap();
        let EtchNodeDef::Function { function_def } = &nodes[0].def else {
            panic!("expected a function node");
        };
        let ret = function_def.return_type.as_ref().unwrap().to_typescript();
        // The Rust type string is collapsed, not "Vec < Option < u64 > >".
        assert!(ret.contains("Vec<Option<u64>>"), "got: {ret}");
    }

    #[test]
    fn invalid_source_is_a_typed_error() {
        let err = parse_rust_source("pub fn (").unwrap_err();
        assert!(matches!(err, EtchError::RustParse(_)));
    }

    #[test]
    fn parse_rust_file_reads_then_parses() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "pub fn from_file() {{}}").unwrap();
        let nodes = parse_rust_file(file.path()).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].name, "from_file");
    }

    #[test]
    fn merge_combines_rust_and_ts_nodes() {
        let rust_nodes = parse_rust_source("pub fn only_rust() {}").unwrap();
        let ts_node = EtchNode {
            name: "only_ts".to_string(),
            ..Default::default()
        };
        let merged = crate::parser::merge_nodes(vec![ts_node], rust_nodes);
        assert!(merged.iter().any(|n| n.name == "only_rust"));
        assert!(merged.iter().any(|n| n.name == "only_ts"));
    }
}
