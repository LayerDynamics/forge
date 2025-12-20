// forge:bundler module - TypeScript wrapper for bundling ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_bundler_info(): ExtensionInfo;
      op_bundler_icon_create(options?: IconCreateOptions): Uint8Array;
      op_bundler_icon_validate(data: Uint8Array): IconValidation;
      op_bundler_icon_resize(data: Uint8Array, options: IconResizeOptions): Uint8Array;
      op_bundler_manifest_parse(content: string): AppManifest;
      op_bundler_sanitize_name(name: string): string;
      op_bundler_platform_info(): PlatformInfo;
      op_bundler_icon_requirements(platform: string): IconResizeOptions[];
      op_bundler_set_app_dir(path: string): void;
      op_bundler_get_app_dir(): string | null;
      op_bundler_set_build_config(config: BuildConfig): void;
      op_bundler_get_build_config(): BuildConfig | null;
      op_bundler_path_info(path: string): PathInfo;
      op_bundler_path_join(components: string[]): string;
      op_bundler_manifest_path(appDir: string): string;
      op_bundler_cache_manifest(path: string, manifest: AppManifest): void;
      op_bundler_get_cached_manifest(path: string): AppManifest | null;
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

export interface IconCreateOptions {
  /** Size in pixels (default: 1024) */
  size?: number;
  /** Primary color (hex, default: "#3C5AB8") */
  color?: string;
}

export interface IconValidation {
  width: number;
  height: number;
  isSquare: boolean;
  meetsMinimum: boolean;
  meetsRecommended: boolean;
  hasTransparency: boolean;
  warnings: string[];
  errors: string[];
}

export interface IconResizeOptions {
  /** Target width */
  width: number;
  /** Target height */
  height: number;
}

export interface AppManifest {
  name: string;
  identifier: string;
  version: string;
  icon?: string;
}

export interface PlatformInfo {
  os: string;
  arch: string;
  bundleFormat: string;
  supported: boolean;
}

export type BundleFormat =
  | "App"      // macOS .app bundle
  | "Dmg"      // macOS .dmg disk image
  | "Pkg"      // macOS .pkg installer
  | "Msix"     // Windows .msix package
  | "AppImage" // Linux AppImage
  | "Tarball"  // Compressed tarball
  | "Zip";     // ZIP archive

export interface BuildConfig {
  /** App directory path */
  appDir: string;
  /** Output directory for bundle */
  outputDir?: string;
  /** Target bundle format */
  format?: BundleFormat;
  /** Whether to sign the bundle */
  sign?: boolean;
  /** Code signing identity (platform-specific) */
  signingIdentity?: string;
}

export interface PathInfo {
  /** The full path */
  path: string;
  /** Whether the path exists */
  exists: boolean;
  /** Whether it's a directory */
  isDir: boolean;
  /** Whether it's a file */
  isFile: boolean;
  /** File extension if any */
  extension?: string;
  /** File name without path */
  fileName?: string;
  /** Parent directory */
  parent?: string;
}

// ============================================================================
// Constants
// ============================================================================

/** Minimum recommended icon size */
export const MIN_ICON_SIZE = 512;

/** Optimal icon size for best quality across all platforms */
export const RECOMMENDED_ICON_SIZE = 1024;

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
  return core.ops.op_bundler_info();
}

/**
 * Create a placeholder icon
 * @param options - Icon creation options
 * @returns PNG image bytes
 */
export function iconCreate(options?: IconCreateOptions): Uint8Array {
  return core.ops.op_bundler_icon_create(options);
}

/**
 * Validate an icon
 * @param data - PNG image bytes
 * @returns Validation result with dimensions, warnings, and errors
 */
export function iconValidate(data: Uint8Array): IconValidation {
  return core.ops.op_bundler_icon_validate(data);
}

/**
 * Resize an icon to specified dimensions
 * @param data - PNG image bytes
 * @param options - Resize options with target dimensions
 * @returns Resized PNG image bytes
 */
export function iconResize(data: Uint8Array, options: IconResizeOptions): Uint8Array {
  return core.ops.op_bundler_icon_resize(data, options);
}

/**
 * Parse a manifest.app.toml file content
 * @param content - TOML content string
 * @returns Parsed app manifest
 */
export function manifestParse(content: string): AppManifest {
  return core.ops.op_bundler_manifest_parse(content);
}

/**
 * Sanitize a name for use as executable/identifier
 * @param name - Original name
 * @returns Sanitized name (lowercase, alphanumeric with hyphens)
 */
export function sanitizeName(name: string): string {
  return core.ops.op_bundler_sanitize_name(name);
}

/**
 * Get platform-specific bundle information
 * @returns Platform info including OS, architecture, and bundle format
 */
export function platformInfo(): PlatformInfo {
  return core.ops.op_bundler_platform_info();
}

/**
 * Get icon requirements for a specific platform
 * @param platform - Platform name ("macos", "windows", or "linux")
 * @returns Array of required icon sizes
 */
export function iconRequirements(platform: string): IconResizeOptions[] {
  return core.ops.op_bundler_icon_requirements(platform);
}

/**
 * Set the current app directory for bundling operations
 * @param path - Path to app directory
 */
export function setAppDir(path: string): void {
  core.ops.op_bundler_set_app_dir(path);
}

/**
 * Get the current app directory
 * @returns App directory path or null if not set
 */
export function getAppDir(): string | null {
  return core.ops.op_bundler_get_app_dir();
}

/**
 * Set build configuration
 * @param config - Build configuration
 */
export function setBuildConfig(config: BuildConfig): void {
  core.ops.op_bundler_set_build_config(config);
}

/**
 * Get current build configuration
 * @returns Build configuration or null if not set
 */
export function getBuildConfig(): BuildConfig | null {
  return core.ops.op_bundler_get_build_config();
}

/**
 * Analyze a path and return information about it
 * @param path - Path to analyze
 * @returns Path information including existence and type
 */
export function pathInfo(path: string): PathInfo {
  return core.ops.op_bundler_path_info(path);
}

/**
 * Join path components
 * @param components - Array of path components
 * @returns Joined path string
 */
export function pathJoin(...components: string[]): string {
  return core.ops.op_bundler_path_join(components);
}

/**
 * Get the manifest path for an app directory
 * @param appDir - App directory path
 * @returns Path to manifest.app.toml
 */
export function manifestPath(appDir: string): string {
  return core.ops.op_bundler_manifest_path(appDir);
}

/**
 * Cache a parsed manifest for later retrieval
 * @param path - Manifest file path
 * @param manifest - Parsed manifest
 */
export function cacheManifest(path: string, manifest: AppManifest): void {
  core.ops.op_bundler_cache_manifest(path, manifest);
}

/**
 * Get a cached manifest
 * @param path - Manifest file path
 * @returns Cached manifest or null if not cached
 */
export function getCachedManifest(path: string): AppManifest | null {
  return core.ops.op_bundler_get_cached_manifest(path);
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  iconCreate: { args: []; result: void };
  iconValidate: { args: []; result: void };
  iconResize: { args: []; result: void };
  manifestParse: { args: []; result: void };
  sanitizeName: { args: []; result: void };
  platformInfo: { args: []; result: void };
  iconRequirements: { args: []; result: void };
  setAppDir: { args: []; result: void };
  getAppDir: { args: []; result: void };
  setBuildConfig: { args: []; result: void };
  getBuildConfig: { args: []; result: void };
  pathInfo: { args: []; result: void };
  pathJoin: { args: []; result: void };
  manifestPath: { args: []; result: void };
  cacheManifest: { args: []; result: void };
  getCachedManifest: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "iconCreate" | "iconValidate" | "iconResize" | "manifestParse" | "sanitizeName" | "platformInfo" | "iconRequirements" | "setAppDir" | "getAppDir" | "setBuildConfig" | "getBuildConfig" | "pathInfo" | "pathJoin" | "manifestPath" | "cacheManifest" | "getCachedManifest";

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

