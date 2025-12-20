// runtime:dock extension bindings
// macOS dock customization - icon, badge, bounce, menu

// ============================================================================
// Type Definitions
// ============================================================================

/** Extension metadata */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

/** Bounce type for dock icon animation */
export type BounceType = "critical" | "informational";

/** Result of a bounce operation */
export interface BounceResult {
  /** Bounce request ID (used to cancel) */
  id: number;
  /** Whether the bounce was started successfully */
  success: boolean;
}

/** Menu item for dock menu */
export interface MenuItem {
  /** Unique identifier for the menu item */
  id?: string;
  /** Display label */
  label: string;
  /** Keyboard shortcut */
  accelerator?: string;
  /** Whether the item is enabled */
  enabled?: boolean;
  /** Whether the item is checked (for checkbox items) */
  checked?: boolean;
  /** Submenu items */
  submenu?: MenuItem[];
  /** Item type: "normal", "checkbox", "separator" */
  type?: "normal" | "checkbox" | "separator";
}

// ============================================================================
// Deno Core Bindings
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      op_dock_info(): ExtensionInfo;
      op_dock_bounce(bounceType: BounceType | null): BounceResult;
      op_dock_cancel_bounce(bounceId: number): void;
      op_dock_set_badge(text: string): void;
      op_dock_get_badge(): string;
      op_dock_hide(): void;
      op_dock_show(): void;
      op_dock_is_visible(): boolean;
      op_dock_set_icon(iconPath: string): boolean;
      op_dock_set_menu(menu: MenuItem[]): boolean;
    };
  };
};

const { core } = Deno;
const ops = core.ops;

// ============================================================================
// Public API
// ============================================================================

/**
 * Get extension information
 */
export function info(): ExtensionInfo {
  return ops.op_dock_info();
}

/**
 * Bounce the dock icon to get user attention.
 *
 * @param type - Bounce type:
 *   - "critical": Continues bouncing until app is activated
 *   - "informational": Bounces once (default)
 * @returns Bounce result with ID for cancellation
 *
 * @example
 * ```typescript
 * import { bounce, cancelBounce } from "runtime:dock";
 *
 * // Informational bounce (bounces once)
 * const result = bounce();
 *
 * // Critical bounce (continues until activated)
 * const result = bounce("critical");
 *
 * // Cancel the bounce
 * cancelBounce(result.id);
 * ```
 *
 * @platform macOS only (no-op on other platforms)
 */
export function bounce(type: BounceType = "informational"): BounceResult {
  return ops.op_dock_bounce(type);
}

/**
 * Cancel a dock icon bounce.
 *
 * @param bounceId - ID returned from bounce()
 *
 * @platform macOS only (no-op on other platforms)
 */
export function cancelBounce(bounceId: number): void {
  ops.op_dock_cancel_bounce(bounceId);
}

/**
 * Set the dock badge text.
 *
 * @param text - Badge text to display. Empty string clears the badge.
 *
 * @example
 * ```typescript
 * import { setBadge } from "runtime:dock";
 *
 * // Set badge to show unread count
 * setBadge("5");
 *
 * // Clear the badge
 * setBadge("");
 * ```
 *
 * @platform macOS only (no-op on other platforms)
 */
export function setBadge(text: string): void {
  ops.op_dock_set_badge(text);
}

/**
 * Get the current dock badge text.
 *
 * @returns Current badge text, or empty string if no badge
 *
 * @platform macOS only (returns empty on other platforms)
 */
export function getBadge(): string {
  return ops.op_dock_get_badge();
}

/**
 * Hide the dock icon.
 *
 * This changes the app to "accessory" mode where it doesn't show in the dock
 * or the Cmd+Tab app switcher, but can still have windows.
 *
 * @platform macOS only (no-op on other platforms)
 */
export function hide(): void {
  ops.op_dock_hide();
}

/**
 * Show the dock icon.
 *
 * This restores the app to "regular" mode where it appears in the dock
 * and Cmd+Tab app switcher.
 *
 * @platform macOS only (no-op on other platforms)
 */
export function show(): void {
  ops.op_dock_show();
}

/**
 * Check if the dock icon is visible.
 *
 * @returns true if dock icon is visible
 *
 * @platform macOS only (always returns true on other platforms)
 */
export function isVisible(): boolean {
  return ops.op_dock_is_visible();
}

/**
 * Set a custom dock icon.
 *
 * @param iconPath - Path to image file (PNG, JPEG, etc.), or empty string to reset to default
 * @returns true if icon was set successfully
 *
 * @example
 * ```typescript
 * import { setIcon } from "runtime:dock";
 *
 * // Set custom dock icon
 * setIcon("./assets/custom-icon.png");
 *
 * // Reset to default icon
 * setIcon("");
 * ```
 *
 * @platform macOS only (returns false on other platforms)
 */
export function setIcon(iconPath: string): boolean {
  return ops.op_dock_set_icon(iconPath);
}

/**
 * Set the dock menu (right-click menu on dock icon).
 *
 * @param menu - Array of menu items
 * @returns true if menu was set successfully
 *
 * @example
 * ```typescript
 * import { setMenu } from "runtime:dock";
 *
 * setMenu([
 *   { id: "new-window", label: "New Window" },
 *   { type: "separator" },
 *   { id: "preferences", label: "Preferences..." },
 * ]);
 * ```
 *
 * @platform macOS only (returns false on other platforms)
 */
export function setMenu(menu: MenuItem[]): boolean {
  return ops.op_dock_set_menu(menu);
}

// ============================================================================
// Default Export
// ============================================================================

export default {
  info,
  bounce,
  cancelBounce,
  setBadge,
  getBadge,
  hide,
  show,
  isVisible,
  setIcon,
  setMenu,
};


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  info: { args: []; result: void };
  bounce: { args: []; result: void };
  cancelBounce: { args: []; result: void };
  setBadge: { args: []; result: void };
  getBadge: { args: []; result: void };
  hide: { args: []; result: void };
  show: { args: []; result: void };
  isVisible: { args: []; result: void };
  setIcon: { args: []; result: void };
  setMenu: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "info" | "bounce" | "cancelBounce" | "setBadge" | "getBadge" | "hide" | "show" | "isVisible" | "setIcon" | "setMenu";

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

