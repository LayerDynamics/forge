// runtime:sys module - TypeScript wrapper for Deno core ops

declare const Deno: {
  core: {
    ops: {
      op_sys_info(): RawSystemInfo;
      op_sys_env_get(key: string): string | null;
      op_sys_env_set(key: string, value: string): void;
      op_sys_cwd(): string;
      op_sys_home_dir(): string | null;
      op_sys_temp_dir(): string;
      op_sys_clipboard_read(): Promise<string>;
      op_sys_clipboard_write(text: string): Promise<void>;
      op_sys_notify(title: string, body: string): Promise<void>;
      op_sys_notify_ext(opts: NotifyOptions): Promise<void>;
      op_sys_power_info(): Promise<PowerInfo>;
      // Enhanced operations
      op_sys_env_all(): Record<string, string>;
      op_sys_env_delete(key: string): void;
      op_sys_locale(): LocaleInfoResult;
      op_sys_app_paths(): AppPathsResult;
    };
  };
};

export interface RawSystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpu_count: number;
  cpuCount?: number;
}

export interface SystemInfo {
  os: string;
  arch: string;
  hostname: string;
  cpuCount: number;
}

export interface NotifyOptions {
  title: string;
  body?: string;
  icon?: string;
  sound?: boolean;
}

export interface PowerInfo {
  state: string;
  percentage: number | null;
  timeToFull: number | null;
  timeToEmpty: number | null;
}

// Enhanced types
export interface LocaleInfoResult {
  language: string;
  country: string | null;
  locale: string;
}

export interface AppPathsResult {
  home: string | null;
  documents: string | null;
  downloads: string | null;
  desktop: string | null;
  music: string | null;
  pictures: string | null;
  videos: string | null;
  data: string | null;
  config: string | null;
  cache: string | null;
  runtime: string | null;
}

/**
 * System locale information
 */
export interface LocaleInfo {
  language: string;
  country: string | null;
  locale: string;
}

/**
 * Standard application paths
 */
export interface AppPaths {
  home: string | null;
  documents: string | null;
  downloads: string | null;
  desktop: string | null;
  music: string | null;
  pictures: string | null;
  videos: string | null;
  data: string | null;
  config: string | null;
  cache: string | null;
  runtime: string | null;
}

const core = Deno.core;

export function info(): SystemInfo {
  const result = core.ops.op_sys_info();
  return {
    os: result.os,
    arch: result.arch,
    hostname: result.hostname,
    cpuCount: result.cpu_count,
  };
}

export function getEnv(key: string): string | null {
  return core.ops.op_sys_env_get(key);
}

export function setEnv(key: string, value: string): void {
  core.ops.op_sys_env_set(key, value);
}

export function cwd(): string {
  return core.ops.op_sys_cwd();
}

export function homeDir(): string | null {
  return core.ops.op_sys_home_dir();
}

export function tempDir(): string {
  return core.ops.op_sys_temp_dir();
}

export const clipboard = {
  async read(): Promise<string> {
    return await core.ops.op_sys_clipboard_read();
  },

  async write(text: string): Promise<void> {
    return await core.ops.op_sys_clipboard_write(text);
  },
};

export async function notify(title: string, body: string = ""): Promise<void> {
  return await core.ops.op_sys_notify(title, body);
}

export async function notifyExt(opts: NotifyOptions): Promise<void> {
  return await core.ops.op_sys_notify_ext(opts);
}

export async function powerInfo(): Promise<PowerInfo> {
  return await core.ops.op_sys_power_info();
}

// ============================================================================
// Enhanced Operations
// ============================================================================

/**
 * Get all environment variables.
 * @returns Object with all environment variables
 */
export function getAllEnv(): Record<string, string> {
  return core.ops.op_sys_env_all();
}

/**
 * Delete an environment variable.
 * @param key - The environment variable key to delete
 */
export function deleteEnv(key: string): void {
  return core.ops.op_sys_env_delete(key);
}

/**
 * Get system locale information.
 * @returns Locale information including language and country codes
 */
export function locale(): LocaleInfo {
  return core.ops.op_sys_locale();
}

/**
 * Get standard application paths.
 * @returns Object with standard directory paths
 */
export function appPaths(): AppPaths {
  return core.ops.op_sys_app_paths();
}

// Convenience aliases
export { getAllEnv as envAll };
export { deleteEnv as envDelete };
export { locale as getLocale };
export { appPaths as getPaths };


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  envGet: { args: []; result: void };
  envSet: { args: []; result: void };
  cwd: { args: []; result: void };
  homeDir: { args: []; result: void };
  tempDir: { args: []; result: void };
  clipboardRead: { args: []; result: void };
  clipboardWrite: { args: []; result: void };
  notify: { args: []; result: void };
  notifyExt: { args: []; result: void };
  powerInfo: { args: []; result: void };
  envAll: { args: []; result: void };
  envDelete: { args: []; result: void };
  locale: { args: []; result: void };
  appPaths: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "envGet" | "envSet" | "cwd" | "homeDir" | "tempDir" | "clipboardRead" | "clipboardWrite" | "notify" | "notifyExt" | "powerInfo" | "envAll" | "envDelete" | "locale" | "appPaths";

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

