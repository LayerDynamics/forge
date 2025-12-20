// runtime:shortcuts module - Global keyboard shortcuts for Forge apps.
// Provides hotkey registration, event handling, and persistence across restarts.

// ============================================================================
// Deno Core Type Declarations
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      // Legacy operations (backward compatibility)
      op_shortcuts_info(): ExtensionInfo;
      op_shortcuts_echo(message: string): string;
      // Registration operations
      op_shortcuts_register(config: ShortcutConfigInternal): ShortcutInfo;
      op_shortcuts_unregister(id: string): void;
      op_shortcuts_unregister_all(): void;
      op_shortcuts_list(): ShortcutInfo[];
      op_shortcuts_enable(id: string, enabled: boolean): void;
      // Event operations
      op_shortcuts_next_event(): Promise<ShortcutEvent | null>;
      // Persistence operations
      op_shortcuts_save(): Promise<void>;
      op_shortcuts_load(): Promise<ShortcutConfigInternal[]>;
      op_shortcuts_set_auto_persist(enabled: boolean): void;
      op_shortcuts_get_auto_persist(): boolean;
    };
  };
};

const { core } = Deno;

// ============================================================================
// Extension Info Types (Legacy)
// ============================================================================

/**
 * Extension information for backward compatibility
 */
export interface ExtensionInfo {
  name: string;
  version: string;
  status: string;
}

// ============================================================================
// Core Types
// ============================================================================

/** Internal config format for Rust interop */
interface ShortcutConfigInternal {
  id: string;
  accelerator: string;
  enabled: boolean;
}

/**
 * Configuration for registering a keyboard shortcut
 */
export interface ShortcutConfig {
  /** Unique identifier for the shortcut (e.g., "save-document", "toggle-sidebar") */
  id: string;
  /**
   * Keyboard accelerator string.
   *
   * Supported modifiers:
   * - Ctrl, Control
   * - Alt, Option
   * - Shift
   * - Meta, Cmd, Command, Super
   * - CmdOrCtrl (Command on macOS, Ctrl on Windows/Linux)
   *
   * Supported keys:
   * - Letters: A-Z
   * - Numbers: 0-9
   * - Function keys: F1-F24
   * - Special: Space, Enter, Tab, Backspace, Delete, Escape, Home, End, PageUp, PageDown
   * - Arrows: Up, Down, Left, Right
   * - Punctuation: Minus, Equal, BracketLeft, BracketRight, Backslash, Semicolon, Quote, etc.
   *
   * @example "CmdOrCtrl+S", "Ctrl+Shift+K", "Alt+F4", "F12"
   */
  accelerator: string;
  /** Whether the shortcut is enabled (default: true) */
  enabled?: boolean;
}

/**
 * Shortcut trigger event
 */
export interface ShortcutEvent {
  /** ID of the triggered shortcut */
  id: string;
  /** Timestamp when triggered (Unix milliseconds) */
  timestamp_ms: number;
}

/**
 * Information about a registered shortcut
 */
export interface ShortcutInfo {
  /** Unique identifier */
  id: string;
  /** Accelerator string */
  accelerator: string;
  /** Whether currently enabled */
  enabled: boolean;
  /** Number of times this shortcut has been triggered */
  trigger_count: number;
}

// ============================================================================
// Legacy Operations (Backward Compatibility)
// ============================================================================

/**
 * Get extension information (legacy).
 * @returns Extension info object
 */
export function info(): ExtensionInfo {
  return core.ops.op_shortcuts_info();
}

/**
 * Echo a message back (legacy, for testing).
 * @param message - Message to echo
 * @returns The same message
 */
export function echo(message: string): string {
  return core.ops.op_shortcuts_echo(message);
}

// ============================================================================
// Registration Functions
// ============================================================================

/**
 * Register a global keyboard shortcut.
 *
 * The shortcut will trigger events system-wide, even when the app is not focused.
 *
 * @param config - Shortcut configuration
 * @returns Information about the registered shortcut
 * @throws Error if accelerator is invalid or shortcut with ID already exists
 *
 * @example
 * ```ts
 * import { register } from "runtime:shortcuts";
 *
 * // Register a save shortcut
 * const info = register({
 *   id: "save",
 *   accelerator: "CmdOrCtrl+S",
 * });
 *
 * // Register a custom shortcut
 * register({
 *   id: "toggle-dev-tools",
 *   accelerator: "CmdOrCtrl+Shift+I",
 * });
 * ```
 */
export function register(config: ShortcutConfig): ShortcutInfo {
  const internal: ShortcutConfigInternal = {
    id: config.id,
    accelerator: config.accelerator,
    enabled: config.enabled ?? true,
  };
  return core.ops.op_shortcuts_register(internal);
}

/**
 * Unregister a shortcut by ID.
 *
 * @param id - ID of the shortcut to unregister
 * @throws Error if shortcut with ID does not exist
 *
 * @example
 * ```ts
 * import { unregister } from "runtime:shortcuts";
 *
 * unregister("save");
 * ```
 */
export function unregister(id: string): void {
  core.ops.op_shortcuts_unregister(id);
}

/**
 * Unregister all registered shortcuts.
 *
 * @example
 * ```ts
 * import { unregisterAll } from "runtime:shortcuts";
 *
 * unregisterAll();
 * ```
 */
export function unregisterAll(): void {
  core.ops.op_shortcuts_unregister_all();
}

/**
 * List all registered shortcuts.
 *
 * @returns Array of shortcut info objects
 *
 * @example
 * ```ts
 * import { list } from "runtime:shortcuts";
 *
 * const shortcuts = list();
 * for (const shortcut of shortcuts) {
 *   console.log(`${shortcut.id}: ${shortcut.accelerator}`);
 *   console.log(`  Enabled: ${shortcut.enabled}`);
 *   console.log(`  Triggered: ${shortcut.trigger_count} times`);
 * }
 * ```
 */
export function list(): ShortcutInfo[] {
  return core.ops.op_shortcuts_list();
}

/**
 * Enable or disable a shortcut.
 *
 * Disabled shortcuts will not trigger events.
 *
 * @param id - ID of the shortcut
 * @param enabled - Whether to enable or disable
 * @throws Error if shortcut with ID does not exist
 *
 * @example
 * ```ts
 * import { enable } from "runtime:shortcuts";
 *
 * // Disable a shortcut temporarily
 * enable("save", false);
 *
 * // Re-enable it
 * enable("save", true);
 * ```
 */
export function enable(id: string, enabled: boolean): void {
  core.ops.op_shortcuts_enable(id, enabled);
}

// ============================================================================
// Event Functions
// ============================================================================

/**
 * Wait for the next shortcut event.
 *
 * This is an async operation that resolves when any registered shortcut is triggered.
 * Returns null if the shortcuts extension is shutting down.
 *
 * @returns The shortcut event or null
 *
 * @example
 * ```ts
 * import { register, nextEvent } from "runtime:shortcuts";
 *
 * // Register shortcuts
 * register({ id: "save", accelerator: "CmdOrCtrl+S" });
 * register({ id: "quit", accelerator: "CmdOrCtrl+Q" });
 *
 * // Listen for events
 * while (true) {
 *   const event = await nextEvent();
 *   if (!event) break;
 *
 *   switch (event.id) {
 *     case "save":
 *       console.log("Save triggered!");
 *       break;
 *     case "quit":
 *       console.log("Quit triggered!");
 *       break;
 *   }
 * }
 * ```
 */
export async function nextEvent(): Promise<ShortcutEvent | null> {
  return await core.ops.op_shortcuts_next_event();
}

// ============================================================================
// Persistence Functions
// ============================================================================

/**
 * Save all registered shortcuts to persistent storage.
 *
 * Shortcuts are saved using ext_storage with an app-specific key.
 * Use `load()` to restore them after app restart.
 *
 * @example
 * ```ts
 * import { register, save } from "runtime:shortcuts";
 *
 * register({ id: "save", accelerator: "CmdOrCtrl+S" });
 * register({ id: "open", accelerator: "CmdOrCtrl+O" });
 *
 * // Save for next app launch
 * await save();
 * ```
 */
export async function save(): Promise<void> {
  await core.ops.op_shortcuts_save();
}

/**
 * Load shortcuts from persistent storage.
 *
 * Returns the saved shortcut configurations without registering them.
 * You can then selectively re-register them.
 *
 * @returns Array of saved shortcut configurations
 *
 * @example
 * ```ts
 * import { load, register } from "runtime:shortcuts";
 *
 * // On app startup, restore saved shortcuts
 * const savedShortcuts = await load();
 * for (const config of savedShortcuts) {
 *   try {
 *     register(config);
 *   } catch (e) {
 *     console.error(`Failed to restore shortcut ${config.id}:`, e);
 *   }
 * }
 * ```
 */
export async function load(): Promise<ShortcutConfig[]> {
  return await core.ops.op_shortcuts_load();
}

/**
 * Enable or disable automatic persistence.
 *
 * When enabled, shortcuts are automatically saved whenever
 * they are registered, unregistered, or modified.
 *
 * @param enabled - Whether to enable auto-persist
 *
 * @example
 * ```ts
 * import { setAutoPersist, register } from "runtime:shortcuts";
 *
 * // Enable auto-save
 * setAutoPersist(true);
 *
 * // This will automatically be saved
 * register({ id: "save", accelerator: "CmdOrCtrl+S" });
 * ```
 */
export function setAutoPersist(enabled: boolean): void {
  core.ops.op_shortcuts_set_auto_persist(enabled);
}

/**
 * Check if auto-persist is enabled.
 *
 * @returns Whether auto-persist is enabled
 */
export function getAutoPersist(): boolean {
  return core.ops.op_shortcuts_get_auto_persist();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Register multiple shortcuts at once.
 *
 * @param configs - Array of shortcut configurations
 * @returns Array of registered shortcut info
 *
 * @example
 * ```ts
 * import { registerAll } from "runtime:shortcuts";
 *
 * const shortcuts = registerAll([
 *   { id: "save", accelerator: "CmdOrCtrl+S" },
 *   { id: "open", accelerator: "CmdOrCtrl+O" },
 *   { id: "new", accelerator: "CmdOrCtrl+N" },
 * ]);
 * ```
 */
export function registerAll(configs: ShortcutConfig[]): ShortcutInfo[] {
  return configs.map((config) => register(config));
}

/**
 * Listen for shortcut events with a callback.
 *
 * @param callback - Function called when a shortcut is triggered
 * @returns Stop function to cancel listening
 *
 * @example
 * ```ts
 * import { register, listen } from "runtime:shortcuts";
 *
 * register({ id: "save", accelerator: "CmdOrCtrl+S" });
 *
 * const stop = await listen((event) => {
 *   if (event.id === "save") {
 *     saveDocument();
 *   }
 * });
 *
 * // Later, stop listening
 * stop();
 * ```
 */
export async function listen(
  callback: (event: ShortcutEvent) => void
): Promise<() => void> {
  let running = true;

  // Start async loop
  (async () => {
    while (running) {
      const event = await nextEvent();
      if (!event || !running) break;
      callback(event);
    }
  })();

  // Return stop function
  return () => {
    running = false;
  };
}

/**
 * Create a shortcut handler map.
 *
 * @param handlers - Map of shortcut IDs to handler functions
 * @returns Stop function to cancel listening
 *
 * @example
 * ```ts
 * import { register, handleShortcuts } from "runtime:shortcuts";
 *
 * register({ id: "save", accelerator: "CmdOrCtrl+S" });
 * register({ id: "quit", accelerator: "CmdOrCtrl+Q" });
 *
 * const stop = await handleShortcuts({
 *   save: () => saveDocument(),
 *   quit: () => quitApp(),
 * });
 * ```
 */
export async function handleShortcuts(
  handlers: Record<string, () => void>
): Promise<() => void> {
  return await listen((event) => {
    const handler = handlers[event.id];
    if (handler) {
      handler();
    }
  });
}

/**
 * Parse an accelerator string into its components.
 *
 * @param accelerator - Accelerator string to parse
 * @returns Object with modifiers array and key
 *
 * @example
 * ```ts
 * import { parseAccelerator } from "runtime:shortcuts";
 *
 * const { modifiers, key } = parseAccelerator("CmdOrCtrl+Shift+S");
 * // modifiers: ["CmdOrCtrl", "Shift"]
 * // key: "S"
 * ```
 */
export function parseAccelerator(accelerator: string): {
  modifiers: string[];
  key: string;
} {
  const parts = accelerator.split("+").map((s) => s.trim());
  const key = parts.pop() || "";
  return { modifiers: parts, key };
}

/**
 * Format an accelerator for display (platform-specific).
 *
 * @param accelerator - Accelerator string
 * @returns Human-readable string
 *
 * @example
 * ```ts
 * import { formatAccelerator } from "runtime:shortcuts";
 *
 * // On macOS: "Cmd+Shift+S"
 * // On Windows/Linux: "Ctrl+Shift+S"
 * console.log(formatAccelerator("CmdOrCtrl+Shift+S"));
 * ```
 */
export function formatAccelerator(accelerator: string): string {
  // Detect platform (simplified - in production use runtime:sys)
  const isMac =
    typeof navigator !== "undefined" &&
    navigator.platform?.toLowerCase().includes("mac");

  return accelerator
    .replace(/CmdOrCtrl/gi, isMac ? "Cmd" : "Ctrl")
    .replace(/CommandOrControl/gi, isMac ? "Command" : "Control")
    .replace(/Meta/gi, isMac ? "Cmd" : "Win")
    .replace(/Super/gi, isMac ? "Cmd" : "Win");
}

// ============================================================================
// Convenience Aliases
// ============================================================================

export { register as add };
export { unregister as remove };
export { list as getAll };
