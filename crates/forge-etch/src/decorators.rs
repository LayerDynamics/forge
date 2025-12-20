//! Decorator definitions
//!
//! This module provides types for representing TypeScript decorators
//! in documentation. Decorators are metadata annotations that can be
//! applied to classes, methods, properties, and parameters.

use serde::{Deserialize, Serialize};

/// Decorator definition
///
/// Represents a TypeScript decorator like:
/// - `@Injectable()`
/// - `@Component({ selector: 'app-root' })`
/// - `@deprecated`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct DecoratorDef {
    /// Decorator name (without @)
    pub name: String,

    /// Decorator arguments (as string representation)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// Full text representation (e.g., "@Component({ selector: 'app' })")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub text: Option<String>,

    /// Whether this is a decorator factory (called with parentheses)
    #[serde(default)]
    pub is_factory: bool,
}

impl DecoratorDef {
    /// Create a new decorator
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: vec![],
            text: None,
            is_factory: false,
        }
    }

    /// Create a decorator factory (with parentheses)
    pub fn factory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: vec![],
            text: None,
            is_factory: true,
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self.is_factory = true;
        self
    }

    /// Add multiple arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        if !self.args.is_empty() {
            self.is_factory = true;
        }
        self
    }

    /// Set the full text representation
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Generate TypeScript decorator syntax
    pub fn to_typescript(&self) -> String {
        if let Some(ref text) = self.text {
            return format!("@{}", text);
        }

        if self.is_factory {
            if self.args.is_empty() {
                format!("@{}()", self.name)
            } else {
                format!("@{}({})", self.name, self.args.join(", "))
            }
        } else {
            format!("@{}", self.name)
        }
    }

    /// Check if this is a specific decorator by name
    pub fn is(&self, name: &str) -> bool {
        self.name == name || self.name.ends_with(&format!("::{}", name))
    }
}

/// Common decorator names in TypeScript/JavaScript ecosystems
pub mod common_decorators {
    /// Angular decorators
    pub mod angular {
        pub const COMPONENT: &str = "Component";
        pub const DIRECTIVE: &str = "Directive";
        pub const INJECTABLE: &str = "Injectable";
        pub const NG_MODULE: &str = "NgModule";
        pub const PIPE: &str = "Pipe";
        pub const INPUT: &str = "Input";
        pub const OUTPUT: &str = "Output";
        pub const HOST_LISTENER: &str = "HostListener";
        pub const HOST_BINDING: &str = "HostBinding";
        pub const VIEW_CHILD: &str = "ViewChild";
        pub const CONTENT_CHILD: &str = "ContentChild";
    }

    /// NestJS decorators
    pub mod nestjs {
        pub const CONTROLLER: &str = "Controller";
        pub const GET: &str = "Get";
        pub const POST: &str = "Post";
        pub const PUT: &str = "Put";
        pub const DELETE: &str = "Delete";
        pub const PATCH: &str = "Patch";
        pub const PARAM: &str = "Param";
        pub const BODY: &str = "Body";
        pub const QUERY: &str = "Query";
        pub const INJECTABLE: &str = "Injectable";
        pub const MODULE: &str = "Module";
    }

    /// TypeORM decorators
    pub mod typeorm {
        pub const ENTITY: &str = "Entity";
        pub const COLUMN: &str = "Column";
        pub const PRIMARY_COLUMN: &str = "PrimaryColumn";
        pub const PRIMARY_GENERATED_COLUMN: &str = "PrimaryGeneratedColumn";
        pub const ONE_TO_MANY: &str = "OneToMany";
        pub const MANY_TO_ONE: &str = "ManyToOne";
        pub const MANY_TO_MANY: &str = "ManyToMany";
        pub const JOIN_COLUMN: &str = "JoinColumn";
    }

    /// Stage 3 decorators / TC39
    pub mod standard {
        pub const DEPRECATED: &str = "deprecated";
        pub const OVERRIDE: &str = "override";
    }
}

/// Helper trait for checking decorator presence
pub trait HasDecorators {
    /// Get all decorators
    fn decorators(&self) -> &[DecoratorDef];

    /// Check if a specific decorator is present
    fn has_decorator(&self, name: &str) -> bool {
        self.decorators().iter().any(|d| d.is(name))
    }

    /// Get a specific decorator by name
    fn get_decorator(&self, name: &str) -> Option<&DecoratorDef> {
        self.decorators().iter().find(|d| d.is(name))
    }

    /// Check if deprecated decorator is present
    fn is_deprecated(&self) -> bool {
        self.has_decorator("deprecated") || self.has_decorator("Deprecated")
    }
}

/// Parameter decorator information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamDecoratorDef {
    /// Parameter index
    pub index: usize,

    /// Decorator applied
    pub decorator: DecoratorDef,
}

impl ParamDecoratorDef {
    /// Create a new parameter decorator
    pub fn new(index: usize, decorator: DecoratorDef) -> Self {
        Self { index, decorator }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_decorator() {
        let dec = DecoratorDef::new("deprecated");
        assert_eq!(dec.to_typescript(), "@deprecated");
        assert!(!dec.is_factory);
    }

    #[test]
    fn test_decorator_factory() {
        let dec = DecoratorDef::factory("Injectable");
        assert_eq!(dec.to_typescript(), "@Injectable()");
        assert!(dec.is_factory);
    }

    #[test]
    fn test_decorator_with_args() {
        let dec = DecoratorDef::factory("Component").with_arg("{ selector: 'app-root' }");

        assert_eq!(dec.to_typescript(), "@Component({ selector: 'app-root' })");
    }

    #[test]
    fn test_decorator_with_text() {
        let dec = DecoratorDef::new("Route").with_text("Route('/api')");
        assert_eq!(dec.to_typescript(), "@Route('/api')");
    }

    #[test]
    fn test_decorator_is_check() {
        let dec = DecoratorDef::new("Injectable");
        assert!(dec.is("Injectable"));
        assert!(!dec.is("Component"));
    }

    #[test]
    fn test_has_decorators_trait() {
        struct TestClass {
            decorators: Vec<DecoratorDef>,
        }

        impl HasDecorators for TestClass {
            fn decorators(&self) -> &[DecoratorDef] {
                &self.decorators
            }
        }

        let class = TestClass {
            decorators: vec![
                DecoratorDef::new("deprecated"),
                DecoratorDef::factory("Injectable"),
            ],
        };

        assert!(class.is_deprecated());
        assert!(class.has_decorator("Injectable"));
        assert!(!class.has_decorator("Component"));
    }
}
