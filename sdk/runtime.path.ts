/**
 * @module runtime:path
 *
 * Cross-platform path manipulation utilities for Forge applications.
 *
 * This module provides functions for working with filesystem paths in a
 * platform-independent way. All operations handle forward slashes on Unix
 * and backslashes on Windows automatically.
 *
 * ## Features
 * - Join path segments with correct separators
 * - Extract directory names and basenames
 * - Get file extensions
 * - Parse paths into components
 * - Cross-platform path normalization
 *
 * ## Platform Behavior
 * - **Unix (macOS/Linux)**: Uses forward slashes (`/`)
 * - **Windows**: Uses backslashes (`\`)
 * - Operations automatically use platform-appropriate separators
 *
 * ## No Permissions Required
 * Path operations are pure string manipulation and don't require filesystem
 * permissions. They work with any path string, whether or not it exists.
 *
 * @example
 * ```typescript
 * import { join, dirname, basename, extname } from "runtime:path";
 *
 * // Join path segments
 * const configPath = join("./data", "config.json");
 *
 * // Extract components
 * const dir = dirname("/usr/local/bin/node");  // "/usr/local/bin"
 * const file = basename("/usr/local/bin/node"); // "node"
 * const ext = extname("file.txt");              // ".txt"
 * ```
 */

/**
 * Components of a parsed path.
 */
export interface PathParts {
  /** Directory path (empty string if no directory) */
  dir: string;
  /** Base filename including extension */
  base: string;
  /** File extension including the dot (empty string if no extension) */
  ext: string;
}

declare const Deno: {
  core: {
    ops: {
      op_path_join(base: string, segments: string[]): string;
      op_path_dirname(path: string): string;
      op_path_basename(path: string): string;
      op_path_extname(path: string): string;
      op_path_parts(path: string): PathParts;
    };
  };
};

const { core } = Deno;
const ops = {
  join: core.ops.op_path_join,
  dirname: core.ops.op_path_dirname,
  basename: core.ops.op_path_basename,
  extname: core.ops.op_path_extname,
  parts: core.ops.op_path_parts,
};

/**
 * Joins path segments into a single path using platform-appropriate separators.
 *
 * On Unix systems, uses forward slashes (`/`). On Windows, uses backslashes (`\`).
 * Automatically normalizes redundant separators and handles relative path components.
 *
 * @param base - The base path to start from
 * @param segments - Additional path segments to append
 * @returns Combined path with platform-appropriate separators
 *
 * @example
 * ```typescript
 * // Unix: "./data/config.json"
 * // Windows: ".\\data\\config.json"
 * const path = join("./data", "config.json");
 * ```
 *
 * @example
 * ```typescript
 * // Build nested paths
 * const imagePath = join("./assets", "images", "logo.png");
 * // Unix: "./assets/images/logo.png"
 * // Windows: ".\\assets\\images\\logo.png"
 * ```
 *
 * @example
 * ```typescript
 * // Join with absolute paths
 * const binPath = join("/usr", "local", "bin", "node");
 * // Unix: "/usr/local/bin/node"
 * ```
 */
export function join(base: string, ...segments: string[]): string {
  return ops.join(base, segments);
}

/**
 * Extracts the directory path from a file path.
 *
 * Returns everything before the final path separator. If there is no directory
 * component, returns an empty string.
 *
 * @param path - The path to extract the directory from
 * @returns The directory portion of the path, or empty string if none
 *
 * @example
 * ```typescript
 * const dir = dirname("/usr/local/bin/node");
 * console.log(dir); // "/usr/local/bin"
 * ```
 *
 * @example
 * ```typescript
 * const dir = dirname("./data/config.json");
 * console.log(dir); // "./data"
 * ```
 *
 * @example
 * ```typescript
 * // No directory component
 * const dir = dirname("file.txt");
 * console.log(dir); // ""
 * ```
 */
export function dirname(path: string): string {
  return ops.dirname(path);
}

/**
 * Extracts the final component of a path (filename with extension).
 *
 * Returns the last segment of the path after the final separator. If the path
 * ends with a separator, returns an empty string.
 *
 * @param path - The path to extract the basename from
 * @returns The filename portion of the path, or empty string if none
 *
 * @example
 * ```typescript
 * const file = basename("/usr/local/bin/node");
 * console.log(file); // "node"
 * ```
 *
 * @example
 * ```typescript
 * const file = basename("./data/config.json");
 * console.log(file); // "config.json"
 * ```
 *
 * @example
 * ```typescript
 * // Path with no directory
 * const file = basename("readme.md");
 * console.log(file); // "readme.md"
 * ```
 */
export function basename(path: string): string {
  return ops.basename(path);
}

/**
 * Extracts the file extension from a path.
 *
 * Returns the extension including the leading dot. If there is no extension,
 * returns an empty string. Only considers the portion after the last dot in
 * the basename.
 *
 * @param path - The path to extract the extension from
 * @returns The file extension including the dot, or empty string if none
 *
 * @example
 * ```typescript
 * const ext = extname("file.txt");
 * console.log(ext); // ".txt"
 * ```
 *
 * @example
 * ```typescript
 * const ext = extname("archive.tar.gz");
 * console.log(ext); // ".gz" (only last extension)
 * ```
 *
 * @example
 * ```typescript
 * // No extension
 * const ext = extname("README");
 * console.log(ext); // ""
 * ```
 *
 * @example
 * ```typescript
 * // Hidden file (dot prefix is not an extension)
 * const ext = extname(".gitignore");
 * console.log(ext); // ""
 * ```
 */
export function extname(path: string): string {
  return ops.extname(path);
}

/**
 * Parses a path into its directory, basename, and extension components.
 *
 * This is a convenience function that combines `dirname()`, `basename()`, and
 * `extname()` in a single operation.
 *
 * @param path - The path to parse
 * @returns Object with `dir`, `base`, and `ext` properties
 *
 * @example
 * ```typescript
 * const parts = parts("/usr/local/bin/node");
 * console.log(parts.dir);  // "/usr/local/bin"
 * console.log(parts.base); // "node"
 * console.log(parts.ext);  // ""
 * ```
 *
 * @example
 * ```typescript
 * const parts = parts("./data/config.json");
 * console.log(parts.dir);  // "./data"
 * console.log(parts.base); // "config.json"
 * console.log(parts.ext);  // ".json"
 * ```
 *
 * @example
 * ```typescript
 * // Use to build modified paths
 * const original = "./images/photo.jpg";
 * const p = parts(original);
 * const thumbnail = join(p.dir, `thumb_${p.base}`);
 * console.log(thumbnail); // "./images/thumb_photo.jpg"
 * ```
 */
export function parts(path: string): PathParts {
  return ops.parts(path);
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  join: { args: []; result: void };
  dirname: { args: []; result: void };
  basename: { args: []; result: void };
  extname: { args: []; result: void };
  parts: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "join" | "dirname" | "basename" | "extname" | "parts";

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

