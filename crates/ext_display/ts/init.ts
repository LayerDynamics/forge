// runtime:display module - Display and monitor information for Forge apps.
// Provides monitor enumeration, cursor position tracking, and display change events.

// ============================================================================
// Deno Core Type Declarations
// ============================================================================

declare const Deno: {
  core: {
    ops: {
      // Legacy operations (backward compatibility)
      op_display_info(): ExtensionInfo;
      op_display_echo(message: string): string;
      // Display query operations
      op_display_get_all(): MonitorInfo[];
      op_display_get_primary(): MonitorInfo | null;
      op_display_get_by_id(id: string): MonitorInfo | null;
      op_display_get_at_point(x: number, y: number): MonitorInfo | null;
      op_display_get_cursor_position(): CursorPosition;
      op_display_get_count(): number;
      // Subscription operations
      op_display_subscribe(options: SubscribeOptionsInternal): Promise<string>;
      op_display_unsubscribe(subscriptionId: string): void;
      op_display_next_event(subscriptionId: string): Promise<DisplayEvent | null>;
      op_display_subscriptions(): SubscriptionInfo[];
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

/**
 * A 2D position in screen coordinates
 */
export interface Position {
  /** X coordinate */
  x: number;
  /** Y coordinate */
  y: number;
}

/**
 * A 2D size in pixels
 */
export interface Size {
  /** Width in pixels */
  width: number;
  /** Height in pixels */
  height: number;
}

/**
 * Information about a connected monitor/display
 */
export interface MonitorInfo {
  /** Unique identifier for this monitor (format: "name:x,y") */
  id: string;
  /** Human-readable name of the monitor (may be null for some displays) */
  name: string | null;
  /** Position of the monitor in virtual screen coordinates */
  position: Position;
  /** Size of the monitor in pixels */
  size: Size;
  /** DPI scale factor (e.g., 1.0 for 100%, 2.0 for 200% HiDPI) */
  scale_factor: number;
  /** Whether this is the primary monitor */
  is_primary: boolean;
  /** Refresh rate in millihertz (e.g., 60000 for 60Hz), null if unavailable */
  refresh_rate_millihertz: number | null;
}

/**
 * Current cursor position with optional monitor context
 */
export interface CursorPosition {
  /** X coordinate in virtual screen space */
  x: number;
  /** Y coordinate in virtual screen space */
  y: number;
  /** ID of the monitor the cursor is on (if determinable) */
  monitor_id: string | null;
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * Types of changes that can occur to a monitor
 */
export type MonitorChangeType =
  | "ScaleFactor"
  | "Position"
  | "Size"
  | "RefreshRate"
  | "Primary";

/**
 * Display event for monitor changes
 */
export type DisplayEvent =
  | { type: "MonitorConnected"; data: { monitor: MonitorInfo } }
  | { type: "MonitorDisconnected"; data: { monitor_id: string } }
  | { type: "MonitorChanged"; data: { monitor: MonitorInfo; changes: MonitorChangeType[] } };

// ============================================================================
// Subscription Types
// ============================================================================

/** Internal subscription options format (snake_case for Rust interop) */
interface SubscribeOptionsInternal {
  interval_ms: number;
}

/**
 * Options for subscribing to display events
 */
export interface SubscribeOptions {
  /**
   * Polling interval in milliseconds for detecting monitor changes.
   * Minimum: 500ms. Default: 1000ms.
   */
  intervalMs?: number;
}

/**
 * Information about an active display subscription
 */
export interface SubscriptionInfo {
  /** Unique subscription ID */
  id: string;
  /** Polling interval in milliseconds */
  interval_ms: number;
  /** Whether the subscription is currently active */
  is_active: boolean;
  /** Number of events delivered so far */
  event_count: number;
}

// ============================================================================
// Legacy Operations (Backward Compatibility)
// ============================================================================

/**
 * Get extension information (legacy).
 * @returns Extension info object
 */
export function info(): ExtensionInfo {
  return core.ops.op_display_info();
}

/**
 * Echo a message back (legacy, for testing).
 * @param message - Message to echo
 * @returns The same message
 */
export function echo(message: string): string {
  return core.ops.op_display_echo(message);
}

// ============================================================================
// Display Query Functions
// ============================================================================

/**
 * Get all connected monitors.
 *
 * @returns Array of monitor information for all connected displays
 *
 * @example
 * ```ts
 * import { getAll } from "runtime:display";
 *
 * const monitors = getAll();
 * for (const monitor of monitors) {
 *   console.log(`${monitor.name}: ${monitor.size.width}x${monitor.size.height}`);
 *   console.log(`  Position: (${monitor.position.x}, ${monitor.position.y})`);
 *   console.log(`  Scale: ${monitor.scale_factor}x`);
 *   console.log(`  Primary: ${monitor.is_primary}`);
 * }
 * ```
 */
export function getAll(): MonitorInfo[] {
  return core.ops.op_display_get_all();
}

/**
 * Get the primary monitor.
 *
 * @returns The primary monitor info, or null if no primary can be determined
 *
 * @example
 * ```ts
 * import { getPrimary } from "runtime:display";
 *
 * const primary = getPrimary();
 * if (primary) {
 *   console.log(`Primary display: ${primary.size.width}x${primary.size.height}`);
 * }
 * ```
 */
export function getPrimary(): MonitorInfo | null {
  return core.ops.op_display_get_primary();
}

/**
 * Get a monitor by its unique ID.
 *
 * @param id - The monitor ID (format: "name:x,y")
 * @returns The monitor info if found, null otherwise
 *
 * @example
 * ```ts
 * import { getById, getAll } from "runtime:display";
 *
 * const monitors = getAll();
 * const monitor = getById(monitors[0].id);
 * if (monitor) {
 *   console.log(`Found: ${monitor.name}`);
 * }
 * ```
 */
export function getById(id: string): MonitorInfo | null {
  return core.ops.op_display_get_by_id(id);
}

/**
 * Get the monitor at a specific screen coordinate.
 *
 * @param x - X coordinate in virtual screen space
 * @param y - Y coordinate in virtual screen space
 * @returns The monitor containing the point, or null if no monitor contains it
 *
 * @example
 * ```ts
 * import { getAtPoint, getCursorPosition } from "runtime:display";
 *
 * const cursor = getCursorPosition();
 * const monitor = getAtPoint(cursor.x, cursor.y);
 * if (monitor) {
 *   console.log(`Cursor is on: ${monitor.name}`);
 * }
 * ```
 */
export function getAtPoint(x: number, y: number): MonitorInfo | null {
  return core.ops.op_display_get_at_point(x, y);
}

/**
 * Get the current cursor position.
 *
 * Note: Platform-specific implementation:
 * - macOS: Uses AppleScript
 * - Windows: Uses Win32 API GetCursorPos
 * - Linux: Uses xdotool (must be installed)
 *
 * @returns Current cursor position in virtual screen coordinates
 *
 * @example
 * ```ts
 * import { getCursorPosition } from "runtime:display";
 *
 * const pos = getCursorPosition();
 * console.log(`Cursor at: (${pos.x}, ${pos.y})`);
 * ```
 */
export function getCursorPosition(): CursorPosition {
  return core.ops.op_display_get_cursor_position();
}

/**
 * Get the number of connected monitors.
 *
 * @returns Number of monitors
 *
 * @example
 * ```ts
 * import { getCount } from "runtime:display";
 *
 * const count = getCount();
 * console.log(`${count} monitor(s) connected`);
 * ```
 */
export function getCount(): number {
  return core.ops.op_display_get_count();
}

// ============================================================================
// Subscription API
// ============================================================================

/**
 * Subscribe to display change events.
 *
 * Creates a subscription that monitors for:
 * - Monitor connections (new display plugged in)
 * - Monitor disconnections (display unplugged)
 * - Monitor changes (resolution, scale, position changes)
 *
 * Use `nextEvent()` to receive events, and `unsubscribe()` to stop.
 * Maximum 10 concurrent subscriptions allowed per runtime.
 *
 * @param options - Subscription configuration
 * @returns Subscription ID to use with nextEvent/unsubscribe
 *
 * @example
 * ```ts
 * import { subscribe, nextEvent, unsubscribe } from "runtime:display";
 *
 * // Start monitoring display changes (check every second)
 * const subId = await subscribe({ intervalMs: 1000 });
 *
 * // Listen for events
 * while (true) {
 *   const event = await nextEvent(subId);
 *   if (!event) break;
 *
 *   switch (event.type) {
 *     case "MonitorConnected":
 *       console.log(`New monitor: ${event.data.monitor.name}`);
 *       break;
 *     case "MonitorDisconnected":
 *       console.log(`Monitor removed: ${event.data.monitor_id}`);
 *       break;
 *     case "MonitorChanged":
 *       console.log(`Monitor changed: ${event.data.changes.join(", ")}`);
 *       break;
 *   }
 * }
 *
 * // Stop monitoring
 * unsubscribe(subId);
 * ```
 */
export async function subscribe(options: SubscribeOptions = {}): Promise<string> {
  const internalOptions: SubscribeOptionsInternal = {
    interval_ms: options.intervalMs ?? 1000,
  };
  return await core.ops.op_display_subscribe(internalOptions);
}

/**
 * Get the next display event from a subscription.
 *
 * This is an async operation that waits for the next event to occur.
 * Returns null if the subscription has been cancelled.
 *
 * @param subscriptionId - ID returned from subscribe()
 * @returns Next display event or null if subscription ended
 *
 * @example
 * ```ts
 * const event = await nextEvent(subId);
 * if (event) {
 *   console.log(`Event type: ${event.type}`);
 * }
 * ```
 */
export async function nextEvent(subscriptionId: string): Promise<DisplayEvent | null> {
  return await core.ops.op_display_next_event(subscriptionId);
}

/**
 * Cancel a display subscription.
 *
 * Stops the background monitoring for this subscription.
 * Any pending nextEvent() calls will return null.
 *
 * @param subscriptionId - ID returned from subscribe()
 * @throws Error if subscription ID is invalid
 *
 * @example
 * ```ts
 * unsubscribe(subId);
 * ```
 */
export function unsubscribe(subscriptionId: string): void {
  core.ops.op_display_unsubscribe(subscriptionId);
}

/**
 * List all active display subscriptions.
 *
 * @returns Array of subscription info objects
 *
 * @example
 * ```ts
 * import { getSubscriptions } from "runtime:display";
 *
 * const subs = getSubscriptions();
 * for (const sub of subs) {
 *   console.log(`Subscription ${sub.id}: ${sub.event_count} events`);
 * }
 * ```
 */
export function getSubscriptions(): SubscriptionInfo[] {
  return core.ops.op_display_subscriptions();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/**
 * Get display configuration summary.
 *
 * @returns Object with display count, primary monitor, and total virtual screen size
 *
 * @example
 * ```ts
 * import { getDisplayInfo } from "runtime:display";
 *
 * const info = getDisplayInfo();
 * console.log(`${info.count} display(s)`);
 * console.log(`Virtual screen: ${info.virtualSize.width}x${info.virtualSize.height}`);
 * if (info.primary) {
 *   console.log(`Primary: ${info.primary.name}`);
 * }
 * ```
 */
export function getDisplayInfo(): {
  count: number;
  primary: MonitorInfo | null;
  monitors: MonitorInfo[];
  virtualSize: Size;
} {
  const monitors = getAll();
  const primary = monitors.find((m) => m.is_primary) ?? null;

  // Calculate virtual screen bounds
  let minX = 0, minY = 0, maxX = 0, maxY = 0;
  for (const m of monitors) {
    minX = Math.min(minX, m.position.x);
    minY = Math.min(minY, m.position.y);
    maxX = Math.max(maxX, m.position.x + m.size.width);
    maxY = Math.max(maxY, m.position.y + m.size.height);
  }

  return {
    count: monitors.length,
    primary,
    monitors,
    virtualSize: {
      width: maxX - minX,
      height: maxY - minY,
    },
  };
}

/**
 * Get the monitor the cursor is currently on.
 *
 * Combines getCursorPosition() and getAtPoint() for convenience.
 *
 * @returns The monitor under the cursor, or null
 *
 * @example
 * ```ts
 * import { getMonitorAtCursor } from "runtime:display";
 *
 * const monitor = getMonitorAtCursor();
 * if (monitor) {
 *   console.log(`Cursor is on: ${monitor.name}`);
 * }
 * ```
 */
export function getMonitorAtCursor(): MonitorInfo | null {
  const cursor = getCursorPosition();
  return getAtPoint(cursor.x, cursor.y);
}

/**
 * Watch for display changes with a callback.
 *
 * @param callback - Function called for each display event
 * @param options - Subscription options
 * @returns Stop function to cancel watching
 *
 * @example
 * ```ts
 * import { watchDisplays } from "runtime:display";
 *
 * const stop = await watchDisplays((event) => {
 *   console.log(`Display event: ${event.type}`);
 * });
 *
 * // Later, stop watching
 * stop();
 * ```
 */
export async function watchDisplays(
  callback: (event: DisplayEvent) => void,
  options: SubscribeOptions = {}
): Promise<() => void> {
  const subId = await subscribe(options);

  let running = true;

  // Start async loop
  (async () => {
    while (running) {
      const event = await nextEvent(subId);
      if (!event || !running) break;
      callback(event);
    }
  })();

  // Return stop function
  return () => {
    running = false;
    unsubscribe(subId);
  };
}

/**
 * Format refresh rate from millihertz to Hz string.
 *
 * @param millihertz - Refresh rate in millihertz
 * @returns Formatted string (e.g., "60 Hz")
 *
 * @example
 * ```ts
 * import { formatRefreshRate, getPrimary } from "runtime:display";
 *
 * const primary = getPrimary();
 * if (primary?.refresh_rate_millihertz) {
 *   console.log(formatRefreshRate(primary.refresh_rate_millihertz)); // "60 Hz"
 * }
 * ```
 */
export function formatRefreshRate(millihertz: number): string {
  const hz = millihertz / 1000;
  return `${Math.round(hz)} Hz`;
}

/**
 * Format monitor resolution as a string.
 *
 * @param size - Size object with width and height
 * @param scaleFactor - Optional scale factor to show effective resolution
 * @returns Formatted string (e.g., "1920x1080" or "1920x1080 (3840x2160 @2x)")
 *
 * @example
 * ```ts
 * import { formatResolution, getPrimary } from "runtime:display";
 *
 * const primary = getPrimary();
 * if (primary) {
 *   console.log(formatResolution(primary.size, primary.scale_factor));
 * }
 * ```
 */
export function formatResolution(size: Size, scaleFactor?: number): string {
  const base = `${size.width}x${size.height}`;
  if (scaleFactor && scaleFactor !== 1.0) {
    const native = `${Math.round(size.width * scaleFactor)}x${Math.round(size.height * scaleFactor)}`;
    return `${base} (${native} @${scaleFactor}x)`;
  }
  return base;
}

// ============================================================================
// Convenience Aliases
// ============================================================================

export { getAll as all };
export { getAll as monitors };
export { getPrimary as primary };
export { getCount as count };
export { getCursorPosition as cursor };
