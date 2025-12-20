// forge:weld module - TypeScript wrapper for code generation ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_weld_info(): ExtensionInfo;
      op_weld_transpile(source: string, options?: TranspileOptions): TranspileResult;
      op_weld_generate_dts(types: TypeDefinition[]): string;
      op_weld_json_to_interface(name: string, jsonSchema: string): string;
      op_weld_validate_ts(source: string): ValidationResult;
      op_weld_register_module(definition: RuntimeModuleDefinition): void;
      op_weld_list_modules(): string[];
      op_weld_generate_module_ts(specifier: string): string;
      op_weld_generate_module_dts(specifier: string): string;
      op_weld_generate_module(specifier: string): GeneratedCode;
      op_weld_generate_from_definition(definition: RuntimeModuleDefinition): GeneratedCode;
      // Weld + Etcher integration ops
      op_weld_generate_docs(config: WeldDocGenConfig): DocGenResult;
      op_weld_register_and_document(config: RegisterAndDocumentConfig): DocGenResult;
      op_weld_generate_sdk_with_docs(config: SdkDocsConfig): SdkDocsResult;
    };
  };
};

// ============================================================================
// Types
// ============================================================================

export interface ExtensionInfo {
  name: string;
  version: string;
  capabilities: string[];
}

export interface TranspileOptions {
  /** Source file name (for error messages) */
  filename?: string;
  /** Whether to include source maps */
  sourceMap?: boolean;
  /** Whether to minify output */
  minify?: boolean;
}

export interface TranspileResult {
  /** Transpiled JavaScript code */
  code: string;
  /** Source map (if requested) */
  sourceMap?: string;
}

export interface TypeDefinition {
  /** Type name */
  name: string;
  /** TypeScript type definition */
  definition: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

export interface GeneratedCode {
  /** Generated TypeScript/JavaScript code */
  code: string;
  /** Generated .d.ts declarations */
  dts: string;
}

// ============================================================================
// Weld + Etcher Integration Types
// ============================================================================

/** Configuration for documentation generation from a registered module */
export interface WeldDocGenConfig {
  /** Module specifier (e.g., "runtime:fs") */
  specifier: string;
  /** Output directory for generated documentation */
  output_dir: string;
  /** Generate Astro markdown (default: true) */
  generate_astro?: boolean;
  /** Generate HTML (default: false) */
  generate_html?: boolean;
  /** Documentation title */
  title?: string;
  /** Documentation description */
  description?: string;
}

/** Configuration for registering a module and generating docs in one call */
export interface RegisterAndDocumentConfig {
  /** Module definition to register */
  definition: RuntimeModuleDefinition;
  /** Output directory for generated documentation */
  docs_output_dir: string;
  /** Generate Astro markdown (default: true) */
  generate_astro?: boolean;
  /** Generate HTML (default: false) */
  generate_html?: boolean;
}

/** Configuration for generating SDK code and documentation together */
export interface SdkDocsConfig {
  /** Module specifier (e.g., "runtime:fs") */
  specifier: string;
  /** Output directory for SDK files (leave empty to skip writing) */
  sdk_output_dir: string;
  /** Output directory for documentation */
  docs_output_dir: string;
  /** Generate Astro markdown (default: true) */
  generate_astro?: boolean;
  /** Generate HTML (default: false) */
  generate_html?: boolean;
  /** Documentation title */
  title?: string;
  /** Documentation description */
  description?: string;
}

/** Result of documentation generation */
export interface DocGenResult {
  /** Number of symbols documented */
  symbol_count: number;
  /** Output directory path */
  output_dir: string;
  /** Generated Astro files */
  astro_files: string[];
  /** Generated HTML files */
  html_files: string[];
}

/** Result of SDK and documentation generation combined */
export interface SdkDocsResult {
  /** Generated TypeScript code */
  code: string;
  /** Generated .d.ts declarations */
  dts: string;
  /** Number of symbols documented */
  symbol_count: number;
  /** Generated Astro files */
  astro_files: string[];
  /** Generated HTML files */
  html_files: string[];
}

// ============================================================================
// Module Definition Types
// ============================================================================

export interface RuntimeModuleDefinition {
  /** Module name (e.g., "my_module") */
  name: string;
  /** Module specifier (e.g., "custom:my-module") */
  specifier: string;
  /** Documentation for the module */
  doc?: string;
  /** Struct definitions */
  structs: RuntimeStructDefinition[];
  /** Enum definitions */
  enums: RuntimeEnumDefinition[];
  /** Op/function definitions */
  ops: RuntimeOpDefinition[];
}

export interface RuntimeStructDefinition {
  name: string;
  tsName?: string;
  doc?: string;
  fields: RuntimeFieldDefinition[];
}

export interface RuntimeFieldDefinition {
  name: string;
  tsName?: string;
  tsType: string;
  doc?: string;
  optional?: boolean;
  readonly?: boolean;
}

export interface RuntimeEnumDefinition {
  name: string;
  tsName?: string;
  doc?: string;
  variants: RuntimeVariantDefinition[];
}

export interface RuntimeVariantDefinition {
  name: string;
  value?: string;
  doc?: string;
  dataType?: string;
}

export interface RuntimeOpDefinition {
  rustName: string;
  tsName?: string;
  doc?: string;
  isAsync?: boolean;
  params: RuntimeParamDefinition[];
  returnType?: string;
}

export interface RuntimeParamDefinition {
  name: string;
  tsName?: string;
  tsType: string;
  doc?: string;
  optional?: boolean;
}

// ============================================================================
// Core ops access
// ============================================================================

const core = Deno.core;

// ============================================================================
// Functions
// ============================================================================

/**
 * Get extension information
 */
export function info(): ExtensionInfo {
  return core.ops.op_weld_info();
}

/**
 * Transpile TypeScript to JavaScript
 * @param source - TypeScript source code
 * @param options - Transpilation options
 * @returns Transpiled JavaScript code and optional source map
 */
export function transpile(source: string, options?: TranspileOptions): TranspileResult {
  return core.ops.op_weld_transpile(source, options);
}

/**
 * Generate TypeScript type declarations from type definitions
 * @param types - Array of type definitions
 * @returns Generated .d.ts content
 */
export function generateDts(types: TypeDefinition[]): string {
  return core.ops.op_weld_generate_dts(types);
}

/**
 * Generate a TypeScript interface from a JSON schema-like definition
 * @param name - Interface name
 * @param jsonSchema - JSON object representing the schema
 * @returns Generated TypeScript interface
 */
export function jsonToInterface(name: string, jsonSchema: string): string {
  return core.ops.op_weld_json_to_interface(name, jsonSchema);
}

/**
 * Validate TypeScript syntax
 * @param source - TypeScript source code
 * @returns Validation result with any errors
 */
export function validateTs(source: string): ValidationResult {
  return core.ops.op_weld_validate_ts(source);
}

/**
 * Register a module definition for code generation
 * @param definition - Module definition
 */
export function registerModule(definition: RuntimeModuleDefinition): void {
  core.ops.op_weld_register_module(definition);
}

/**
 * List all registered modules
 * @returns Array of module specifiers
 */
export function listModules(): string[] {
  return core.ops.op_weld_list_modules();
}

/**
 * Generate TypeScript code for a registered module
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @returns Generated TypeScript code
 */
export function generateModuleTs(specifier: string): string {
  return core.ops.op_weld_generate_module_ts(specifier);
}

/**
 * Generate .d.ts declarations for a registered module
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @returns Generated .d.ts content
 */
export function generateModuleDts(specifier: string): string {
  return core.ops.op_weld_generate_module_dts(specifier);
}

/**
 * Generate both TypeScript and .d.ts for a registered module
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @returns Object containing both code and dts
 */
export function generateModule(specifier: string): GeneratedCode {
  return core.ops.op_weld_generate_module(specifier);
}

/**
 * Generate TypeScript code from an inline module definition (without registering)
 * @param definition - Module definition
 * @returns Object containing both code and dts
 */
export function generateFromDefinition(definition: RuntimeModuleDefinition): GeneratedCode {
  return core.ops.op_weld_generate_from_definition(definition);
}

// ============================================================================
// Weld + Etcher Integration Functions
// ============================================================================

/**
 * Generate documentation for a registered module using forge-etch
 * @param config - Documentation generation configuration
 * @returns Generated documentation result
 */
export function generateDocs(config: WeldDocGenConfig): DocGenResult {
  return core.ops.op_weld_generate_docs(config);
}

/**
 * Register a module and generate documentation in one call
 * @param config - Configuration with module definition and docs output settings
 * @returns Generated documentation result
 */
export function registerAndDocument(config: RegisterAndDocumentConfig): DocGenResult {
  return core.ops.op_weld_register_and_document(config);
}

/**
 * Generate SDK code and documentation together
 * @param config - Configuration for combined SDK and docs generation
 * @returns Result containing both SDK code and documentation info
 */
export function generateSdkWithDocs(config: SdkDocsConfig): SdkDocsResult {
  return core.ops.op_weld_generate_sdk_with_docs(config);
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  transpile: { args: []; result: void };
  generateDts: { args: []; result: void };
  jsonToInterface: { args: []; result: void };
  validateTs: { args: []; result: void };
  registerModule: { args: []; result: void };
  listModules: { args: []; result: void };
  generateModuleTs: { args: []; result: void };
  generateModuleDts: { args: []; result: void };
  generateModule: { args: []; result: void };
  generateFromDefinition: { args: []; result: void };
  generateDocs: { args: []; result: void };
  registerAndDocument: { args: []; result: void };
  generateSdkWithDocs: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "transpile" | "generateDts" | "jsonToInterface" | "validateTs" | "registerModule" | "listModules" | "generateModuleTs" | "generateModuleDts" | "generateModule" | "generateFromDefinition" | "generateDocs" | "registerAndDocument" | "generateSdkWithDocs";

/** Hook callback types */
type BeforeHookCallback<T extends OpName> = (args: OpArgs<T>) => void | Promise<void>;
type AfterHookCallback<T extends OpName> = (result: OpResult<T>, args: OpArgs<T>) => void | Promise<void>;
type ErrorHookCallback<T extends OpName> = (error: Error, args: OpArgs<T>) => void | Promise<void>;

/** Internal hook storage */
const _hooks = {
  before: new Map<OpName, Set<BeforeHookCallback<OpName>>>(),
  after: new Map<OpName, Set<AfterHookCallback<OpName>>>(),
  error: new Map<OpName, Set<ErrorHookCallback<OpName>>>(),
};

/**
 * Register a callback to be called before an operation executes.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the operation arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onBefore<T extends OpName>(
  opName: T,
  callback: BeforeHookCallback<T>
): () => void {
  if (!_hooks.before.has(opName)) {
    _hooks.before.set(opName, new Set());
  }
  _hooks.before.get(opName)!.add(callback as BeforeHookCallback<OpName>);
  return () => _hooks.before.get(opName)?.delete(callback as BeforeHookCallback<OpName>);
}

/**
 * Register a callback to be called after an operation completes successfully.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the result and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onAfter<T extends OpName>(
  opName: T,
  callback: AfterHookCallback<T>
): () => void {
  if (!_hooks.after.has(opName)) {
    _hooks.after.set(opName, new Set());
  }
  _hooks.after.get(opName)!.add(callback as AfterHookCallback<OpName>);
  return () => _hooks.after.get(opName)?.delete(callback as AfterHookCallback<OpName>);
}

/**
 * Register a callback to be called when an operation throws an error.
 * @param opName - The name of the operation to hook
 * @param callback - Function called with the error and original arguments
 * @returns Unsubscribe function to remove the hook
 */
export function onError<T extends OpName>(
  opName: T,
  callback: ErrorHookCallback<T>
): () => void {
  if (!_hooks.error.has(opName)) {
    _hooks.error.set(opName, new Set());
  }
  _hooks.error.get(opName)!.add(callback as ErrorHookCallback<OpName>);
  return () => _hooks.error.get(opName)?.delete(callback as ErrorHookCallback<OpName>);
}

/** Internal: Invoke before hooks for an operation */
async function _invokeBeforeHooks<T extends OpName>(opName: T, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.before.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(args);
    }
  }
}

/** Internal: Invoke after hooks for an operation */
async function _invokeAfterHooks<T extends OpName>(opName: T, result: OpResult<T>, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.after.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(result, args);
    }
  }
}

/** Internal: Invoke error hooks for an operation */
async function _invokeErrorHooks<T extends OpName>(opName: T, error: Error, args: OpArgs<T>): Promise<void> {
  const hooks = _hooks.error.get(opName);
  if (hooks) {
    for (const hook of hooks) {
      await hook(error, args);
    }
  }
}

/**
 * Remove all hooks for a specific operation or all operations.
 * @param opName - Optional: specific operation to clear hooks for
 */
export function removeAllHooks(opName?: OpName): void {
  if (opName) {
    _hooks.before.delete(opName);
    _hooks.after.delete(opName);
    _hooks.error.delete(opName);
  } else {
    _hooks.before.clear();
    _hooks.after.clear();
    _hooks.error.clear();
  }
}

/** Handler function type */
type HandlerFn = (...args: unknown[]) => unknown | Promise<unknown>;

/** Internal handler storage */
const _handlers = new Map<string, HandlerFn>();

/**
 * Register a custom handler that can be invoked by name.
 * @param name - Unique name for the handler
 * @param handler - Handler function to register
 * @throws Error if a handler with the same name already exists
 */
export function registerHandler(name: string, handler: HandlerFn): void {
  if (_handlers.has(name)) {
    throw new Error(`Handler '${name}' already registered`);
  }
  _handlers.set(name, handler);
}

/**
 * Invoke a registered handler by name.
 * @param name - Name of the handler to invoke
 * @param args - Arguments to pass to the handler
 * @returns The handler's return value
 * @throws Error if no handler with the given name exists
 */
export async function invokeHandler(name: string, ...args: unknown[]): Promise<unknown> {
  const handler = _handlers.get(name);
  if (!handler) {
    throw new Error(`Handler '${name}' not found`);
  }
  return await handler(...args);
}

/**
 * List all registered handler names.
 * @returns Array of handler names
 */
export function listHandlers(): string[] {
  return Array.from(_handlers.keys());
}

/**
 * Remove a registered handler.
 * @param name - Name of the handler to remove
 * @returns true if the handler was removed, false if it didn't exist
 */
export function removeHandler(name: string): boolean {
  return _handlers.delete(name);
}

/**
 * Check if a handler is registered.
 * @param name - Name of the handler to check
 * @returns true if the handler exists
 */
export function hasHandler(name: string): boolean {
  return _handlers.has(name);
}

