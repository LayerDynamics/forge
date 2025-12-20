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
