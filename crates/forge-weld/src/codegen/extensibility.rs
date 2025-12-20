//! Extensibility code generator for Forge extensions
//!
//! This module generates TypeScript code for extension extensibility features:
//! - Lifecycle hooks (onBefore, onAfter, onError)
//! - Custom handler registration
//! - Configuration APIs

use crate::ir::{HookType, OpSymbol, WeldModule};

/// Generator for TypeScript extensibility APIs
pub struct ExtensibilityGenerator<'a> {
    module: &'a WeldModule,
}

impl<'a> ExtensibilityGenerator<'a> {
    /// Create a new extensibility generator for a module
    pub fn new(module: &'a WeldModule) -> Self {
        Self { module }
    }

    /// Generate the complete extensibility code
    pub fn generate(&self) -> String {
        if !self.module.extensibility.is_enabled() {
            return String::new();
        }

        let mut output = String::new();

        output.push_str(
            "\n// ============================================================================\n",
        );
        output.push_str("// Extensibility API (auto-generated)\n");
        output.push_str(
            "// ============================================================================\n\n",
        );

        // Generate type registry for ops
        output.push_str(&self.generate_op_registry());

        // Generate hook storage and registration
        if self.module.extensibility.hooks_enabled {
            output.push_str(&self.generate_hook_storage());
            output.push_str(&self.generate_hook_functions());
        }

        // Generate handler registration
        if self.module.extensibility.handlers_enabled {
            output.push_str(&self.generate_handler_storage());
            output.push_str(&self.generate_handler_functions());
        }

        // Generate configuration API if there are config options
        if !self.module.extensibility.config_options.is_empty() {
            output.push_str(&self.generate_config_api());
        }

        output
    }

    /// Generate the OpRegistry interface for type-safe hook callbacks
    fn generate_op_registry(&self) -> String {
        let mut output = String::new();

        output.push_str("/** Registry of operations with their argument and result types */\n");
        output.push_str("interface OpRegistry {\n");

        for op in &self.module.ops {
            let args = self.generate_op_args_type(op);
            let result = self.generate_op_result_type(op);
            output.push_str(&format!(
                "  {}: {{ args: {}; result: {} }};\n",
                op.ts_name, args, result
            ));
        }

        output.push_str("}\n\n");

        // Generate helper types
        output.push_str("/** Extract argument types for an operation */\n");
        output.push_str("type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];\n\n");

        output.push_str("/** Extract result type for an operation */\n");
        output.push_str("type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];\n\n");

        // Generate op names type
        output.push_str("/** Valid operation names for this extension */\n");
        let op_names: Vec<String> = self
            .module
            .ops
            .iter()
            .map(|op| format!("\"{}\"", op.ts_name))
            .collect();
        if op_names.is_empty() {
            output.push_str("type OpName = never;\n\n");
        } else {
            output.push_str(&format!("type OpName = {};\n\n", op_names.join(" | ")));
        }

        output
    }

    /// Generate args type for an op
    fn generate_op_args_type(&self, op: &OpSymbol) -> String {
        let visible_params: Vec<_> = op.visible_params().collect();
        if visible_params.is_empty() {
            return "[]".to_string();
        }

        let params: Vec<String> = visible_params
            .iter()
            .map(|p| {
                let optional = if p.optional { "?" } else { "" };
                format!("{}{}: {}", p.ts_name, optional, p.ty.to_typescript())
            })
            .collect();

        format!("[{}]", params.join(", "))
    }

    /// Generate result type for an op
    fn generate_op_result_type(&self, op: &OpSymbol) -> String {
        // Strip Promise wrapper from return type for the result type
        let return_type = op.ts_return_type();
        if return_type.starts_with("Promise<") && return_type.ends_with('>') {
            return_type[8..return_type.len() - 1].to_string()
        } else {
            return_type
        }
    }

    /// Generate hook storage
    fn generate_hook_storage(&self) -> String {
        let mut output = String::new();

        output.push_str("/** Hook callback types */\n");
        output.push_str("type BeforeHookCallback<T extends OpName> = (args: OpArgs<T>) => void | Promise<void>;\n");
        output.push_str("type AfterHookCallback<T extends OpName> = (result: OpResult<T>, args: OpArgs<T>) => void | Promise<void>;\n");
        output.push_str("type ErrorHookCallback<T extends OpName> = (error: Error, args: OpArgs<T>) => void | Promise<void>;\n\n");

        output.push_str("/** Internal hook storage */\n");
        output.push_str("const _hooks = {\n");
        output.push_str("  before: new Map<OpName, Set<BeforeHookCallback<OpName>>>(),\n");
        output.push_str("  after: new Map<OpName, Set<AfterHookCallback<OpName>>>(),\n");
        output.push_str("  error: new Map<OpName, Set<ErrorHookCallback<OpName>>>(),\n");
        output.push_str("};\n\n");

        output
    }

    /// Generate hook registration functions
    fn generate_hook_functions(&self) -> String {
        let mut output = String::new();

        // onBefore
        output.push_str("/**\n");
        output.push_str(" * Register a callback to be called before an operation executes.\n");
        output.push_str(" * @param opName - The name of the operation to hook\n");
        output.push_str(" * @param callback - Function called with the operation arguments\n");
        output.push_str(" * @returns Unsubscribe function to remove the hook\n");
        output.push_str(" */\n");
        output.push_str("export function onBefore<T extends OpName>(\n");
        output.push_str("  opName: T,\n");
        output.push_str("  callback: BeforeHookCallback<T>\n");
        output.push_str("): () => void {\n");
        output.push_str("  if (!_hooks.before.has(opName)) {\n");
        output.push_str("    _hooks.before.set(opName, new Set());\n");
        output.push_str("  }\n");
        output.push_str(
            "  _hooks.before.get(opName)!.add(callback as BeforeHookCallback<OpName>);\n",
        );
        output.push_str("  return () => _hooks.before.get(opName)?.delete(callback as BeforeHookCallback<OpName>);\n");
        output.push_str("}\n\n");

        // onAfter
        output.push_str("/**\n");
        output.push_str(
            " * Register a callback to be called after an operation completes successfully.\n",
        );
        output.push_str(" * @param opName - The name of the operation to hook\n");
        output.push_str(
            " * @param callback - Function called with the result and original arguments\n",
        );
        output.push_str(" * @returns Unsubscribe function to remove the hook\n");
        output.push_str(" */\n");
        output.push_str("export function onAfter<T extends OpName>(\n");
        output.push_str("  opName: T,\n");
        output.push_str("  callback: AfterHookCallback<T>\n");
        output.push_str("): () => void {\n");
        output.push_str("  if (!_hooks.after.has(opName)) {\n");
        output.push_str("    _hooks.after.set(opName, new Set());\n");
        output.push_str("  }\n");
        output
            .push_str("  _hooks.after.get(opName)!.add(callback as AfterHookCallback<OpName>);\n");
        output.push_str("  return () => _hooks.after.get(opName)?.delete(callback as AfterHookCallback<OpName>);\n");
        output.push_str("}\n\n");

        // onError
        output.push_str("/**\n");
        output.push_str(" * Register a callback to be called when an operation throws an error.\n");
        output.push_str(" * @param opName - The name of the operation to hook\n");
        output.push_str(
            " * @param callback - Function called with the error and original arguments\n",
        );
        output.push_str(" * @returns Unsubscribe function to remove the hook\n");
        output.push_str(" */\n");
        output.push_str("export function onError<T extends OpName>(\n");
        output.push_str("  opName: T,\n");
        output.push_str("  callback: ErrorHookCallback<T>\n");
        output.push_str("): () => void {\n");
        output.push_str("  if (!_hooks.error.has(opName)) {\n");
        output.push_str("    _hooks.error.set(opName, new Set());\n");
        output.push_str("  }\n");
        output
            .push_str("  _hooks.error.get(opName)!.add(callback as ErrorHookCallback<OpName>);\n");
        output.push_str("  return () => _hooks.error.get(opName)?.delete(callback as ErrorHookCallback<OpName>);\n");
        output.push_str("}\n\n");

        // Internal helper to invoke hooks
        output.push_str("/** Internal: Invoke before hooks for an operation */\n");
        output.push_str("async function _invokeBeforeHooks<T extends OpName>(opName: T, args: OpArgs<T>): Promise<void> {\n");
        output.push_str("  const hooks = _hooks.before.get(opName);\n");
        output.push_str("  if (hooks) {\n");
        output.push_str("    for (const hook of hooks) {\n");
        output.push_str("      await hook(args);\n");
        output.push_str("    }\n");
        output.push_str("  }\n");
        output.push_str("}\n\n");

        output.push_str("/** Internal: Invoke after hooks for an operation */\n");
        output.push_str("async function _invokeAfterHooks<T extends OpName>(opName: T, result: OpResult<T>, args: OpArgs<T>): Promise<void> {\n");
        output.push_str("  const hooks = _hooks.after.get(opName);\n");
        output.push_str("  if (hooks) {\n");
        output.push_str("    for (const hook of hooks) {\n");
        output.push_str("      await hook(result, args);\n");
        output.push_str("    }\n");
        output.push_str("  }\n");
        output.push_str("}\n\n");

        output.push_str("/** Internal: Invoke error hooks for an operation */\n");
        output.push_str("async function _invokeErrorHooks<T extends OpName>(opName: T, error: Error, args: OpArgs<T>): Promise<void> {\n");
        output.push_str("  const hooks = _hooks.error.get(opName);\n");
        output.push_str("  if (hooks) {\n");
        output.push_str("    for (const hook of hooks) {\n");
        output.push_str("      await hook(error, args);\n");
        output.push_str("    }\n");
        output.push_str("  }\n");
        output.push_str("}\n\n");

        // removeAllHooks helper
        output.push_str("/**\n");
        output.push_str(" * Remove all hooks for a specific operation or all operations.\n");
        output.push_str(" * @param opName - Optional: specific operation to clear hooks for\n");
        output.push_str(" */\n");
        output.push_str("export function removeAllHooks(opName?: OpName): void {\n");
        output.push_str("  if (opName) {\n");
        output.push_str("    _hooks.before.delete(opName);\n");
        output.push_str("    _hooks.after.delete(opName);\n");
        output.push_str("    _hooks.error.delete(opName);\n");
        output.push_str("  } else {\n");
        output.push_str("    _hooks.before.clear();\n");
        output.push_str("    _hooks.after.clear();\n");
        output.push_str("    _hooks.error.clear();\n");
        output.push_str("  }\n");
        output.push_str("}\n\n");

        output
    }

    /// Generate handler storage
    fn generate_handler_storage(&self) -> String {
        let mut output = String::new();

        output.push_str("/** Handler function type */\n");
        output.push_str("type HandlerFn = (...args: unknown[]) => unknown | Promise<unknown>;\n\n");

        output.push_str("/** Internal handler storage */\n");
        output.push_str("const _handlers = new Map<string, HandlerFn>();\n\n");

        output
    }

    /// Generate handler registration functions
    fn generate_handler_functions(&self) -> String {
        let mut output = String::new();

        // registerHandler
        output.push_str("/**\n");
        output.push_str(" * Register a custom handler that can be invoked by name.\n");
        output.push_str(" * @param name - Unique name for the handler\n");
        output.push_str(" * @param handler - Handler function to register\n");
        output.push_str(" * @throws Error if a handler with the same name already exists\n");
        output.push_str(" */\n");
        output.push_str(
            "export function registerHandler(name: string, handler: HandlerFn): void {\n",
        );
        output.push_str("  if (_handlers.has(name)) {\n");
        output.push_str("    throw new Error(`Handler '${name}' already registered`);\n");
        output.push_str("  }\n");
        output.push_str("  _handlers.set(name, handler);\n");
        output.push_str("}\n\n");

        // invokeHandler
        output.push_str("/**\n");
        output.push_str(" * Invoke a registered handler by name.\n");
        output.push_str(" * @param name - Name of the handler to invoke\n");
        output.push_str(" * @param args - Arguments to pass to the handler\n");
        output.push_str(" * @returns The handler's return value\n");
        output.push_str(" * @throws Error if no handler with the given name exists\n");
        output.push_str(" */\n");
        output.push_str("export async function invokeHandler(name: string, ...args: unknown[]): Promise<unknown> {\n");
        output.push_str("  const handler = _handlers.get(name);\n");
        output.push_str("  if (!handler) {\n");
        output.push_str("    throw new Error(`Handler '${name}' not found`);\n");
        output.push_str("  }\n");
        output.push_str("  return await handler(...args);\n");
        output.push_str("}\n\n");

        // listHandlers
        output.push_str("/**\n");
        output.push_str(" * List all registered handler names.\n");
        output.push_str(" * @returns Array of handler names\n");
        output.push_str(" */\n");
        output.push_str("export function listHandlers(): string[] {\n");
        output.push_str("  return Array.from(_handlers.keys());\n");
        output.push_str("}\n\n");

        // removeHandler
        output.push_str("/**\n");
        output.push_str(" * Remove a registered handler.\n");
        output.push_str(" * @param name - Name of the handler to remove\n");
        output.push_str(" * @returns true if the handler was removed, false if it didn't exist\n");
        output.push_str(" */\n");
        output.push_str("export function removeHandler(name: string): boolean {\n");
        output.push_str("  return _handlers.delete(name);\n");
        output.push_str("}\n\n");

        // hasHandler
        output.push_str("/**\n");
        output.push_str(" * Check if a handler is registered.\n");
        output.push_str(" * @param name - Name of the handler to check\n");
        output.push_str(" * @returns true if the handler exists\n");
        output.push_str(" */\n");
        output.push_str("export function hasHandler(name: string): boolean {\n");
        output.push_str("  return _handlers.has(name);\n");
        output.push_str("}\n\n");

        output
    }

    /// Generate configuration API
    fn generate_config_api(&self) -> String {
        let mut output = String::new();

        // Generate config interface
        output.push_str("/** Extension configuration options */\n");
        output.push_str("export interface ExtensionConfig {\n");
        for option in &self.module.extensibility.config_options {
            if let Some(ref doc) = option.doc {
                output.push_str(&format!("  /** {} */\n", doc));
            }
            let optional = if option.required { "" } else { "?" };
            output.push_str(&format!(
                "  {}{}: {};\n",
                option.name,
                optional,
                option.ts_type.to_typescript()
            ));
        }
        output.push_str("}\n\n");

        // Generate default config
        output.push_str("/** Internal config storage */\n");
        output.push_str("let _config: ExtensionConfig = {\n");
        for option in &self.module.extensibility.config_options {
            if let Some(ref default) = option.default_value {
                output.push_str(&format!("  {}: {},\n", option.name, default));
            }
        }
        output.push_str("};\n\n");

        // extend function
        output.push_str("/**\n");
        output.push_str(" * Extend the extension configuration with custom options.\n");
        output.push_str(" * @param config - Partial configuration to merge\n");
        output.push_str(" */\n");
        output.push_str("export function extend(config: Partial<ExtensionConfig>): void {\n");
        output.push_str("  _config = { ..._config, ...config };\n");
        output.push_str("}\n\n");

        // getConfig function
        output.push_str("/**\n");
        output.push_str(" * Get the current extension configuration.\n");
        output.push_str(" * @returns Current configuration object\n");
        output.push_str(" */\n");
        output.push_str("export function getConfig(): ExtensionConfig {\n");
        output.push_str("  return { ..._config };\n");
        output.push_str("}\n\n");

        output
    }

    /// Get the TypeScript function name for a hook type
    pub fn hook_function_name(hook_type: HookType) -> &'static str {
        hook_type.ts_function_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OpParam, WeldType};

    fn create_test_module() -> WeldModule {
        WeldModule::host("test")
            .op(crate::ir::OpSymbol::from_rust_name("op_test_read")
                .ts_name("read")
                .async_op()
                .param(OpParam::new("path", WeldType::string()))
                .returns(WeldType::result(
                    WeldType::string(),
                    WeldType::struct_ref("TestError"),
                )))
            .op(crate::ir::OpSymbol::from_rust_name("op_test_write")
                .ts_name("write")
                .async_op()
                .param(OpParam::new("path", WeldType::string()))
                .param(OpParam::new("content", WeldType::string()))
                .returns(WeldType::result(
                    WeldType::void(),
                    WeldType::struct_ref("TestError"),
                )))
            .with_extensibility()
    }

    #[test]
    fn test_generate_with_extensibility() {
        let module = create_test_module();
        let gen = ExtensibilityGenerator::new(&module);
        let output = gen.generate();

        // Should contain hook functions
        assert!(output.contains("export function onBefore"));
        assert!(output.contains("export function onAfter"));
        assert!(output.contains("export function onError"));

        // Should contain handler functions
        assert!(output.contains("export function registerHandler"));
        assert!(output.contains("export async function invokeHandler"));
        assert!(output.contains("export function listHandlers"));

        // Should contain OpRegistry
        assert!(output.contains("interface OpRegistry"));
        assert!(output.contains("read:"));
        assert!(output.contains("write:"));
    }

    #[test]
    fn test_generate_without_extensibility() {
        let module = WeldModule::host("test");
        let gen = ExtensibilityGenerator::new(&module);
        let output = gen.generate();

        // Should be empty
        assert!(output.is_empty());
    }

    #[test]
    fn test_op_args_type() {
        let module = create_test_module();
        let gen = ExtensibilityGenerator::new(&module);

        let read_op = &module.ops[0];
        let args = gen.generate_op_args_type(read_op);
        assert_eq!(args, "[path: string]");

        let write_op = &module.ops[1];
        let args = gen.generate_op_args_type(write_op);
        assert_eq!(args, "[path: string, content: string]");
    }
}
