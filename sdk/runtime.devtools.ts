/**
 * @module runtime:devtools
 *
 * Developer tools control extension for Forge runtime.
 *
 * This extension provides a simple API for opening, closing, and checking the
 * state of browser DevTools for WebView windows. Built as a thin wrapper around
 * the ext_window runtime, it offers programmatic control over the DevTools panel
 * that developers use for debugging and inspecting web content.
 *
 * ## Features
 *
 * ### DevTools Control
 * - Open DevTools panel for any window
 * - Close DevTools panel programmatically
 * - Check if DevTools are currently open
 * - Simple boolean return values for all operations
 *
 * ### Window Integration
 * - Works with any window created via ext_window or ext_webview
 * - DevTools state persists until explicitly closed or window destroyed
 * - Multiple windows can have DevTools open simultaneously
 *
 * ## Error Codes (9100-9101)
 *
 * | Code | Error | Description |
 * |------|-------|-------------|
 * | 9100 | Generic | General DevTools operation error |
 * | 9101 | PermissionDenied | Permission denied for window operations |
 *
 * ## Quick Start
 *
 * ```typescript
 * import { open, close, isOpen } from "runtime:devtools";
 *
 * // Assuming you have a window ID from ext_window or ext_webview
 * const windowId = "window-123";
 *
 * // Open DevTools
 * await open(windowId);
 *
 * // Check if open
 * const devToolsOpen = await isOpen(windowId);
 * console.log("DevTools open:", devToolsOpen); // true
 *
 * // Close DevTools
 * await close(windowId);
 * ```
 *
 * ## Architecture
 *
 * ext_devtools is a thin wrapper around ext_window:
 *
 * ```text
 * TypeScript Application
 *   |
 *   | open(), close(), isOpen()
 *   v
 * runtime:devtools (ext_devtools)
 *   |
 *   | WindowCmd::OpenDevTools, WindowCmd::CloseDevTools, WindowCmd::IsDevToolsOpen
 *   v
 * runtime:window (ext_window)
 *   |
 *   | wry DevTools API
 *   v
 * Native WebView DevTools
 * ```
 *
 * All DevTools operations are translated to window commands and sent through
 * ext_window's command channel, ensuring consistent behavior with the window
 * management system.
 *
 * ## Permission Model
 *
 * DevTools operations require window management permissions as defined in your
 * app's manifest.app.toml:
 *
 * ```toml
 * [permissions.ui]
 * windows = true  # Required for DevTools operations
 * ```
 *
 * Operations will fail with error 9101 if permissions are not granted.
 *
 * ## Usage Patterns
 *
 * ### Conditional DevTools (Development Mode)
 *
 * ```typescript
 * import { open } from "runtime:devtools";
 *
 * const isDev = Deno.env.get("MODE") === "development";
 *
 * if (isDev) {
 *   await open(windowId);
 * }
 * ```
 *
 * ### Toggle DevTools
 *
 * ```typescript
 * import { open, close, isOpen } from "runtime:devtools";
 *
 * async function toggleDevTools(windowId: string) {
 *   if (await isOpen(windowId)) {
 *     await close(windowId);
 *   } else {
 *     await open(windowId);
 *   }
 * }
 * ```
 *
 * ### DevTools Keyboard Shortcut
 *
 * ```typescript
 * import { open, close, isOpen } from "runtime:devtools";
 * import { on } from "runtime:shortcuts";
 *
 * // F12 to toggle DevTools
 * await on("F12", async () => {
 *   if (await isOpen(currentWindowId)) {
 *     await close(currentWindowId);
 *   } else {
 *     await open(currentWindowId);
 *   }
 * });
 * ```
 *
 * @example
 * ```typescript
 * import * as devtools from "runtime:devtools";
 * import { webviewNew } from "runtime:webview";
 *
 * // Create window with debug mode enabled
 * const window = await webviewNew({
 *   title: "Debug Window",
 *   url: "app://index.html",
 *   width: 1200,
 *   height: 800,
 *   resizable: true,
 *   debug: true,  // DevTools available but not automatically open
 *   frameless: false
 * });
 *
 * // Open DevTools programmatically
 * await devtools.open(window.id);
 *
 * // Later, close DevTools
 * await devtools.close(window.id);
 * ```
 */

declare const Deno: {
  core: {
    ops: {
      op_devtools_open(windowId: string): Promise<boolean>;
      op_devtools_close(windowId: string): Promise<boolean>;
      op_devtools_is_open(windowId: string): Promise<boolean>;
    };
  };
};

// deno-lint-ignore no-explicit-any
const core = (Deno as any).core;

/**
 * Open the DevTools panel for a window.
 *
 * Opens the browser DevTools (inspector, console, network monitor, etc.) for
 * the specified window. The DevTools panel appears as a separate docked panel
 * or window depending on the platform and WebView implementation.
 *
 * This operation is translated to WindowCmd::OpenDevTools and sent through the
 * ext_window command channel.
 *
 * @param windowId - Window ID from ext_window or ext_webview
 * @returns Promise resolving to true on success
 *
 * @throws Error [9100] if DevTools open fails
 * @throws Error [9101] if permission denied for window operations
 *
 * @example
 * ```typescript
 * import { open } from "runtime:devtools";
 * import { webviewNew } from "runtime:webview";
 *
 * // Create window with debug mode enabled
 * const window = await webviewNew({
 *   title: "My App",
 *   url: "app://index.html",
 *   width: 1200,
 *   height: 800,
 *   resizable: true,
 *   debug: true,  // DevTools available
 *   frameless: false
 * });
 *
 * // Open DevTools programmatically
 * await open(window.id);
 * ```
 *
 * @example
 * ```typescript
 * import { open } from "runtime:devtools";
 *
 * // Open DevTools only in development mode
 * const isDev = Deno.env.get("MODE") === "development";
 * if (isDev) {
 *   await open(windowId);
 * }
 * ```
 */
export async function open(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_open(windowId);
}

/**
 * Close the DevTools panel for a window.
 *
 * Closes the browser DevTools panel if it is currently open for the specified
 * window. If the DevTools are already closed, this operation succeeds without
 * error.
 *
 * This operation is translated to WindowCmd::CloseDevTools and sent through the
 * ext_window command channel.
 *
 * @param windowId - Window ID from ext_window or ext_webview
 * @returns Promise resolving to true on success
 *
 * @throws Error [9100] if DevTools close fails
 * @throws Error [9101] if permission denied for window operations
 *
 * @example
 * ```typescript
 * import { open, close } from "runtime:devtools";
 *
 * // Open DevTools for debugging
 * await open(windowId);
 *
 * // User completes debugging...
 *
 * // Close DevTools to reclaim screen space
 * await close(windowId);
 * ```
 *
 * @example
 * ```typescript
 * import { close, isOpen } from "runtime:devtools";
 *
 * // Close DevTools if open
 * if (await isOpen(windowId)) {
 *   await close(windowId);
 * }
 * ```
 */
export async function close(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_close(windowId);
}

/**
 * Check if the DevTools panel is currently open for a window.
 *
 * Queries the current state of the DevTools panel for the specified window.
 * Returns true if DevTools are open, false otherwise.
 *
 * This operation is translated to WindowCmd::IsDevToolsOpen and sent through the
 * ext_window command channel.
 *
 * @param windowId - Window ID from ext_window or ext_webview
 * @returns Promise resolving to true if DevTools are open, false otherwise
 *
 * @throws Error [9100] if state query fails
 * @throws Error [9101] if permission denied for window operations
 *
 * @example
 * ```typescript
 * import { isOpen } from "runtime:devtools";
 *
 * // Check DevTools state
 * const devToolsOpen = await isOpen(windowId);
 * console.log("DevTools open:", devToolsOpen);
 * ```
 *
 * @example
 * ```typescript
 * import { open, close, isOpen } from "runtime:devtools";
 *
 * // Toggle DevTools on/off
 * async function toggleDevTools(windowId: string) {
 *   if (await isOpen(windowId)) {
 *     await close(windowId);
 *     console.log("DevTools closed");
 *   } else {
 *     await open(windowId);
 *     console.log("DevTools opened");
 *   }
 * }
 * ```
 *
 * @example
 * ```typescript
 * import { isOpen } from "runtime:devtools";
 *
 * // Conditional UI state based on DevTools
 * const devToolsButton = document.getElementById("devtools-toggle");
 * if (await isOpen(windowId)) {
 *   devToolsButton.textContent = "Close DevTools";
 *   devToolsButton.classList.add("active");
 * } else {
 *   devToolsButton.textContent = "Open DevTools";
 *   devToolsButton.classList.remove("active");
 * }
 * ```
 */
export async function isOpen(windowId: string): Promise<boolean> {
  return await core.ops.op_devtools_is_open(windowId);
}


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  open: { args: []; result: void };
  close: { args: []; result: void };
  isOpen: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "open" | "close" | "isOpen";

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

