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
