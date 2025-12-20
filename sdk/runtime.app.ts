// runtime:app module - TypeScript wrapper for Deno core ops

// Deno.core type declaration
declare const Deno: {
  core: {
    ops: {
      op_app_quit(): Promise<void>;
      op_app_exit(exitCode: number): void;
      op_app_relaunch(): Promise<void>;
      op_app_get_version(): string;
      op_app_get_name(): string;
      op_app_get_identifier(): string;
      op_app_get_path(pathType: string): string;
      op_app_is_packaged(): boolean;
      op_app_get_locale(): LocaleResult;
      op_app_request_single_instance_lock(): Promise<boolean>;
      op_app_release_single_instance_lock(): Promise<void>;
      op_app_focus(): Promise<void>;
      op_app_hide(): Promise<void>;
      op_app_show(): Promise<void>;
      op_app_set_badge_count(count: number | null): Promise<void>;
      op_app_set_user_model_id(appId: string): void;
    };
  };
};

export interface LocaleResult {
  language: string;
  country: string | null;
  locale: string;
}

const core = Deno.core;

/**
 * System locale information
 */
export interface LocaleInfo {
  /** Language code (e.g., "en") */
  language: string;
  /** Country code (e.g., "US") */
  country: string | null;
  /** Full locale string (e.g., "en-US") */
  locale: string;
}

/**
 * Types of special paths that can be requested
 */
export type PathType =
  | "home"
  | "appData"
  | "documents"
  | "downloads"
  | "desktop"
  | "music"
  | "pictures"
  | "videos"
  | "temp"
  | "exe"
  | "resources"
  | "logs"
  | "cache";

/**
 * Quit the application gracefully.
 * Triggers cleanup handlers before exiting.
 */
export async function quit(): Promise<void> {
  return await core.ops.op_app_quit();
}

/**
 * Force exit the application immediately.
 * No cleanup handlers are called.
 * @param exitCode - Exit code (default: 0)
 */
export function exit(exitCode: number = 0): void {
  return core.ops.op_app_exit(exitCode);
}

/**
 * Relaunch the application.
 * The current instance will exit and a new instance will start.
 */
export async function relaunch(): Promise<void> {
  return await core.ops.op_app_relaunch();
}

/**
 * Get the application version.
 * @returns Version string from manifest.app.toml
 */
export function getVersion(): string {
  return core.ops.op_app_get_version();
}

/**
 * Get the application name.
 * @returns App name from manifest.app.toml
 */
export function getName(): string {
  return core.ops.op_app_get_name();
}

/**
 * Get the application identifier.
 * @returns App identifier (e.g., "com.example.app")
 */
export function getIdentifier(): string {
  return core.ops.op_app_get_identifier();
}

/**
 * Get a special system or application path.
 * @param pathType - Type of path to retrieve
 * @returns The requested path
 */
export function getPath(pathType: PathType): string {
  return core.ops.op_app_get_path(pathType);
}

/**
 * Check if the application is running in packaged mode.
 * @returns true if running as a bundled application, false in development
 */
export function isPackaged(): boolean {
  return core.ops.op_app_is_packaged();
}

/**
 * Get the system locale information.
 * @returns Locale information including language and country codes
 */
export function getLocale(): LocaleInfo {
  const result = core.ops.op_app_get_locale();
  return {
    language: result.language,
    country: result.country,
    locale: result.locale,
  };
}

/**
 * Request a single instance lock.
 * Prevents multiple instances of the application from running.
 * @returns true if the lock was acquired, false if another instance holds it
 */
export async function requestSingleInstanceLock(): Promise<boolean> {
  return await core.ops.op_app_request_single_instance_lock();
}

/**
 * Release the single instance lock.
 * Call this before exiting to allow another instance to start.
 */
export async function releaseSingleInstanceLock(): Promise<void> {
  return await core.ops.op_app_release_single_instance_lock();
}

/**
 * Bring the application to the foreground.
 * Makes the app's windows visible and focused.
 */
export async function focus(): Promise<void> {
  return await core.ops.op_app_focus();
}

/**
 * Hide all application windows.
 * On macOS, this hides the application. On other platforms, minimizes windows.
 */
export async function hide(): Promise<void> {
  return await core.ops.op_app_hide();
}

/**
 * Show all application windows.
 * Restores hidden or minimized windows.
 */
export async function show(): Promise<void> {
  return await core.ops.op_app_show();
}

/**
 * Set the dock/taskbar badge count.
 * @param count - Badge count to display, or null/undefined to clear
 */
export async function setBadgeCount(count?: number | null): Promise<void> {
  return await core.ops.op_app_set_badge_count(count ?? null);
}

/**
 * Set the Windows App User Model ID.
 * Used for taskbar grouping on Windows.
 * @param appId - Application user model ID
 */
export function setUserModelId(appId: string): void {
  return core.ops.op_app_set_user_model_id(appId);
}

// Convenience exports
export const version = getVersion;
export const name = getName;
export const identifier = getIdentifier;


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  quit: { args: []; result: void };
  exit: { args: []; result: void };
  relaunch: { args: []; result: void };
  getVersion: { args: []; result: void };
  getName: { args: []; result: void };
  getIdentifier: { args: []; result: void };
  getPath: { args: []; result: void };
  isPackaged: { args: []; result: void };
  getLocale: { args: []; result: void };
  requestSingleInstanceLock: { args: []; result: void };
  releaseSingleInstanceLock: { args: []; result: void };
  focus: { args: []; result: void };
  hide: { args: []; result: void };
  show: { args: []; result: void };
  setBadgeCount: { args: []; result: void };
  setUserModelId: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "quit" | "exit" | "relaunch" | "getVersion" | "getName" | "getIdentifier" | "getPath" | "isPackaged" | "getLocale" | "requestSingleInstanceLock" | "releaseSingleInstanceLock" | "focus" | "hide" | "show" | "setBadgeCount" | "setUserModelId";

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

