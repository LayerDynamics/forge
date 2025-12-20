//! Extensibility configuration for Forge extensions
//!
//! This module provides types for configuring the extensibility features
//! of generated extension modules, including lifecycle hooks, custom handlers,
//! and configuration options.

use crate::ir::WeldType;
use serde::{Deserialize, Serialize};

/// Configuration for extension extensibility features
///
/// When enabled, the TypeScript generator will include hook registration,
/// handler registration, and configuration APIs in the generated SDK module.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensibilityConfig {
    /// Whether lifecycle hooks are enabled for this extension
    ///
    /// When true, generates:
    /// - `onBefore(opName, callback)` - Called before operation executes
    /// - `onAfter(opName, callback)` - Called after operation completes successfully
    /// - `onError(opName, callback)` - Called when operation throws an error
    pub hooks_enabled: bool,

    /// Whether custom handler registration is enabled
    ///
    /// When true, generates:
    /// - `registerHandler(name, handler)` - Register a named handler
    /// - `invokeHandler(name, ...args)` - Invoke a registered handler
    /// - `listHandlers()` - List all registered handler names
    /// - `removeHandler(name)` - Remove a registered handler
    pub handlers_enabled: bool,

    /// Specific ops that support hooks (empty = all ops are hookable)
    ///
    /// If empty, all ops in the module will have hook support.
    /// If specified, only the listed ops will be hookable.
    pub hookable_ops: Vec<String>,

    /// Configuration options exposed to users
    ///
    /// These generate the `extend(config)` and `getConfig()` APIs.
    pub config_options: Vec<ConfigOption>,
}

impl ExtensibilityConfig {
    /// Create a new extensibility config with hooks and handlers enabled
    pub fn new() -> Self {
        Self {
            hooks_enabled: true,
            handlers_enabled: true,
            hookable_ops: Vec::new(),
            config_options: Vec::new(),
        }
    }

    /// Create a hooks-only config
    pub fn hooks_only() -> Self {
        Self {
            hooks_enabled: true,
            handlers_enabled: false,
            hookable_ops: Vec::new(),
            config_options: Vec::new(),
        }
    }

    /// Create a handlers-only config
    pub fn handlers_only() -> Self {
        Self {
            hooks_enabled: false,
            handlers_enabled: true,
            hookable_ops: Vec::new(),
            config_options: Vec::new(),
        }
    }

    /// Check if any extensibility features are enabled
    pub fn is_enabled(&self) -> bool {
        self.hooks_enabled || self.handlers_enabled || !self.config_options.is_empty()
    }

    /// Set hookable ops (only these ops will support hooks)
    pub fn with_hookable_ops(mut self, ops: Vec<String>) -> Self {
        self.hookable_ops = ops;
        self
    }

    /// Add a configuration option
    pub fn with_config_option(mut self, option: ConfigOption) -> Self {
        self.config_options.push(option);
        self
    }

    /// Check if a specific op is hookable
    pub fn is_op_hookable(&self, op_name: &str) -> bool {
        if !self.hooks_enabled {
            return false;
        }
        // If hookable_ops is empty, all ops are hookable
        if self.hookable_ops.is_empty() {
            return true;
        }
        self.hookable_ops.iter().any(|op| op == op_name)
    }
}

/// A configuration option exposed to extension users
///
/// Configuration options allow users to customize extension behavior
/// through the `extend(config)` API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigOption {
    /// Name of the configuration option (camelCase)
    pub name: String,

    /// TypeScript type for this option
    pub ts_type: WeldType,

    /// Default value as a TypeScript literal (e.g., "true", "\"default\"", "null")
    pub default_value: Option<String>,

    /// Documentation for this option
    pub doc: Option<String>,

    /// Whether this option is required (no default value)
    pub required: bool,
}

impl ConfigOption {
    /// Create a new configuration option
    pub fn new(name: impl Into<String>, ts_type: WeldType) -> Self {
        Self {
            name: name.into(),
            ts_type,
            default_value: None,
            doc: None,
            required: false,
        }
    }

    /// Set the default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self.default_value = None;
        self
    }

    /// Create a boolean option with default false
    pub fn bool_option(name: impl Into<String>) -> Self {
        Self::new(name, WeldType::bool()).with_default("false")
    }

    /// Create a string option
    pub fn string_option(name: impl Into<String>) -> Self {
        Self::new(name, WeldType::string())
    }

    /// Create a number option
    pub fn number_option(name: impl Into<String>) -> Self {
        Self::new(name, WeldType::Primitive(crate::ir::WeldPrimitive::U32))
    }
}

/// Type of lifecycle hook
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    /// Called before the operation executes
    Before,
    /// Called after the operation completes successfully
    After,
    /// Called when the operation throws an error
    Error,
}

impl HookType {
    /// Get the TypeScript function name for this hook type
    pub fn ts_function_name(&self) -> &'static str {
        match self {
            HookType::Before => "onBefore",
            HookType::After => "onAfter",
            HookType::Error => "onError",
        }
    }

    /// Get all hook types
    pub fn all() -> [HookType; 3] {
        [HookType::Before, HookType::After, HookType::Error]
    }
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::Before => write!(f, "before"),
            HookType::After => write!(f, "after"),
            HookType::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extensibility_config_default() {
        let config = ExtensibilityConfig::default();
        assert!(!config.hooks_enabled);
        assert!(!config.handlers_enabled);
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_extensibility_config_new() {
        let config = ExtensibilityConfig::new();
        assert!(config.hooks_enabled);
        assert!(config.handlers_enabled);
        assert!(config.is_enabled());
    }

    #[test]
    fn test_hookable_ops() {
        let config = ExtensibilityConfig::new();
        // All ops hookable by default
        assert!(config.is_op_hookable("op_fs_read"));
        assert!(config.is_op_hookable("op_any_operation"));

        // Restrict to specific ops
        let config = config.with_hookable_ops(vec!["op_fs_read".to_string()]);
        assert!(config.is_op_hookable("op_fs_read"));
        assert!(!config.is_op_hookable("op_fs_write"));
    }

    #[test]
    fn test_config_option() {
        let option = ConfigOption::bool_option("enableCache").with_doc("Enable caching of results");

        assert_eq!(option.name, "enableCache");
        assert_eq!(option.default_value, Some("false".to_string()));
        assert!(option.doc.is_some());
        assert!(!option.required);
    }

    #[test]
    fn test_hook_type() {
        assert_eq!(HookType::Before.ts_function_name(), "onBefore");
        assert_eq!(HookType::After.ts_function_name(), "onAfter");
        assert_eq!(HookType::Error.ts_function_name(), "onError");
    }
}
