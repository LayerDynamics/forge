//! Test utilities and mock objects for forge-etch.
//!
//! This module provides utilities for testing forge-etch functionality,
//! including mock node generators and test fixtures.
//!
//! # Example
//!
//! ```
//! use forge_etch::test::{mock_function_node, mock_op_node, mock_extension_doc};
//!
//! let func = mock_function_node("readFile");
//! let op = mock_op_node("readTextFile", true);
//! let doc = mock_extension_doc("fs", "runtime:fs");
//! ```

use std::path::PathBuf;

use crate::class::{ClassDef, ClassMethodDef, ClassPropertyDef};
use crate::docgen::ExtensionDoc;
use crate::function::{FunctionDef, OpDef};
use crate::interface::{InterfaceDef, InterfaceMethodDef, InterfacePropertyDef};
use crate::js_doc::{EtchDoc, JsDocTag};
use crate::node::{EtchNode, EtchNodeDef, Location};
use crate::params::ParamDef;
use crate::r#enum::{EnumDef, EnumMemberDef};
use crate::type_alias::TypeAliasDef;
use crate::types::{EtchPrimitive, EtchType, EtchTypeKind};
use crate::variable::{VariableDef, VariableKind};
use crate::visibility::Visibility;

/// Create a mock location for test nodes.
///
/// Returns a location pointing to "test.ts" at line 1, column 0.
pub fn mock_location() -> Location {
    Location::new("test.ts", 1, 0)
}

/// Create a mock location with custom values.
pub fn mock_location_at(filename: &str, line: usize, col: usize) -> Location {
    Location::new(filename, line, col)
}

/// Create a simple mock EtchDoc with optional description.
pub fn mock_doc(description: Option<&str>) -> EtchDoc {
    EtchDoc {
        description: description.map(String::from),
        tags: vec![],
    }
}

/// Create a mock EtchDoc with description and tags.
pub fn mock_doc_with_tags(description: &str, tags: Vec<JsDocTag>) -> EtchDoc {
    EtchDoc {
        description: Some(description.to_string()),
        tags,
    }
}

/// Create a mock function node.
///
/// # Arguments
///
/// * `name` - The function name
///
/// # Returns
///
/// An `EtchNode` with a `Function` definition.
///
/// # Example
///
/// ```
/// use forge_etch::test::mock_function_node;
///
/// let node = mock_function_node("processData");
/// assert_eq!(node.name, "processData");
/// ```
pub fn mock_function_node(name: &str) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock function {}", name))),
        def: EtchNodeDef::Function {
            function_def: FunctionDef {
                def_name: None,
                params: vec![],
                return_type: Some(EtchType::void()),
                type_params: vec![],
                is_async: false,
                is_generator: false,
                has_body: true,
                decorators: vec![],
                overloads: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock function node with parameters.
///
/// # Arguments
///
/// * `name` - The function name
/// * `params` - Parameter definitions
/// * `return_type` - The return type
pub fn mock_function_with_params(
    name: &str,
    params: Vec<ParamDef>,
    return_type: EtchType,
) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock function {}", name))),
        def: EtchNodeDef::Function {
            function_def: FunctionDef {
                def_name: None,
                params,
                return_type: Some(return_type),
                type_params: vec![],
                is_async: false,
                is_generator: false,
                has_body: true,
                decorators: vec![],
                overloads: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock async function node.
pub fn mock_async_function_node(name: &str) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock async function {}", name))),
        def: EtchNodeDef::Function {
            function_def: FunctionDef {
                def_name: None,
                params: vec![],
                return_type: Some(EtchType::new(EtchTypeKind::Promise(Box::new(
                    EtchType::void(),
                )))),
                type_params: vec![],
                is_async: true,
                is_generator: false,
                has_body: true,
                decorators: vec![],
                overloads: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock op node.
///
/// # Arguments
///
/// * `name` - The op name (TypeScript export name)
/// * `is_async` - Whether the op is async
///
/// # Returns
///
/// An `EtchNode` with an `Op` definition.
///
/// # Example
///
/// ```
/// use forge_etch::test::mock_op_node;
///
/// let op = mock_op_node("readTextFile", true);
/// assert_eq!(op.name, "readTextFile");
/// ```
pub fn mock_op_node(name: &str, is_async: bool) -> EtchNode {
    let rust_name = format!("op_{}", name.to_lowercase().replace('-', "_"));
    let return_type = if is_async {
        "Promise<string>".to_string()
    } else {
        "string".to_string()
    };

    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock op {}", name))),
        def: EtchNodeDef::Op {
            op_def: OpDef {
                rust_name,
                ts_name: name.to_string(),
                is_async,
                params: vec![],
                return_type,
                return_type_def: None,
                op_attrs: Default::default(),
                type_params: vec![],
                can_throw: false,
                permissions: None,
            },
        },
        module: None,
    }
}

/// Create a mock op node with parameters.
pub fn mock_op_with_params(name: &str, is_async: bool, params: Vec<ParamDef>) -> EtchNode {
    let rust_name = format!("op_{}", name.to_lowercase().replace('-', "_"));
    let return_type = if is_async {
        "Promise<void>".to_string()
    } else {
        "void".to_string()
    };

    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock op {}", name))),
        def: EtchNodeDef::Op {
            op_def: OpDef {
                rust_name,
                ts_name: name.to_string(),
                is_async,
                params,
                return_type,
                return_type_def: None,
                op_attrs: Default::default(),
                type_params: vec![],
                can_throw: false,
                permissions: None,
            },
        },
        module: None,
    }
}

/// Create a mock class node.
///
/// # Arguments
///
/// * `name` - The class name
///
/// # Returns
///
/// An `EtchNode` with a `Class` definition.
pub fn mock_class_node(name: &str) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock class {}", name))),
        def: EtchNodeDef::Class {
            class_def: ClassDef::default(),
        },
        module: None,
    }
}

/// Create a mock class node with members.
pub fn mock_class_with_members(
    name: &str,
    properties: Vec<ClassPropertyDef>,
    methods: Vec<ClassMethodDef>,
) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock class {}", name))),
        def: EtchNodeDef::Class {
            class_def: ClassDef {
                def_name: None,
                is_abstract: false,
                constructors: vec![],
                properties,
                methods,
                index_signatures: vec![],
                extends: None,
                implements: vec![],
                type_params: vec![],
                decorators: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock interface node.
///
/// # Arguments
///
/// * `name` - The interface name
///
/// # Returns
///
/// An `EtchNode` with an `Interface` definition.
pub fn mock_interface_node(name: &str) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock interface {}", name))),
        def: EtchNodeDef::Interface {
            interface_def: InterfaceDef {
                def_name: None,
                extends: vec![],
                properties: vec![],
                methods: vec![],
                call_signatures: vec![],
                construct_signatures: vec![],
                index_signatures: vec![],
                type_params: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock interface node with members.
pub fn mock_interface_with_members(
    name: &str,
    properties: Vec<InterfacePropertyDef>,
    methods: Vec<InterfaceMethodDef>,
) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock interface {}", name))),
        def: EtchNodeDef::Interface {
            interface_def: InterfaceDef {
                def_name: None,
                extends: vec![],
                properties,
                methods,
                call_signatures: vec![],
                construct_signatures: vec![],
                index_signatures: vec![],
                type_params: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock enum node.
///
/// # Arguments
///
/// * `name` - The enum name
///
/// # Returns
///
/// An `EtchNode` with an `Enum` definition.
pub fn mock_enum_node(name: &str) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock enum {}", name))),
        def: EtchNodeDef::Enum {
            enum_def: EnumDef::default(),
        },
        module: None,
    }
}

/// Create a mock enum node with members.
pub fn mock_enum_with_members(name: &str, members: Vec<EnumMemberDef>) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock enum {}", name))),
        def: EtchNodeDef::Enum {
            enum_def: EnumDef {
                members,
                is_const: false,
                is_declare: false,
            },
        },
        module: None,
    }
}

/// Create a mock type alias node.
///
/// # Arguments
///
/// * `name` - The type alias name
/// * `ty` - The aliased type
pub fn mock_type_alias_node(name: &str, ty: EtchType) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock type alias {}", name))),
        def: EtchNodeDef::TypeAlias {
            type_alias_def: TypeAliasDef {
                ts_type: ty,
                type_params: vec![],
            },
        },
        module: None,
    }
}

/// Create a mock variable node.
///
/// # Arguments
///
/// * `name` - The variable name
/// * `kind` - The variable kind (const, let, var)
/// * `ty` - Optional type annotation
pub fn mock_variable_node(name: &str, kind: VariableKind, ty: Option<EtchType>) -> EtchNode {
    EtchNode {
        name: name.to_string(),
        is_default: None,
        location: mock_location(),
        visibility: Visibility::Public,
        doc: mock_doc(Some(&format!("Mock variable {}", name))),
        def: EtchNodeDef::Variable {
            variable_def: VariableDef {
                kind,
                ts_type: ty,
                value: None,
            },
        },
        module: None,
    }
}

/// Create a mock constant variable node.
pub fn mock_const_node(name: &str, ty: EtchType) -> EtchNode {
    mock_variable_node(name, VariableKind::Const, Some(ty))
}

/// Create a mock ExtensionDoc.
///
/// # Arguments
///
/// * `name` - The extension internal name
/// * `specifier` - The module specifier (e.g., "runtime:fs")
///
/// # Returns
///
/// An `ExtensionDoc` with some default mock nodes.
///
/// # Example
///
/// ```
/// use forge_etch::test::mock_extension_doc;
///
/// let doc = mock_extension_doc("fs", "runtime:fs");
/// assert_eq!(doc.name, "fs");
/// assert_eq!(doc.specifier, "runtime:fs");
/// ```
pub fn mock_extension_doc(name: &str, specifier: &str) -> ExtensionDoc {
    ExtensionDoc {
        name: name.to_string(),
        specifier: specifier.to_string(),
        title: format!("{} Module", name),
        description: Some(format!("Mock documentation for the {} module", name)),
        nodes: vec![
            mock_function_node("foo"),
            mock_function_node("bar"),
            mock_op_node("baz", true),
        ],
        module_doc: Some(mock_doc(Some(&format!(
            "Module-level documentation for {}",
            name
        )))),
    }
}

/// Create a mock ExtensionDoc with custom nodes.
pub fn mock_extension_doc_with_nodes(
    name: &str,
    specifier: &str,
    nodes: Vec<EtchNode>,
) -> ExtensionDoc {
    ExtensionDoc {
        name: name.to_string(),
        specifier: specifier.to_string(),
        title: format!("{} Module", name),
        description: Some(format!("Mock documentation for the {} module", name)),
        nodes,
        module_doc: None,
    }
}

/// Get the test fixtures directory path.
///
/// Returns the path to `tests/fixtures` relative to the crate root.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Create a mock ParamDef with a string type.
pub fn mock_string_param(name: &str) -> ParamDef {
    ParamDef::new(name, EtchType::string())
}

/// Create a mock ParamDef with a number type.
pub fn mock_number_param(name: &str) -> ParamDef {
    ParamDef::new(name, EtchType::number())
}

/// Create a mock optional ParamDef.
pub fn mock_optional_param(name: &str, ty: EtchType) -> ParamDef {
    ParamDef::new(name, ty).as_optional()
}

/// Create a mock ClassPropertyDef.
pub fn mock_class_property(name: &str, ty: EtchType) -> ClassPropertyDef {
    ClassPropertyDef {
        name: name.to_string(),
        ts_name: None,
        ts_type: Some(ty),
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

/// Create a mock InterfacePropertyDef.
pub fn mock_interface_property(name: &str, ty: EtchType, optional: bool) -> InterfacePropertyDef {
    InterfacePropertyDef {
        name: name.to_string(),
        ts_type: Some(ty),
        readonly: false,
        optional,
        computed: false,
        doc: None,
    }
}

/// Create common test types.
pub mod types {
    use super::*;

    /// Create a string type
    pub fn string() -> EtchType {
        EtchType::string()
    }

    /// Create a number type
    pub fn number() -> EtchType {
        EtchType::number()
    }

    /// Create a boolean type
    pub fn boolean() -> EtchType {
        EtchType::boolean()
    }

    /// Create a void type
    pub fn void() -> EtchType {
        EtchType::void()
    }

    /// Create an any type
    pub fn any() -> EtchType {
        EtchType::any()
    }

    /// Create a Promise type
    pub fn promise(inner: EtchType) -> EtchType {
        EtchType::new(EtchTypeKind::Promise(Box::new(inner)))
    }

    /// Create an array type
    pub fn array(inner: EtchType) -> EtchType {
        EtchType::new(EtchTypeKind::Array(Box::new(inner)))
    }

    /// Create a union type
    pub fn union(types: Vec<EtchType>) -> EtchType {
        EtchType::new(EtchTypeKind::Union(types))
    }

    /// Create an optional type (T | undefined)
    pub fn optional(inner: EtchType) -> EtchType {
        union(vec![inner, EtchType::primitive(EtchPrimitive::Undefined)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::EtchNodeKind;

    #[test]
    fn test_mock_location() {
        let loc = mock_location();
        assert_eq!(loc.filename, "test.ts");
        assert_eq!(loc.line, 1);
        assert_eq!(loc.col, 0);
    }

    #[test]
    fn test_mock_function_node() {
        let node = mock_function_node("testFunc");
        assert_eq!(node.name, "testFunc");
        assert_eq!(node.kind(), EtchNodeKind::Function);
        assert!(node.visibility.is_public());
    }

    #[test]
    fn test_mock_op_node() {
        let node = mock_op_node("readFile", true);
        assert_eq!(node.name, "readFile");
        assert_eq!(node.kind(), EtchNodeKind::Op);

        if let EtchNodeDef::Op { op_def } = &node.def {
            assert!(op_def.is_async);
            assert_eq!(op_def.rust_name, "op_readfile");
        } else {
            panic!("Expected Op node");
        }
    }

    #[test]
    fn test_mock_class_node() {
        let node = mock_class_node("TestClass");
        assert_eq!(node.name, "TestClass");
        assert_eq!(node.kind(), EtchNodeKind::Class);
    }

    #[test]
    fn test_mock_interface_node() {
        let node = mock_interface_node("ITestInterface");
        assert_eq!(node.name, "ITestInterface");
        assert_eq!(node.kind(), EtchNodeKind::Interface);
    }

    #[test]
    fn test_mock_enum_node() {
        let node = mock_enum_node("TestEnum");
        assert_eq!(node.name, "TestEnum");
        assert_eq!(node.kind(), EtchNodeKind::Enum);
    }

    #[test]
    fn test_mock_extension_doc() {
        let doc = mock_extension_doc("test_ext", "runtime:test");
        assert_eq!(doc.name, "test_ext");
        assert_eq!(doc.specifier, "runtime:test");
        assert_eq!(doc.title, "test_ext Module");
        assert!(!doc.nodes.is_empty());
    }

    #[test]
    fn test_mock_params() {
        let string_param = mock_string_param("name");
        assert_eq!(string_param.name, "name");
        assert!(!string_param.optional);

        let number_param = mock_number_param("count");
        assert_eq!(number_param.name, "count");

        let optional_param = mock_optional_param("options", types::any());
        assert!(optional_param.optional);
    }

    #[test]
    fn test_types_module() {
        assert!(matches!(types::string().kind, EtchTypeKind::Primitive(_)));
        assert!(matches!(
            types::promise(types::string()).kind,
            EtchTypeKind::Promise(_)
        ));
        assert!(matches!(
            types::array(types::number()).kind,
            EtchTypeKind::Array(_)
        ));
    }
}
