//! Main parsing coordinator for forge-etch
//!
//! This module provides the high-level parsing functions that coordinate:
//! - TypeScript file parsing via deno_ast/SWC
//! - forge-weld IR extraction
//! - Merging documentation from multiple sources

use crate::diagnostics::EtchResult;
use crate::js_doc::EtchDoc;
use crate::node::{EtchNode, Location};
use crate::utils::swc::{parse_typescript_file, parse_typescript_source, ParsedModule};
use crate::visibility::Visibility;
use deno_ast::swc::ast as swc_ast;
use forge_weld::ir::{OpSymbol, WeldEnum, WeldModule, WeldStruct};
use indexmap::IndexMap;
use std::path::Path;

/// Parse a TypeScript file and extract documentation nodes
///
/// This function parses a TypeScript file and extracts all documented
/// symbols (functions, classes, interfaces, etc.) as EtchNode structures.
pub fn parse_typescript(path: impl AsRef<Path>) -> EtchResult<Vec<EtchNode>> {
    let parsed = parse_typescript_file(path)?;
    extract_nodes_from_module(&parsed)
}

/// Parse TypeScript source code and extract documentation nodes
pub fn parse_typescript_str(path: impl AsRef<Path>, source: &str) -> EtchResult<Vec<EtchNode>> {
    let parsed = parse_typescript_source(path, source)?;
    extract_nodes_from_module(&parsed)
}

/// Extract documentation nodes from a parsed TypeScript module
fn extract_nodes_from_module(parsed: &ParsedModule) -> EtchResult<Vec<EtchNode>> {
    let mut nodes = Vec::new();
    let module = parsed.module();

    for item in &module.body {
        if let Some(node) = extract_node_from_item(parsed, item) {
            nodes.push(node);
        }
    }

    Ok(nodes)
}

/// Extract a documentation node from a module item
fn extract_node_from_item(parsed: &ParsedModule, item: &swc_ast::ModuleItem) -> Option<EtchNode> {
    match item {
        swc_ast::ModuleItem::ModuleDecl(decl) => extract_node_from_module_decl(parsed, decl),
        swc_ast::ModuleItem::Stmt(stmt) => {
            // Non-exported statements - check for declarations
            if let swc_ast::Stmt::Decl(decl) = stmt {
                let mut node = extract_node_from_decl(parsed, decl)?;
                node.visibility = Visibility::Private;
                Some(node)
            } else {
                None
            }
        }
    }
}

/// Extract a node from a module declaration (exports, imports)
fn extract_node_from_module_decl(
    parsed: &ParsedModule,
    decl: &swc_ast::ModuleDecl,
) -> Option<EtchNode> {
    use deno_ast::swc::common::Spanned;

    match decl {
        swc_ast::ModuleDecl::ExportDecl(export) => {
            let mut node = extract_node_from_decl(parsed, &export.decl)?;
            node.visibility = Visibility::Public;
            // JSDoc comments are attached to the export span, not the inner declaration
            // If the inner declaration didn't have a doc, try extracting from export span
            if node.doc.is_empty() {
                node.doc = extract_jsdoc(parsed, export.span);
            }
            Some(node)
        }
        swc_ast::ModuleDecl::ExportDefaultDecl(export) => {
            let mut node = extract_node_from_default_decl(parsed, &export.decl)?;
            node.visibility = Visibility::Public;
            node.is_default = Some(true);
            // JSDoc comments are attached to the export span, not the inner declaration
            if node.doc.is_empty() {
                node.doc = extract_jsdoc(parsed, export.span);
            }
            Some(node)
        }
        swc_ast::ModuleDecl::ExportDefaultExpr(export) => {
            // Default exported expression (e.g., `export default someValue`)
            let location = parsed.span_to_location(export.span);
            let doc = extract_jsdoc(parsed, export.span);

            Some(EtchNode {
                name: "default".to_string(),
                is_default: Some(true),
                location,
                visibility: Visibility::Public,
                doc,
                def: crate::node::EtchNodeDef::Variable {
                    variable_def: crate::variable::VariableDef {
                        kind: crate::variable::VariableKind::Const,
                        ts_type: None,
                        value: Some(parsed.text_for_span(export.expr.span()).to_string()),
                    },
                },
                module: None,
            })
        }
        _ => None, // Import declarations, re-exports, etc.
    }
}

/// Extract a node from a declaration
fn extract_node_from_decl(parsed: &ParsedModule, decl: &swc_ast::Decl) -> Option<EtchNode> {
    match decl {
        swc_ast::Decl::Fn(fn_decl) => Some(extract_function_node(parsed, fn_decl)),
        swc_ast::Decl::Class(class_decl) => Some(extract_class_node(parsed, class_decl)),
        swc_ast::Decl::Var(var_decl) => extract_variable_node(parsed, var_decl),
        swc_ast::Decl::TsInterface(iface) => Some(extract_interface_node(parsed, iface)),
        swc_ast::Decl::TsTypeAlias(alias) => Some(extract_type_alias_node(parsed, alias)),
        swc_ast::Decl::TsEnum(ts_enum) => Some(extract_enum_node(parsed, ts_enum)),
        swc_ast::Decl::TsModule(module) => Some(extract_namespace_node(parsed, module)),
        swc_ast::Decl::Using(_) => None,
    }
}

/// Extract a node from a default declaration
fn extract_node_from_default_decl(
    parsed: &ParsedModule,
    decl: &swc_ast::DefaultDecl,
) -> Option<EtchNode> {
    match decl {
        swc_ast::DefaultDecl::Fn(fn_expr) => Some(extract_function_expr_node(parsed, fn_expr)),
        swc_ast::DefaultDecl::Class(class_expr) => {
            Some(extract_class_expr_node(parsed, class_expr))
        }
        swc_ast::DefaultDecl::TsInterfaceDecl(iface) => Some(extract_interface_node(parsed, iface)),
    }
}

/// Extract a function declaration node
fn extract_function_node(parsed: &ParsedModule, fn_decl: &swc_ast::FnDecl) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = fn_decl.ident.sym.to_string();
    let location = parsed.span_to_location(fn_decl.span());
    let doc = extract_jsdoc(parsed, fn_decl.span());

    let function_def = crate::function::FunctionDef {
        def_name: None,
        params: extract_params(&fn_decl.function.params),
        return_type: fn_decl
            .function
            .return_type
            .as_ref()
            .map(|t| swc_type_to_etch_type(&t.type_ann)),
        type_params: extract_type_params(fn_decl.function.type_params.as_deref()),
        is_async: fn_decl.function.is_async,
        is_generator: fn_decl.function.is_generator,
        has_body: fn_decl.function.body.is_some(),
        decorators: extract_decorators(&fn_decl.function.decorators),
        overloads: vec![],
    };

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Function { function_def },
        module: None,
    }
}

/// Extract a function expression node (for default exports)
fn extract_function_expr_node(parsed: &ParsedModule, fn_expr: &swc_ast::FnExpr) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = fn_expr
        .ident
        .as_ref()
        .map(|i| i.sym.to_string())
        .unwrap_or_else(|| "default".to_string());
    let location = parsed.span_to_location(fn_expr.span());
    let doc = extract_jsdoc(parsed, fn_expr.span());

    let function_def = crate::function::FunctionDef {
        def_name: fn_expr.ident.as_ref().map(|i| i.sym.to_string()),
        params: extract_params(&fn_expr.function.params),
        return_type: fn_expr
            .function
            .return_type
            .as_ref()
            .map(|t| swc_type_to_etch_type(&t.type_ann)),
        type_params: extract_type_params(fn_expr.function.type_params.as_deref()),
        is_async: fn_expr.function.is_async,
        is_generator: fn_expr.function.is_generator,
        has_body: fn_expr.function.body.is_some(),
        decorators: extract_decorators(&fn_expr.function.decorators),
        overloads: vec![],
    };

    EtchNode {
        name,
        is_default: Some(true),
        location,
        visibility: Visibility::Public,
        doc,
        def: crate::node::EtchNodeDef::Function { function_def },
        module: None,
    }
}

/// Extract a class declaration node
fn extract_class_node(parsed: &ParsedModule, class_decl: &swc_ast::ClassDecl) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = class_decl.ident.sym.to_string();
    let location = parsed.span_to_location(class_decl.span());
    let doc = extract_jsdoc(parsed, class_decl.span());

    let class_def = extract_class_def(&class_decl.class, parsed);

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Class { class_def },
        module: None,
    }
}

/// Extract a class expression node
fn extract_class_expr_node(parsed: &ParsedModule, class_expr: &swc_ast::ClassExpr) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = class_expr
        .ident
        .as_ref()
        .map(|i| i.sym.to_string())
        .unwrap_or_else(|| "default".to_string());
    let location = parsed.span_to_location(class_expr.span());
    let doc = extract_jsdoc(parsed, class_expr.span());

    let class_def = extract_class_def(&class_expr.class, parsed);

    EtchNode {
        name,
        is_default: Some(true),
        location,
        visibility: Visibility::Public,
        doc,
        def: crate::node::EtchNodeDef::Class { class_def },
        module: None,
    }
}

/// Extract class definition from SWC Class
fn extract_class_def(class: &swc_ast::Class, parsed: &ParsedModule) -> crate::class::ClassDef {
    use crate::class::{
        ClassConstructorDef, ClassDef, ClassMethodDef, ClassPropertyDef,
        MethodKind as ClassMethodKind,
    };

    let mut constructors = Vec::new();
    let mut properties = Vec::new();
    let mut methods = Vec::new();
    let mut index_signatures = Vec::new();

    for member in &class.body {
        match member {
            swc_ast::ClassMember::Constructor(ctor) => {
                let ctor_def = ClassConstructorDef {
                    params: extract_params_from_ctor(&ctor.params),
                    doc: Some(extract_jsdoc(parsed, ctor.span)),
                    accessibility: ctor.accessibility.map(accessibility_to_string),
                    has_body: ctor.body.is_some(),
                };
                constructors.push(ctor_def);
            }
            swc_ast::ClassMember::Method(method) => {
                let name = prop_name_to_string(&method.key);
                let method_def = ClassMethodDef {
                    name,
                    is_static: method.is_static,
                    accessibility: method.accessibility.map(accessibility_to_string),
                    is_abstract: method.is_abstract,
                    is_optional: method.is_optional,
                    is_override: method.is_override,
                    kind: match method.kind {
                        swc_ast::MethodKind::Method => ClassMethodKind::Method,
                        swc_ast::MethodKind::Getter => ClassMethodKind::Getter,
                        swc_ast::MethodKind::Setter => ClassMethodKind::Setter,
                    },
                    params: extract_params(&method.function.params),
                    return_type: method
                        .function
                        .return_type
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    type_params: extract_type_params(method.function.type_params.as_deref()),
                    doc: Some(extract_jsdoc(parsed, method.span)),
                    decorators: extract_decorators(&method.function.decorators),
                };
                methods.push(method_def);
            }
            swc_ast::ClassMember::ClassProp(prop) => {
                let name = prop_name_to_string(&prop.key);
                let computed = matches!(&prop.key, swc_ast::PropName::Computed(_));
                let prop_def = ClassPropertyDef {
                    name,
                    ts_name: None,
                    is_static: prop.is_static,
                    accessibility: prop.accessibility.map(accessibility_to_string),
                    is_abstract: prop.is_abstract,
                    is_optional: prop.is_optional,
                    is_override: prop.is_override,
                    readonly: prop.readonly,
                    computed,
                    ts_type: prop
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    doc: Some(extract_jsdoc(parsed, prop.span)),
                    decorators: extract_decorators(&prop.decorators),
                    has_default: prop.value.is_some(),
                };
                properties.push(prop_def);
            }
            swc_ast::ClassMember::TsIndexSignature(sig) => {
                // Extract param info from first param
                let (param_name, index_type) = sig
                    .params
                    .first()
                    .map(|p| match p {
                        swc_ast::TsFnParam::Ident(i) => {
                            let name = i.sym.to_string();
                            let ty = i
                                .type_ann
                                .as_ref()
                                .map(|t| swc_type_to_etch_type(&t.type_ann))
                                .unwrap_or_else(crate::types::EtchType::string);
                            (name, ty)
                        }
                        _ => ("index".to_string(), crate::types::EtchType::string()),
                    })
                    .unwrap_or_else(|| ("index".to_string(), crate::types::EtchType::string()));

                let value_type = sig
                    .type_ann
                    .as_ref()
                    .map(|t| swc_type_to_etch_type(&t.type_ann))
                    .unwrap_or_else(crate::types::EtchType::any);

                let index_sig = crate::class::ClassIndexSignature {
                    param_name,
                    index_type,
                    value_type,
                    readonly: sig.readonly,
                };
                index_signatures.push(index_sig);
            }
            _ => {}
        }
    }

    // Convert extends from Expr to EtchType
    let extends_type = class.super_class.as_ref().map(|e| {
        if let swc_ast::Expr::Ident(i) = e.as_ref() {
            crate::types::EtchType::simple_ref(i.sym.to_string())
        } else {
            crate::types::EtchType::any()
        }
    });

    ClassDef {
        def_name: None,
        is_abstract: class.is_abstract,
        extends: extends_type,
        implements: class
            .implements
            .iter()
            .map(ts_expr_with_type_args_to_etch_type)
            .collect(),
        type_params: extract_type_params(class.type_params.as_deref()),
        constructors,
        properties,
        methods,
        index_signatures,
        decorators: extract_decorators(&class.decorators),
    }
}

/// Extract an interface node
fn extract_interface_node(parsed: &ParsedModule, iface: &swc_ast::TsInterfaceDecl) -> EtchNode {
    use crate::interface::{
        CallSignatureDef, IndexSignatureDef, InterfaceDef, InterfaceMethodDef, InterfacePropertyDef,
    };
    use deno_ast::swc::common::Spanned;

    let name = iface.id.sym.to_string();
    let location = parsed.span_to_location(iface.span());
    let doc = extract_jsdoc(parsed, iface.span());

    let mut properties = Vec::new();
    let mut methods = Vec::new();
    let mut call_signatures = Vec::new();
    let mut index_signatures = Vec::new();

    for member in &iface.body.body {
        match member {
            swc_ast::TsTypeElement::TsPropertySignature(prop) => {
                let name = if let swc_ast::Expr::Ident(i) = prop.key.as_ref() {
                    i.sym.to_string()
                } else {
                    continue;
                };
                properties.push(InterfacePropertyDef {
                    name,
                    ts_type: prop
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    optional: prop.optional,
                    readonly: prop.readonly,
                    doc: extract_jsdoc(parsed, prop.span).description,
                    computed: prop.computed,
                });
            }
            swc_ast::TsTypeElement::TsMethodSignature(method) => {
                let name = if let swc_ast::Expr::Ident(i) = method.key.as_ref() {
                    i.sym.to_string()
                } else {
                    continue;
                };
                methods.push(InterfaceMethodDef {
                    name,
                    params: method
                        .params
                        .iter()
                        .map(extract_param_from_fn_param)
                        .collect(),
                    return_type: method
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    type_params: extract_type_params(method.type_params.as_deref()),
                    optional: method.optional,
                    doc: extract_jsdoc(parsed, method.span).description,
                });
            }
            swc_ast::TsTypeElement::TsCallSignatureDecl(call) => {
                call_signatures.push(CallSignatureDef {
                    params: call
                        .params
                        .iter()
                        .map(extract_param_from_fn_param)
                        .collect(),
                    return_type: call
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    type_params: extract_type_params(call.type_params.as_deref()),
                    doc: extract_jsdoc(parsed, call.span).description,
                });
            }
            swc_ast::TsTypeElement::TsIndexSignature(sig) => {
                // Extract param info from first param
                let (param_name, index_type) = sig
                    .params
                    .first()
                    .map(|p| match p {
                        swc_ast::TsFnParam::Ident(i) => {
                            let name = i.sym.to_string();
                            let ty = i
                                .type_ann
                                .as_ref()
                                .map(|t| swc_type_to_etch_type(&t.type_ann))
                                .unwrap_or_else(crate::types::EtchType::string);
                            (name, ty)
                        }
                        _ => ("index".to_string(), crate::types::EtchType::string()),
                    })
                    .unwrap_or_else(|| ("index".to_string(), crate::types::EtchType::string()));

                let value_type = sig
                    .type_ann
                    .as_ref()
                    .map(|t| swc_type_to_etch_type(&t.type_ann))
                    .unwrap_or_else(crate::types::EtchType::any);

                index_signatures.push(
                    IndexSignatureDef::new(param_name, index_type, value_type)
                        .as_readonly_if(sig.readonly),
                );
            }
            _ => {}
        }
    }

    let interface_def = InterfaceDef {
        def_name: None,
        extends: iface
            .extends
            .iter()
            .map(ts_expr_with_type_args_to_etch_type)
            .collect(),
        type_params: extract_type_params(iface.type_params.as_deref()),
        properties,
        methods,
        call_signatures,
        construct_signatures: vec![],
        index_signatures,
    };

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Interface { interface_def },
        module: None,
    }
}

/// Extract a type alias node
fn extract_type_alias_node(parsed: &ParsedModule, alias: &swc_ast::TsTypeAliasDecl) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = alias.id.sym.to_string();
    let location = parsed.span_to_location(alias.span());
    let doc = extract_jsdoc(parsed, alias.span());

    let type_alias_def = crate::type_alias::TypeAliasDef {
        ts_type: swc_type_to_etch_type(&alias.type_ann),
        type_params: extract_type_params(alias.type_params.as_deref()),
    };

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::TypeAlias { type_alias_def },
        module: None,
    }
}

/// Extract an enum node
fn extract_enum_node(parsed: &ParsedModule, ts_enum: &swc_ast::TsEnumDecl) -> EtchNode {
    use crate::r#enum::{EnumDef, EnumMemberDef, EnumMemberValue};
    use deno_ast::swc::common::Spanned;

    let name = ts_enum.id.sym.to_string();
    let location = parsed.span_to_location(ts_enum.span());
    let doc = extract_jsdoc(parsed, ts_enum.span());

    let members = ts_enum
        .members
        .iter()
        .map(|m| {
            let member_name = match &m.id {
                swc_ast::TsEnumMemberId::Ident(i) => i.sym.to_string(),
                swc_ast::TsEnumMemberId::Str(s) => {
                    String::from_utf8_lossy(s.value.as_bytes()).to_string()
                }
            };
            let init = m.init.as_ref().map(|init| match init.as_ref() {
                swc_ast::Expr::Lit(swc_ast::Lit::Str(s)) => EnumMemberValue::String {
                    value: String::from_utf8_lossy(s.value.as_bytes()).to_string(),
                },
                swc_ast::Expr::Lit(swc_ast::Lit::Num(n)) => {
                    EnumMemberValue::Number { value: n.value }
                }
                _ => EnumMemberValue::Computed {
                    repr: parsed.text_for_span(init.span()).to_string(),
                },
            });

            EnumMemberDef {
                name: member_name,
                init,
                doc: extract_jsdoc(parsed, m.span).description,
            }
        })
        .collect();

    let enum_def = EnumDef {
        is_const: ts_enum.is_const,
        is_declare: false,
        members,
    };

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Enum { enum_def },
        module: None,
    }
}

/// Extract a variable declaration node
fn extract_variable_node(parsed: &ParsedModule, var_decl: &swc_ast::VarDecl) -> Option<EtchNode> {
    use deno_ast::swc::common::Spanned;

    // Get the first declarator
    let declarator = var_decl.decls.first()?;

    let name = match &declarator.name {
        swc_ast::Pat::Ident(i) => i.sym.to_string(),
        _ => return None,
    };

    let location = parsed.span_to_location(var_decl.span());
    let doc = extract_jsdoc(parsed, var_decl.span());

    let ts_type = match &declarator.name {
        swc_ast::Pat::Ident(i) => i
            .type_ann
            .as_ref()
            .map(|t| swc_type_to_etch_type(&t.type_ann)),
        _ => None,
    };

    let kind = match var_decl.kind {
        swc_ast::VarDeclKind::Const => crate::variable::VariableKind::Const,
        swc_ast::VarDeclKind::Let => crate::variable::VariableKind::Let,
        swc_ast::VarDeclKind::Var => crate::variable::VariableKind::Var,
    };

    let variable_def = crate::variable::VariableDef {
        kind,
        ts_type,
        value: declarator
            .init
            .as_ref()
            .map(|e| parsed.text_for_span(e.span()).to_string()),
    };

    Some(EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Variable { variable_def },
        module: None,
    })
}

/// Extract a namespace/module node
fn extract_namespace_node(parsed: &ParsedModule, module: &swc_ast::TsModuleDecl) -> EtchNode {
    use deno_ast::swc::common::Spanned;

    let name = match &module.id {
        swc_ast::TsModuleName::Ident(i) => i.sym.to_string(),
        swc_ast::TsModuleName::Str(s) => String::from_utf8_lossy(s.value.as_bytes()).to_string(),
    };

    let location = parsed.span_to_location(module.span());
    let doc = extract_jsdoc(parsed, module.span());

    let namespace_def = crate::node::NamespaceDef {
        elements: vec![], // Would need recursive extraction
    };

    EtchNode {
        name,
        is_default: None,
        location,
        visibility: Visibility::Private,
        doc,
        def: crate::node::EtchNodeDef::Namespace { namespace_def },
        module: None,
    }
}

// ============================================================================
// Helper functions for converting SWC types
// ============================================================================

/// Extract JSDoc from comments
fn extract_jsdoc(parsed: &ParsedModule, span: deno_ast::swc::common::Span) -> EtchDoc {
    if let Some(jsdoc_text) = parsed.jsdoc_for_span(span) {
        EtchDoc::parse(&jsdoc_text)
    } else {
        EtchDoc::default()
    }
}

/// Convert TsExprWithTypeArgs to EtchType (for implements/extends clauses)
fn ts_expr_with_type_args_to_etch_type(
    expr_with_args: &swc_ast::TsExprWithTypeArgs,
) -> crate::types::EtchType {
    // Extract the base type name from the expression
    let type_name = match expr_with_args.expr.as_ref() {
        swc_ast::Expr::Ident(i) => i.sym.to_string(),
        swc_ast::Expr::Member(m) => {
            // Handle qualified names like Namespace.Type
            let mut parts = vec![];
            let mut current: &swc_ast::Expr = &m.obj;
            loop {
                match current {
                    swc_ast::Expr::Ident(i) => {
                        parts.push(i.sym.to_string());
                        break;
                    }
                    swc_ast::Expr::Member(inner) => {
                        if let swc_ast::MemberProp::Ident(i) = &inner.prop {
                            parts.push(i.sym.to_string());
                        }
                        current = &inner.obj;
                    }
                    _ => break,
                }
            }
            if let swc_ast::MemberProp::Ident(i) = &m.prop {
                parts.push(i.sym.to_string());
            }
            parts.reverse();
            parts.join(".")
        }
        _ => "unknown".to_string(),
    };

    // Extract type arguments if any
    let type_params = expr_with_args
        .type_args
        .as_ref()
        .map(|args| {
            args.params
                .iter()
                .map(|t| swc_type_to_etch_type(t))
                .collect()
        })
        .unwrap_or_default();

    crate::types::EtchType::type_ref(type_name, type_params)
}

/// Convert SWC TsType to EtchType
fn swc_type_to_etch_type(ty: &swc_ast::TsType) -> crate::types::EtchType {
    use crate::types::{EtchLiteral, EtchPrimitive, EtchType, EtchTypeKind};

    match ty {
        swc_ast::TsType::TsKeywordType(kw) => {
            let primitive = match kw.kind {
                swc_ast::TsKeywordTypeKind::TsStringKeyword => EtchPrimitive::String,
                swc_ast::TsKeywordTypeKind::TsNumberKeyword => EtchPrimitive::Number,
                swc_ast::TsKeywordTypeKind::TsBooleanKeyword => EtchPrimitive::Boolean,
                swc_ast::TsKeywordTypeKind::TsVoidKeyword => EtchPrimitive::Void,
                swc_ast::TsKeywordTypeKind::TsNullKeyword => EtchPrimitive::Null,
                swc_ast::TsKeywordTypeKind::TsUndefinedKeyword => EtchPrimitive::Undefined,
                swc_ast::TsKeywordTypeKind::TsNeverKeyword => EtchPrimitive::Never,
                swc_ast::TsKeywordTypeKind::TsUnknownKeyword => EtchPrimitive::Unknown,
                swc_ast::TsKeywordTypeKind::TsAnyKeyword => EtchPrimitive::Any,
                swc_ast::TsKeywordTypeKind::TsObjectKeyword => EtchPrimitive::Object,
                swc_ast::TsKeywordTypeKind::TsSymbolKeyword => EtchPrimitive::Symbol,
                swc_ast::TsKeywordTypeKind::TsBigIntKeyword => EtchPrimitive::BigInt,
                swc_ast::TsKeywordTypeKind::TsIntrinsicKeyword => EtchPrimitive::Unknown,
            };
            EtchType::new(EtchTypeKind::Primitive(primitive))
        }
        swc_ast::TsType::TsTypeRef(ref_type) => {
            let name = match &ref_type.type_name {
                swc_ast::TsEntityName::Ident(i) => i.sym.to_string(),
                swc_ast::TsEntityName::TsQualifiedName(q) => format_qualified_name(q),
            };

            let type_params = ref_type
                .type_params
                .as_ref()
                .map(|params| {
                    params
                        .params
                        .iter()
                        .map(|p| swc_type_to_etch_type(p))
                        .collect()
                })
                .unwrap_or_default();

            EtchType::new(EtchTypeKind::TypeRef { name, type_params })
        }
        swc_ast::TsType::TsArrayType(arr) => {
            let elem = swc_type_to_etch_type(&arr.elem_type);
            EtchType::new(EtchTypeKind::Array(Box::new(elem)))
        }
        swc_ast::TsType::TsTupleType(tuple) => {
            let types = tuple
                .elem_types
                .iter()
                .map(|e| swc_type_to_etch_type(&e.ty))
                .collect();
            EtchType::new(EtchTypeKind::Tuple(types))
        }
        swc_ast::TsType::TsUnionOrIntersectionType(union_inter) => match union_inter {
            swc_ast::TsUnionOrIntersectionType::TsUnionType(u) => {
                let types = u.types.iter().map(|t| swc_type_to_etch_type(t)).collect();
                EtchType::new(EtchTypeKind::Union(types))
            }
            swc_ast::TsUnionOrIntersectionType::TsIntersectionType(i) => {
                let types = i.types.iter().map(|t| swc_type_to_etch_type(t)).collect();
                EtchType::new(EtchTypeKind::Intersection(types))
            }
        },
        swc_ast::TsType::TsLitType(lit) => {
            let literal = match &lit.lit {
                swc_ast::TsLit::Str(s) => {
                    EtchLiteral::String(String::from_utf8_lossy(s.value.as_bytes()).to_string())
                }
                swc_ast::TsLit::Number(n) => EtchLiteral::Number(n.value),
                swc_ast::TsLit::Bool(b) => EtchLiteral::Boolean(b.value),
                swc_ast::TsLit::BigInt(b) => {
                    // Parse BigInt value, defaulting to 0 if parsing fails
                    let bigint_str = format!("{:?}", b.value);
                    let value = bigint_str.parse::<i64>().unwrap_or(0);
                    EtchLiteral::BigInt(value)
                }
                swc_ast::TsLit::Tpl(_) => EtchLiteral::String("template".to_string()),
            };
            EtchType::new(EtchTypeKind::Literal(literal))
        }
        swc_ast::TsType::TsFnOrConstructorType(fn_type) => match fn_type {
            swc_ast::TsFnOrConstructorType::TsFnType(f) => {
                let params: Vec<crate::types::FunctionTypeParam> = f
                    .params
                    .iter()
                    .map(|p| {
                        let param_def = extract_param_from_fn_param(p);
                        crate::types::FunctionTypeParam {
                            name: param_def.name,
                            param_type: param_def.ts_type.unwrap_or_else(EtchType::any),
                            optional: param_def.optional,
                        }
                    })
                    .collect();
                let return_type = swc_type_to_etch_type(&f.type_ann.type_ann);
                let type_params = extract_type_params(f.type_params.as_deref())
                    .into_iter()
                    .map(|tp| crate::types::TypeParamDef {
                        name: tp.name,
                        constraint: tp.constraint,
                        default: tp.default,
                    })
                    .collect();
                EtchType::new(EtchTypeKind::Function(Box::new(
                    crate::types::FunctionTypeDef {
                        type_params,
                        params,
                        return_type: Box::new(return_type),
                        is_constructor: false,
                    },
                )))
            }
            swc_ast::TsFnOrConstructorType::TsConstructorType(c) => {
                let params: Vec<crate::types::FunctionTypeParam> = c
                    .params
                    .iter()
                    .map(|p| {
                        let param_def = extract_param_from_fn_param(p);
                        crate::types::FunctionTypeParam {
                            name: param_def.name,
                            param_type: param_def.ts_type.unwrap_or_else(EtchType::any),
                            optional: param_def.optional,
                        }
                    })
                    .collect();
                let return_type = swc_type_to_etch_type(&c.type_ann.type_ann);
                let type_params = extract_type_params(c.type_params.as_deref())
                    .into_iter()
                    .map(|tp| crate::types::TypeParamDef {
                        name: tp.name,
                        constraint: tp.constraint,
                        default: tp.default,
                    })
                    .collect();
                EtchType::new(EtchTypeKind::Function(Box::new(
                    crate::types::FunctionTypeDef {
                        type_params,
                        params,
                        return_type: Box::new(return_type),
                        is_constructor: true,
                    },
                )))
            }
        },
        swc_ast::TsType::TsTypeLit(lit) => {
            let mut properties = Vec::new();
            let mut methods = Vec::new();

            for member in &lit.members {
                match member {
                    swc_ast::TsTypeElement::TsPropertySignature(prop) => {
                        if let swc_ast::Expr::Ident(i) = prop.key.as_ref() {
                            properties.push(crate::ts_types::TsTypeLiteralProperty {
                                name: i.sym.to_string(),
                                ts_type: prop
                                    .type_ann
                                    .as_ref()
                                    .map(|t| swc_type_to_etch_type(&t.type_ann)),
                                optional: prop.optional,
                                readonly: prop.readonly,
                            });
                        }
                    }
                    swc_ast::TsTypeElement::TsMethodSignature(method) => {
                        if let swc_ast::Expr::Ident(i) = method.key.as_ref() {
                            methods.push(crate::ts_types::TsTypeLiteralMethod {
                                name: i.sym.to_string(),
                                params: method
                                    .params
                                    .iter()
                                    .map(extract_param_from_fn_param)
                                    .collect(),
                                return_type: method
                                    .type_ann
                                    .as_ref()
                                    .map(|t| swc_type_to_etch_type(&t.type_ann)),
                                optional: method.optional,
                            });
                        }
                    }
                    _ => {}
                }
            }

            EtchType::new(EtchTypeKind::TypeLiteral {
                properties,
                methods,
            })
        }
        swc_ast::TsType::TsParenthesizedType(paren) => swc_type_to_etch_type(&paren.type_ann),
        swc_ast::TsType::TsOptionalType(opt) => {
            let inner = swc_type_to_etch_type(&opt.type_ann);
            EtchType::new(EtchTypeKind::Optional(Box::new(inner)))
        }
        swc_ast::TsType::TsRestType(rest) => {
            let inner = swc_type_to_etch_type(&rest.type_ann);
            EtchType::new(EtchTypeKind::Rest(Box::new(inner)))
        }
        swc_ast::TsType::TsTypeQuery(query) => {
            let name = match &query.expr_name {
                swc_ast::TsTypeQueryExpr::TsEntityName(entity) => match entity {
                    swc_ast::TsEntityName::Ident(i) => i.sym.to_string(),
                    swc_ast::TsEntityName::TsQualifiedName(q) => format_qualified_name(q),
                },
                swc_ast::TsTypeQueryExpr::Import(_) => "import".to_string(),
            };
            EtchType::new(EtchTypeKind::TypeQuery(name))
        }
        swc_ast::TsType::TsThisType(_) => EtchType::new(EtchTypeKind::This),
        swc_ast::TsType::TsConditionalType(cond) => EtchType::new(EtchTypeKind::Conditional {
            check_type: Box::new(swc_type_to_etch_type(&cond.check_type)),
            extends_type: Box::new(swc_type_to_etch_type(&cond.extends_type)),
            true_type: Box::new(swc_type_to_etch_type(&cond.true_type)),
            false_type: Box::new(swc_type_to_etch_type(&cond.false_type)),
        }),
        swc_ast::TsType::TsInferType(infer) => {
            EtchType::new(EtchTypeKind::Infer(infer.type_param.name.sym.to_string()))
        }
        swc_ast::TsType::TsMappedType(mapped) => {
            // value_type is required - use template or fallback to unknown
            let value_type = mapped
                .type_ann
                .as_ref()
                .map(|t| Box::new(swc_type_to_etch_type(t)))
                .unwrap_or_else(|| Box::new(EtchType::unknown()));

            EtchType::new(EtchTypeKind::Mapped {
                type_param: mapped.type_param.name.sym.to_string(),
                name_type: mapped
                    .name_type
                    .as_ref()
                    .map(|t| Box::new(swc_type_to_etch_type(t))),
                value_type,
                optional: mapped.optional.map(|o| o == swc_ast::TruePlusMinus::True),
                readonly: mapped.readonly.map(|r| r == swc_ast::TruePlusMinus::True),
                template: mapped
                    .type_ann
                    .as_ref()
                    .map(|t| Box::new(swc_type_to_etch_type(t))),
                constraint: mapped
                    .type_param
                    .constraint
                    .as_ref()
                    .map(|c| Box::new(swc_type_to_etch_type(c))),
            })
        }
        swc_ast::TsType::TsIndexedAccessType(indexed) => {
            EtchType::new(EtchTypeKind::IndexedAccess {
                obj_type: Box::new(swc_type_to_etch_type(&indexed.obj_type)),
                index_type: Box::new(swc_type_to_etch_type(&indexed.index_type)),
            })
        }
        swc_ast::TsType::TsTypeOperator(op) => {
            use crate::types::TypeOperator as EtchTypeOperator;
            let operator = match op.op {
                swc_ast::TsTypeOperatorOp::KeyOf => EtchTypeOperator::KeyOf,
                swc_ast::TsTypeOperatorOp::Unique => EtchTypeOperator::Unique,
                swc_ast::TsTypeOperatorOp::ReadOnly => EtchTypeOperator::Readonly,
            };
            EtchType::new(EtchTypeKind::TypeOperator {
                operator,
                type_arg: Box::new(swc_type_to_etch_type(&op.type_ann)),
            })
        }
        swc_ast::TsType::TsImportType(import) => {
            let arg = String::from_utf8_lossy(import.arg.value.as_bytes()).to_string();
            let qualifier = import.qualifier.as_ref().map(|q| match q {
                swc_ast::TsEntityName::Ident(i) => i.sym.to_string(),
                swc_ast::TsEntityName::TsQualifiedName(q) => format_qualified_name(q),
            });
            EtchType::new(EtchTypeKind::Import { arg, qualifier })
        }
        swc_ast::TsType::TsTypePredicate(pred) => {
            let param_name = match &pred.param_name {
                swc_ast::TsThisTypeOrIdent::TsThisType(_) => "this".to_string(),
                swc_ast::TsThisTypeOrIdent::Ident(i) => i.sym.to_string(),
            };
            let ts_type = pred
                .type_ann
                .as_ref()
                .map(|t| Box::new(swc_type_to_etch_type(&t.type_ann)));
            EtchType::new(EtchTypeKind::TypePredicate {
                param_name,
                ts_type,
                asserts: pred.asserts,
            })
        }
    }
}

/// Format a qualified name (e.g., Namespace.Type)
fn format_qualified_name(name: &swc_ast::TsQualifiedName) -> String {
    let left = match &name.left {
        swc_ast::TsEntityName::Ident(i) => i.sym.to_string(),
        swc_ast::TsEntityName::TsQualifiedName(q) => format_qualified_name(q),
    };
    format!("{}.{}", left, name.right.sym)
}

/// Extract function parameters
fn extract_params(params: &[swc_ast::Param]) -> Vec<crate::params::ParamDef> {
    params.iter().map(|p| extract_param(&p.pat)).collect()
}

/// Extract constructor parameters
fn extract_params_from_ctor(
    params: &[swc_ast::ParamOrTsParamProp],
) -> Vec<crate::params::ParamDef> {
    params
        .iter()
        .filter_map(|p| match p {
            swc_ast::ParamOrTsParamProp::Param(param) => Some(extract_param(&param.pat)),
            swc_ast::ParamOrTsParamProp::TsParamProp(prop) => match &prop.param {
                swc_ast::TsParamPropParam::Ident(i) => Some(crate::params::ParamDef {
                    name: i.sym.to_string(),
                    ts_type: i
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    optional: i.optional,
                    default: None,
                    rest: false,
                    doc: None,
                    decorators: vec![],
                }),
                swc_ast::TsParamPropParam::Assign(a) => {
                    if let swc_ast::Pat::Ident(i) = a.left.as_ref() {
                        Some(crate::params::ParamDef {
                            name: i.sym.to_string(),
                            ts_type: i
                                .type_ann
                                .as_ref()
                                .map(|t| swc_type_to_etch_type(&t.type_ann)),
                            optional: true,
                            default: None,
                            rest: false,
                            doc: None,
                            decorators: vec![],
                        })
                    } else {
                        None
                    }
                }
            },
        })
        .collect()
}

/// Extract a single parameter from a pattern
fn extract_param(pat: &swc_ast::Pat) -> crate::params::ParamDef {
    match pat {
        swc_ast::Pat::Ident(i) => crate::params::ParamDef {
            name: i.sym.to_string(),
            ts_type: i
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: i.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::Pat::Rest(r) => {
            let mut param = extract_param(&r.arg);
            param.rest = true;
            param
        }
        swc_ast::Pat::Assign(a) => {
            let mut param = extract_param(&a.left);
            param.optional = true;
            param
        }
        swc_ast::Pat::Array(arr) => crate::params::ParamDef {
            name: "destructured".to_string(),
            ts_type: arr
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: arr.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::Pat::Object(obj) => crate::params::ParamDef {
            name: "destructured".to_string(),
            ts_type: obj
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: obj.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::Pat::Expr(_) | swc_ast::Pat::Invalid(_) => crate::params::ParamDef {
            name: "unknown".to_string(),
            ts_type: None,
            optional: false,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
    }
}

/// Extract a parameter from a TsFnParam
fn extract_param_from_fn_param(param: &swc_ast::TsFnParam) -> crate::params::ParamDef {
    match param {
        swc_ast::TsFnParam::Ident(i) => crate::params::ParamDef {
            name: i.sym.to_string(),
            ts_type: i
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: i.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::TsFnParam::Array(arr) => crate::params::ParamDef {
            name: "destructured".to_string(),
            ts_type: arr
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: arr.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::TsFnParam::Object(obj) => crate::params::ParamDef {
            name: "destructured".to_string(),
            ts_type: obj
                .type_ann
                .as_ref()
                .map(|t| swc_type_to_etch_type(&t.type_ann)),
            optional: obj.optional,
            default: None,
            rest: false,
            doc: None,
            decorators: vec![],
        },
        swc_ast::TsFnParam::Rest(r) => {
            let mut param = match r.arg.as_ref() {
                swc_ast::Pat::Ident(i) => crate::params::ParamDef {
                    name: i.sym.to_string(),
                    ts_type: i
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    optional: false,
                    default: None,
                    rest: false,
                    doc: None,
                    decorators: vec![],
                },
                _ => crate::params::ParamDef {
                    name: "rest".to_string(),
                    ts_type: r
                        .type_ann
                        .as_ref()
                        .map(|t| swc_type_to_etch_type(&t.type_ann)),
                    optional: false,
                    default: None,
                    rest: false,
                    doc: None,
                    decorators: vec![],
                },
            };
            param.rest = true;
            param
        }
    }
}

/// Extract type parameters
fn extract_type_params(
    params: Option<&swc_ast::TsTypeParamDecl>,
) -> Vec<crate::ts_type_params::TsTypeParamDef> {
    params
        .map(|p| {
            p.params
                .iter()
                .map(|param| crate::ts_type_params::TsTypeParamDef {
                    name: param.name.sym.to_string(),
                    constraint: param
                        .constraint
                        .as_ref()
                        .map(|c| Box::new(swc_type_to_etch_type(c))),
                    default: param
                        .default
                        .as_ref()
                        .map(|d| Box::new(swc_type_to_etch_type(d))),
                    is_const: param.is_const,
                    is_in: param.is_in,
                    is_out: param.is_out,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Extract decorators
fn extract_decorators(decorators: &[swc_ast::Decorator]) -> Vec<crate::decorators::DecoratorDef> {
    decorators
        .iter()
        .map(|d| {
            let (name, args, is_factory) = match d.expr.as_ref() {
                swc_ast::Expr::Ident(i) => (i.sym.to_string(), vec![], false),
                swc_ast::Expr::Call(c) => {
                    let name = match c.callee.as_expr() {
                        Some(expr) => match expr.as_ref() {
                            swc_ast::Expr::Ident(i) => i.sym.to_string(),
                            _ => "unknown".to_string(),
                        },
                        None => "unknown".to_string(),
                    };
                    let args = c.args.iter().map(|a| format!("{:?}", a.expr)).collect();
                    (name, args, true)
                }
                _ => ("unknown".to_string(), vec![], false),
            };

            crate::decorators::DecoratorDef {
                name,
                args,
                is_factory,
                text: None,
            }
        })
        .collect()
}

/// Convert accessibility to string
fn accessibility_to_string(access: swc_ast::Accessibility) -> String {
    match access {
        swc_ast::Accessibility::Public => "public".to_string(),
        swc_ast::Accessibility::Protected => "protected".to_string(),
        swc_ast::Accessibility::Private => "private".to_string(),
    }
}

/// Convert property name to string
fn prop_name_to_string(name: &swc_ast::PropName) -> String {
    match name {
        swc_ast::PropName::Ident(i) => i.sym.to_string(),
        swc_ast::PropName::Str(s) => String::from_utf8_lossy(s.value.as_bytes()).to_string(),
        swc_ast::PropName::Num(n) => n.value.to_string(),
        swc_ast::PropName::BigInt(b) => format!("{:?}", b.value),
        swc_ast::PropName::Computed(c) => {
            if let swc_ast::Expr::Ident(i) = c.expr.as_ref() {
                format!("[{}]", i.sym)
            } else {
                "[computed]".to_string()
            }
        }
    }
}

// ============================================================================
// forge-weld integration
// ============================================================================

/// Convert a WeldModule to EtchNodes
pub fn weld_module_to_nodes(module: &WeldModule) -> Vec<EtchNode> {
    let mut nodes = Vec::new();

    // Convert ops
    for op in &module.ops {
        nodes.push(op_symbol_to_node(op, &module.specifier));
    }

    // Convert structs
    for s in &module.structs {
        nodes.push(weld_struct_to_node(s, &module.specifier));
    }

    // Convert enums
    for e in &module.enums {
        nodes.push(weld_enum_to_node(e, &module.specifier));
    }

    nodes
}

/// Convert an OpSymbol to an EtchNode
fn op_symbol_to_node(op: &OpSymbol, module: &str) -> EtchNode {
    use crate::function::OpDef;

    let params = op
        .params
        .iter()
        .filter(|p| !matches!(p.attr, forge_weld::ir::ParamAttr::State))
        .map(|p| crate::params::ParamDef {
            name: p.ts_name.clone(),
            ts_type: Some(crate::types::EtchType::from(&p.ty)),
            optional: p.optional,
            default: None,
            rest: false,
            doc: p.doc.clone(),
            decorators: vec![],
        })
        .collect();

    let op_def = OpDef {
        rust_name: op.rust_name.clone(),
        ts_name: op.ts_name.clone(),
        is_async: op.is_async,
        params,
        return_type: op.ts_return_type(),
        return_type_def: Some(crate::types::EtchType::from(&op.return_type)),
        op_attrs: crate::function::OpAttrs {
            is_async: op.is_async,
            fast: op.op2_attrs.is_fast,
            reentrant: false,
            attrs: Default::default(),
        },
        type_params: vec![],
        can_throw: matches!(op.return_type, forge_weld::ir::WeldType::Result { .. }),
        permissions: None,
    };

    EtchNode {
        name: op.ts_name.clone(),
        is_default: None,
        location: Location::default(),
        visibility: Visibility::Public,
        doc: EtchDoc {
            description: op.doc.clone(),
            tags: vec![],
        },
        def: crate::node::EtchNodeDef::Op { op_def },
        module: Some(module.to_string()),
    }
}

/// Convert a WeldStruct to an EtchNode
fn weld_struct_to_node(s: &WeldStruct, module: &str) -> EtchNode {
    let struct_def = crate::node::StructDef {
        rust_name: s.rust_name.clone(),
        ts_name: s.ts_name.clone(),
        fields: s
            .fields
            .iter()
            .map(|f| crate::node::StructFieldDef {
                name: f.rust_name.clone(),
                ts_name: f.ts_name.clone(),
                ts_type: crate::types::EtchType::from(&f.ty).to_typescript(),
                optional: f.optional,
                readonly: f.readonly,
                doc: f.doc.clone(),
            })
            .collect(),
        type_params: s.type_params.clone(),
    };

    EtchNode {
        name: s.ts_name.clone(),
        is_default: None,
        location: Location::default(),
        visibility: Visibility::Public,
        doc: EtchDoc {
            description: s.doc.clone(),
            tags: vec![],
        },
        def: crate::node::EtchNodeDef::Struct { struct_def },
        module: Some(module.to_string()),
    }
}

/// Convert a WeldEnum to an EtchNode
fn weld_enum_to_node(e: &WeldEnum, module: &str) -> EtchNode {
    use crate::r#enum::{EnumDef, EnumMemberDef, EnumMemberValue};

    let members = e
        .variants
        .iter()
        .map(|v| EnumMemberDef {
            name: v.name.clone(),
            init: v
                .value
                .as_ref()
                .map(|val| EnumMemberValue::String { value: val.clone() }),
            doc: v.doc.clone(),
        })
        .collect();

    let enum_def = EnumDef {
        members,
        is_const: false,
        is_declare: false,
    };

    EtchNode {
        name: e.ts_name.clone(),
        is_default: None,
        location: Location::default(),
        visibility: Visibility::Public,
        doc: EtchDoc {
            description: e.doc.clone(),
            tags: vec![],
        },
        def: crate::node::EtchNodeDef::Enum { enum_def },
        module: Some(module.to_string()),
    }
}

/// Merge TypeScript nodes with forge-weld nodes
///
/// TypeScript JSDoc takes precedence for documentation, but forge-weld
/// provides accurate type information from Rust.
pub fn merge_nodes(ts_nodes: Vec<EtchNode>, weld_nodes: Vec<EtchNode>) -> Vec<EtchNode> {
    let mut result = IndexMap::new();

    // First, add all weld nodes (they have accurate types)
    for node in weld_nodes {
        result.insert(node.name.clone(), node);
    }

    // Then, merge TypeScript nodes (JSDoc takes precedence)
    for ts_node in ts_nodes {
        if let Some(existing) = result.get_mut(&ts_node.name) {
            // Merge documentation - TypeScript JSDoc takes precedence
            if ts_node.doc.description.is_some() {
                existing.doc.description = ts_node.doc.description;
            }
            if !ts_node.doc.tags.is_empty() {
                existing.doc.tags = ts_node.doc.tags;
            }
            // Keep the weld location if we have one
            if existing.location.is_unknown() {
                existing.location = ts_node.location;
            }
        } else {
            // TypeScript-only symbol
            result.insert(ts_node.name.clone(), ts_node);
        }
    }

    result.into_values().collect()
}

// ============================================================================
// Public API for docgen/rust.rs
// ============================================================================

/// Public wrapper for op_symbol_to_node
pub fn op_symbol_to_node_pub(op: &OpSymbol, module: &str) -> EtchNode {
    op_symbol_to_node(op, module)
}

/// Public wrapper for weld_struct_to_node
pub fn weld_struct_to_node_pub(s: &WeldStruct, module: &str) -> EtchNode {
    weld_struct_to_node(s, module)
}

/// Public wrapper for weld_enum_to_node
pub fn weld_enum_to_node_pub(e: &WeldEnum, module: &str) -> EtchNode {
    weld_enum_to_node(e, module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let source = r#"
/**
 * Reads a file as text
 * @param path - The file path
 * @returns The contents
 */
export async function readTextFile(path: string): Promise<string> {
    return "";
}
"#;

        // deno_ast requires absolute paths for file specifiers
        let nodes = parse_typescript_str("/tmp/test.ts", source).unwrap();
        assert_eq!(nodes.len(), 1);

        let node = &nodes[0];
        assert_eq!(node.name, "readTextFile");
        assert!(matches!(
            node.def,
            crate::node::EtchNodeDef::Function { .. }
        ));
        assert!(node.doc.description.is_some());
    }

    #[test]
    fn test_parse_interface() {
        let source = r#"
/**
 * File statistics
 */
export interface FileStat {
    /** File size in bytes */
    size: number;
    /** Is this a directory? */
    isDirectory: boolean;
}
"#;

        // deno_ast requires absolute paths for file specifiers
        let nodes = parse_typescript_str("/tmp/test.ts", source).unwrap();
        assert_eq!(nodes.len(), 1);

        let node = &nodes[0];
        assert_eq!(node.name, "FileStat");
        assert!(matches!(
            node.def,
            crate::node::EtchNodeDef::Interface { .. }
        ));
    }

    #[test]
    fn test_parse_class() {
        let source = r#"
export class FileReader {
    private path: string;

    constructor(path: string) {
        this.path = path;
    }

    async read(): Promise<string> {
        return "";
    }
}
"#;

        // deno_ast requires absolute paths for file specifiers
        let nodes = parse_typescript_str("/tmp/test.ts", source).unwrap();
        assert_eq!(nodes.len(), 1);

        let node = &nodes[0];
        assert_eq!(node.name, "FileReader");
        assert!(matches!(node.def, crate::node::EtchNodeDef::Class { .. }));
    }
}
