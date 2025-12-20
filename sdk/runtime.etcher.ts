// forge:etcher module - TypeScript wrapper for documentation generation ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_etcher_info(): ExtensionInfo;
      op_etcher_generate_docs(config: DocGenConfig): Promise<DocGenResult>;
      op_etcher_parse_ts(sourcePath: string): Promise<ParseResult>;
      op_etcher_parse_rust(sourcePath: string): Promise<ParseResult>;
      op_etcher_merge_nodes(
        name: string,
        specifier: string,
        tsSource: string | null,
        rustSource: string | null
      ): Promise<ParseResult>;
      op_etcher_nodes_to_astro(
        name: string,
        specifier: string,
        crateRoot: string,
        outputDir: string
      ): Promise<DocGenResult>;
      op_etcher_nodes_to_html(
        name: string,
        specifier: string,
        crateRoot: string,
        outputDir: string
      ): Promise<DocGenResult>;
      // WeldModule-based documentation
      op_etcher_from_weld_module(config: WeldModuleDocConfig): Promise<DocGenResult>;
      // Site update/regeneration ops
      op_etcher_update_site(config: SiteUpdateConfig): Promise<SiteUpdateResult>;
      op_etcher_regenerate_site(config: SiteRegenConfig): Promise<SiteUpdateResult>;
      op_etcher_generate_site_index(config: SiteIndexConfig): Promise<string>;
      op_etcher_validate_config(projectDir: string): Promise<AstroConfigValidation>;
      op_etcher_validate_output_dir(config: OutputDirValidationConfig): Promise<ValidationCheck>;
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

/** Configuration for documentation generation */
export interface DocGenConfig {
  /** Extension/module name */
  name: string;
  /** Module specifier (e.g., "runtime:fs") */
  specifier: string;
  /** Path to TypeScript source file */
  ts_source?: string;
  /** Path to Rust source file */
  rust_source?: string;
  /** Output directory */
  output_dir: string;
  /** Generate Astro markdown */
  generate_astro?: boolean;
  /** Generate HTML */
  generate_html?: boolean;
  /** Documentation title */
  title?: string;
  /** Documentation description */
  description?: string;
  /** Include private symbols */
  include_private?: boolean;
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

/** Information about a documentation node */
export interface DocNodeInfo {
  /** Symbol name */
  name: string;
  /** Node kind as string */
  kind: string;
  /** Module specifier (if any) */
  module?: string;
  /** Description from JSDoc/doc comments */
  description?: string;
  /** TypeScript signature (if applicable) */
  signature?: string;
  /** Whether this is a default export */
  is_default: boolean;
  /** Visibility (public, private, internal) */
  visibility: string;
}

/** Result of TypeScript parsing */
export interface ParseResult {
  /** Number of nodes parsed */
  node_count: number;
  /** Information about parsed nodes */
  nodes: DocNodeInfo[];
}

// ============================================================================
// WeldModule-based Documentation Types
// ============================================================================

/** Configuration for generating documentation from WeldModule data */
export interface WeldModuleDocConfig {
  /** Module name (e.g., "host_fs") */
  name: string;
  /** Module specifier (e.g., "runtime:fs") */
  specifier: string;
  /** Module documentation */
  doc?: string;
  /** Output directory */
  output_dir: string;
  /** Generate Astro markdown */
  generate_astro?: boolean;
  /** Generate HTML */
  generate_html?: boolean;
  /** Documentation title */
  title?: string;
  /** Documentation description */
  description?: string;
  /** Struct definitions as JSON */
  structs_json?: string;
  /** Enum definitions as JSON */
  enums_json?: string;
  /** Op definitions as JSON */
  ops_json?: string;
}

// ============================================================================
// Site Update/Regeneration Types
// ============================================================================

/** Configuration for site update */
export interface SiteUpdateConfig {
  /** Output directory for the site */
  output_dir: string;
  /** Whether to clean the output directory first */
  clean_first?: boolean;
  /** Extension documentation definitions as JSON array */
  docs_json?: string;
}

/** Result of site update */
export interface SiteUpdateResult {
  /** Number of files generated */
  generated_count: number;
  /** Paths to generated files */
  generated_files: string[];
  /** Number of files removed (when cleaning) */
  removed_count: number;
  /** Paths to removed files */
  removed_files: string[];
  /** Total number of symbols documented */
  total_symbols: number;
  /** Number of modules processed */
  module_count: number;
}

/** Configuration for site regeneration */
export interface SiteRegenConfig {
  /** Output directory for the site */
  output_dir: string;
  /** Extension documentation definitions as JSON array */
  docs_json?: string;
}

/** Configuration for site index generation */
export interface SiteIndexConfig {
  /** Output directory for the index */
  output_dir: string;
  /** Extension documentation definitions as JSON array */
  docs_json?: string;
}

/** Astro configuration validation result */
export interface AstroConfigValidation {
  /** Whether all validation checks passed */
  valid: boolean;
  /** Individual validation results */
  checks: ValidationCheck[];
  /** Site URL if found */
  site_url?: string;
  /** Starlight title if found */
  starlight_title?: string;
}

/** Individual validation check result */
export interface ValidationCheck {
  /** Whether this check passed */
  passed: boolean;
  /** Description of what was checked */
  description: string;
  /** Error message if failed */
  error?: string;
}

/** Configuration for output directory validation */
export interface OutputDirValidationConfig {
  /** Target directory or slug to validate */
  target: string;
  /** Project directory path (for loading astro config) */
  project_dir: string;
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
  return core.ops.op_etcher_info();
}

/**
 * Generate documentation from a configuration
 * @param config - Documentation generation configuration
 * @returns Generated documentation result with file paths
 */
export async function generateDocs(config: DocGenConfig): Promise<DocGenResult> {
  return core.ops.op_etcher_generate_docs(config);
}

/**
 * Parse TypeScript source file and extract documentation nodes
 * @param sourcePath - Path to the TypeScript source file
 * @returns Parse result with node information
 */
export async function parseTs(sourcePath: string): Promise<ParseResult> {
  return core.ops.op_etcher_parse_ts(sourcePath);
}

/**
 * Parse Rust source file for weld metadata
 * Note: Direct Rust parsing is not yet implemented. Use generateDocs with rust_source option instead.
 * @param sourcePath - Path to the Rust source file
 * @returns Parse result with node information
 */
export async function parseRust(sourcePath: string): Promise<ParseResult> {
  return core.ops.op_etcher_parse_rust(sourcePath);
}

/**
 * Merge documentation nodes from TypeScript and Rust sources
 * TSDoc/JSDoc takes precedence over Rust doc comments
 * @param name - Extension/module name
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @param tsSource - Path to TypeScript source file
 * @param rustSource - Path to Rust source file
 * @returns Merged parse result
 */
export async function mergeNodes(
  name: string,
  specifier: string,
  tsSource?: string,
  rustSource?: string
): Promise<ParseResult> {
  return core.ops.op_etcher_merge_nodes(
    name,
    specifier,
    tsSource ?? null,
    rustSource ?? null
  );
}

/**
 * Generate Astro-compatible markdown from a crate directory
 * @param name - Extension/module name
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @param crateRoot - Path to the crate root directory
 * @param outputDir - Output directory for generated files
 * @returns Generated documentation result
 */
export async function nodesToAstro(
  name: string,
  specifier: string,
  crateRoot: string,
  outputDir: string
): Promise<DocGenResult> {
  return core.ops.op_etcher_nodes_to_astro(name, specifier, crateRoot, outputDir);
}

/**
 * Generate HTML from a crate directory
 * @param name - Extension/module name
 * @param specifier - Module specifier (e.g., "runtime:fs")
 * @param crateRoot - Path to the crate root directory
 * @param outputDir - Output directory for generated files
 * @returns Generated documentation result
 */
export async function nodesToHtml(
  name: string,
  specifier: string,
  crateRoot: string,
  outputDir: string
): Promise<DocGenResult> {
  return core.ops.op_etcher_nodes_to_html(name, specifier, crateRoot, outputDir);
}

// ============================================================================
// Builder Pattern API (convenience wrapper)
// ============================================================================

/**
 * Builder for configuring and running documentation generation
 *
 * @example
 * ```typescript
 * const result = await new DocsBuilder("host_fs", "runtime:fs")
 *   .tsSource("ts/init.ts")
 *   .rustSource("src/lib.rs")
 *   .outputDir("docs/api/fs")
 *   .title("File System API")
 *   .generateAstro()
 *   .build();
 * ```
 */
export class DocsBuilder {
  private config: DocGenConfig;

  constructor(name: string, specifier: string) {
    this.config = {
      name,
      specifier,
      output_dir: "docs",
      generate_astro: true,
      generate_html: false,
    };
  }

  /** Set the TypeScript source file path */
  tsSource(path: string): DocsBuilder {
    this.config.ts_source = path;
    return this;
  }

  /** Set the Rust source file path */
  rustSource(path: string): DocsBuilder {
    this.config.rust_source = path;
    return this;
  }

  /** Set the output directory */
  outputDir(path: string): DocsBuilder {
    this.config.output_dir = path;
    return this;
  }

  /** Enable Astro markdown generation */
  generateAstro(enable = true): DocsBuilder {
    this.config.generate_astro = enable;
    return this;
  }

  /** Enable HTML generation */
  generateHtml(enable = true): DocsBuilder {
    this.config.generate_html = enable;
    return this;
  }

  /** Set the documentation title */
  title(title: string): DocsBuilder {
    this.config.title = title;
    return this;
  }

  /** Set the documentation description */
  description(desc: string): DocsBuilder {
    this.config.description = desc;
    return this;
  }

  /** Include private symbols in documentation */
  includePrivate(include = true): DocsBuilder {
    this.config.include_private = include;
    return this;
  }

  /** Build the documentation */
  async build(): Promise<DocGenResult> {
    return generateDocs(this.config);
  }
}

// ============================================================================
// WeldModule-based Documentation Functions
// ============================================================================

/**
 * Generate documentation from WeldModule data passed as JSON
 * @param config - Configuration with module data and output settings
 * @returns Generated documentation result
 */
export async function fromWeldModule(config: WeldModuleDocConfig): Promise<DocGenResult> {
  return core.ops.op_etcher_from_weld_module(config);
}

// ============================================================================
// Site Update/Regeneration Functions
// ============================================================================

/**
 * Update an Astro documentation site with new content
 * @param config - Site update configuration
 * @returns Site update result with generated files
 */
export async function updateSite(config: SiteUpdateConfig): Promise<SiteUpdateResult> {
  return core.ops.op_etcher_update_site(config);
}

/**
 * Regenerate the entire documentation site (cleans first)
 * @param config - Site regeneration configuration
 * @returns Site update result with generated and removed files
 */
export async function regenerateSite(config: SiteRegenConfig): Promise<SiteUpdateResult> {
  return core.ops.op_etcher_regenerate_site(config);
}

/**
 * Generate a site-wide index page listing all modules
 * @param config - Site index configuration
 * @returns Path to generated index file
 */
export async function generateSiteIndex(config: SiteIndexConfig): Promise<string> {
  return core.ops.op_etcher_generate_site_index(config);
}

/**
 * Validate Astro project configuration
 * @param projectDir - Path to Astro project root
 * @returns Validation result with checks and parsed config info
 */
export async function validateConfig(projectDir: string): Promise<AstroConfigValidation> {
  return core.ops.op_etcher_validate_config(projectDir);
}

/**
 * Check if an output directory is properly configured in the Starlight sidebar
 * @param config - Output directory validation configuration
 * @returns Validation check result
 */
export async function validateOutputDir(config: OutputDirValidationConfig): Promise<ValidationCheck> {
  return core.ops.op_etcher_validate_output_dir(config);
}

// ============================================================================
// Site Builder Pattern API
// ============================================================================

/**
 * Builder for updating documentation sites
 *
 * @example
 * ```typescript
 * const result = await new SiteBuilder("site/src/content/docs")
 *   .addDoc({ name: "fs", specifier: "runtime:fs", title: "File System" })
 *   .clean()
 *   .build();
 * ```
 */
export class SiteBuilder {
  private config: SiteUpdateConfig;
  private docs: Array<{ name: string; specifier: string; title: string; description?: string }> = [];

  constructor(outputDir: string) {
    this.config = {
      output_dir: outputDir,
      clean_first: false,
    };
  }

  /** Add documentation for a module */
  addDoc(doc: { name: string; specifier: string; title: string; description?: string }): SiteBuilder {
    this.docs.push(doc);
    return this;
  }

  /** Enable cleaning the output directory first */
  clean(enable = true): SiteBuilder {
    this.config.clean_first = enable;
    return this;
  }

  /** Build the site update */
  async build(): Promise<SiteUpdateResult> {
    this.config.docs_json = JSON.stringify(this.docs.map(d => ({
      name: d.name,
      specifier: d.specifier,
      title: d.title,
      description: d.description,
      nodes: [],
    })));
    return updateSite(this.config);
  }
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  generateDocs: { args: []; result: void };
  parseTs: { args: []; result: void };
  parseRust: { args: []; result: void };
  mergeNodes: { args: []; result: void };
  nodesToAstro: { args: []; result: void };
  nodesToHtml: { args: []; result: void };
  fromWeldModule: { args: []; result: void };
  updateSite: { args: []; result: void };
  regenerateSite: { args: []; result: void };
  generateSiteIndex: { args: []; result: void };
  validateConfig: { args: []; result: void };
  validateOutputDir: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "generateDocs" | "parseTs" | "parseRust" | "mergeNodes" | "nodesToAstro" | "nodesToHtml" | "fromWeldModule" | "updateSite" | "regenerateSite" | "generateSiteIndex" | "validateConfig" | "validateOutputDir";

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

