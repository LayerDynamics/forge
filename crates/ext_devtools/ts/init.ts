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
