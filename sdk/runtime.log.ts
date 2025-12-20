// runtime:log module - structured logging bridge to host tracing.
// Also provides browser console forwarding via IPC.

export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

export type LogLevel = "trace" | "debug" | "info" | "warn" | "warning" | "error";

declare const Deno: {
  core: {
    ops: {
      op_log_info(): ExtensionInfo;
      op_log_emit(level: string, message: string, fields?: Record<string, unknown>): void;
      op_ipc_send(windowId: string, channel: string, payload: unknown): Promise<void>;
    };
  };
};

const { core } = Deno;

// ============================================================================
// Host Logging (outputs to terminal via tracing)
// ============================================================================

export function info(): ExtensionInfo {
  return core.ops.op_log_info();
}

export function emit(level: LogLevel, message: string, fields?: Record<string, unknown>): void {
  core.ops.op_log_emit(level, message, fields ?? {});
}

export function trace(message: string, fields?: Record<string, unknown>): void {
  emit("trace", message, fields);
}

export function debug(message: string, fields?: Record<string, unknown>): void {
  emit("debug", message, fields);
}

export function infoLog(message: string, fields?: Record<string, unknown>): void {
  emit("info", message, fields);
}

export function warn(message: string, fields?: Record<string, unknown>): void {
  emit("warn", message, fields);
}

export function error(message: string, fields?: Record<string, unknown>): void {
  emit("error", message, fields);
}

// ============================================================================
// Browser Console Forwarding (outputs to browser DevTools)
// ============================================================================

/**
 * Configuration for browser console forwarding.
 * Set a default window ID to avoid passing it on every call.
 */
let defaultWindowId: string | null = null;

/**
 * Set the default window ID for browser console forwarding.
 * Once set, you can omit the windowId parameter in browserConsole calls.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, browserConsole } from "runtime:log";
 *
 * setDefaultWindow(win.id);
 * browserConsole.log("This appears in browser DevTools");
 * ```
 */
export function setDefaultWindow(windowId: string): void {
  defaultWindowId = windowId;
}

/**
 * Send a log message to the browser's DevTools console via IPC.
 * @param windowId - The window to send to (uses default if not specified)
 * @param level - Log level (trace, debug, info, warn, error)
 * @param message - The message to log
 * @param fields - Optional structured data to include
 */
async function sendToBrowser(
  windowId: string | undefined,
  level: LogLevel,
  message: string,
  fields?: Record<string, unknown>
): Promise<void> {
  const targetWindow = windowId ?? defaultWindowId;
  if (!targetWindow) {
    // Fallback to host logging if no window ID
    emit(level, `[no window] ${message}`, fields);
    return;
  }
  await core.ops.op_ipc_send(targetWindow, "__console__", {
    level,
    message,
    fields: fields ?? {},
  });
}

/**
 * Browser console logging utilities.
 * These send messages to the browser DevTools console via IPC.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, browserConsole } from "runtime:log";
 *
 * // Set default window once
 * setDefaultWindow(win.id);
 *
 * // Then log without specifying window
 * browserConsole.log("Hello from Deno backend!");
 * browserConsole.warn("This is a warning");
 * browserConsole.error("Something went wrong", { code: 500 });
 *
 * // Or specify window explicitly
 * browserConsole.log("Message to specific window", undefined, otherWindowId);
 * ```
 */
export const browserConsole = {
  /** Log a trace-level message to browser DevTools */
  trace: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "trace", message, fields),

  /** Log a debug-level message to browser DevTools */
  debug: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "debug", message, fields),

  /** Log an info-level message to browser DevTools */
  log: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "info", message, fields),

  /** Log an info-level message to browser DevTools */
  info: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "info", message, fields),

  /** Log a warning to browser DevTools */
  warn: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "warn", message, fields),

  /** Log an error to browser DevTools */
  error: (message: string, fields?: Record<string, unknown>, windowId?: string) =>
    sendToBrowser(windowId, "error", message, fields),
};

/**
 * Dual-output logging - logs to BOTH host terminal AND browser DevTools.
 * Useful for development when you want to see logs in both places.
 *
 * @example
 * ```ts
 * import { setDefaultWindow, dualLog } from "runtime:log";
 *
 * setDefaultWindow(win.id);
 * dualLog.info("Starting app..."); // Appears in terminal AND browser
 * ```
 */
export const dualLog = {
  trace: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("trace", message, fields);
    await sendToBrowser(windowId, "trace", message, fields);
  },

  debug: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("debug", message, fields);
    await sendToBrowser(windowId, "debug", message, fields);
  },

  info: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("info", message, fields);
    await sendToBrowser(windowId, "info", message, fields);
  },

  warn: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("warn", message, fields);
    await sendToBrowser(windowId, "warn", message, fields);
  },

  error: async (message: string, fields?: Record<string, unknown>, windowId?: string) => {
    emit("error", message, fields);
    await sendToBrowser(windowId, "error", message, fields);
  },
};


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  emit: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "emit";

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

