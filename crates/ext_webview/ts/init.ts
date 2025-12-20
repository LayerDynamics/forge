/**
 * @module runtime:webview
 *
 * Lightweight WebView creation and management extension for Forge runtime.
 *
 * This extension provides a simple API for creating and controlling WebView windows,
 * built as a wrapper around the runtime:window system. It offers a streamlined
 * interface for common WebView operations without requiring direct window management.
 *
 * ## Features
 *
 * ### WebView Creation
 * - Create WebView windows with customizable dimensions and behavior
 * - Configure title, URL, size, resizable state
 * - Support for frameless windows and debug mode
 * - Automatic window management integration
 *
 * ### WebView Control
 * - Execute JavaScript code in the WebView context
 * - Set window title and background color dynamically
 * - Toggle fullscreen mode
 * - Close WebView windows
 *
 * ### Event Loop Integration
 * - Centralized event loop (loop and run operations are no-ops)
 * - All WebView events handled by Forge's main event loop
 * - No manual event loop management required
 *
 * ## Error Codes (9000-9001)
 *
 * | Code | Error | Description |
 * |------|-------|-------------|
 * | 9000 | Generic | General WebView operation error |
 * | 9001 | PermissionDenied | Permission denied for window operations |
 *
 * ## Quick Start
 *
 * ```typescript
 * import { webviewNew, webviewEval, webviewExit } from "runtime:webview";
 *
 * // Create a WebView window
 * const webview = await webviewNew({
 *   title: "My App",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Execute JavaScript in the WebView
 * await webviewEval(webview.id, "console.log('Hello from WebView!')");
 *
 * // Close when done
 * await webviewExit(webview.id);
 * ```
 *
 * ## Architecture
 *
 * ext_webview is built as a lightweight wrapper around runtime:window:
 *
 * ```text
 * TypeScript Application
 *   |
 *   | webviewNew(), webviewEval()
 *   v
 * runtime:webview (ext_webview)
 *   |
 *   | WindowCmd::Create, WindowCmd::EvalJs
 *   v
 * runtime:window (ext_window)
 *   |
 *   | wry/tao window management
 *   v
 * Native Window System
 * ```
 *
 * All WebView operations are translated to window commands and sent through
 * ext_window's command channel. This ensures consistent behavior and centralized
 * window management.
 *
 * ## Permission Model
 *
 * WebView operations require window creation permissions as defined in your
 * app's manifest.app.toml:
 *
 * ```toml
 * [permissions.ui]
 * windows = true  # Required for WebView operations
 * ```
 *
 * Operations will fail with error 9001 if permissions are not granted.
 *
 * ## Usage Patterns
 *
 * ### Creating a Browser-Like Window
 *
 * ```typescript
 * const browser = await webviewNew({
 *   title: "Web Browser",
 *   url: "about:blank",
 *   width: 1024,
 *   height: 768,
 *   resizable: true,
 *   debug: true,  // Enable DevTools
 *   frameless: false
 * });
 * ```
 *
 * ### Frameless Application Window
 *
 * ```typescript
 * const app = await webviewNew({
 *   title: "Frameless App",
 *   url: "app://index.html",
 *   width: 600,
 *   height: 400,
 *   resizable: false,
 *   debug: false,
 *   frameless: true  // No title bar or borders
 * });
 * ```
 *
 * ### Dynamic Content Injection
 *
 * ```typescript
 * const view = await webviewNew({
 *   title: "Dynamic Content",
 *   url: "about:blank",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Inject HTML content
 * await webviewEval(view.id, `
 *   document.body.innerHTML = '<h1>Hello, World!</h1>';
 * `);
 *
 * // Style the page
 * await webviewSetColor(view.id, 240, 240, 255, 255);  // Light blue
 * ```
 *
 * @example
 * ```typescript
 * import * as webview from "runtime:webview";
 *
 * // Create WebView with all options
 * const wv = await webview.webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Update title
 * await webview.webviewSetTitle(wv.id, "New Title");
 *
 * // Toggle fullscreen
 * await webview.webviewSetFullscreen(wv.id, true);
 *
 * // Execute JavaScript
 * await webview.webviewEval(wv.id, "alert('Hello!')");
 *
 * // Close
 * await webview.webviewExit(wv.id);
 * ```
 */

declare const Deno: {
  core: {
    ops: {
      op_host_webview_new(opts: WebViewOptions): WebViewHandle;
      op_host_webview_exit(params: { id: string }): void;
      op_host_webview_eval(params: { id: string; js: string }): void;
      op_host_webview_set_color(params: { id: string; r: number; g: number; b: number; a: number }): void;
      op_host_webview_set_title(params: { id: string; title: string }): void;
      op_host_webview_set_fullscreen(params: { id: string; fullscreen: boolean }): void;
      op_host_webview_loop(params: { id: string; blocking: number }): Promise<{ code: number }>;
      op_host_webview_run(params: { id: string }): Promise<void>;
    };
  };
};

// deno-lint-ignore no-explicit-any
const core = (Deno as any).core;

/**
 * Configuration options for creating a new WebView window.
 *
 * All fields are required to ensure consistent window creation behavior.
 * Use this interface when calling webviewNew() to specify the WebView's
 * appearance and initial state.
 *
 * @property title - Window title displayed in the title bar (if not frameless)
 * @property url - Initial URL to load (supports http://, https://, file://, app:// protocols)
 * @property width - Initial window width in pixels
 * @property height - Initial window height in pixels
 * @property resizable - Whether the window can be resized by the user
 * @property debug - Enable DevTools for debugging (WebView inspector)
 * @property frameless - Remove window decorations (title bar, borders)
 *
 * @example
 * ```typescript
 * const options: WebViewOptions = {
 *   title: "My Application",
 *   url: "https://example.com",
 *   width: 1024,
 *   height: 768,
 *   resizable: true,
 *   debug: true,
 *   frameless: false
 * };
 * ```
 */
export interface WebViewOptions {
  /** Window title displayed in title bar */
  title: string;
  /** Initial URL to load */
  url: string;
  /** Window width in pixels */
  width: number;
  /** Window height in pixels */
  height: number;
  /** Allow user to resize window */
  resizable: boolean;
  /** Enable DevTools for debugging */
  debug: boolean;
  /** Remove window decorations (title bar, borders) */
  frameless: boolean;
}

/**
 * Handle to a WebView window returned by webviewNew().
 *
 * The handle contains an opaque ID used to reference the WebView in
 * subsequent operations (eval, setTitle, setFullscreen, exit, etc.).
 *
 * @property id - Unique identifier for the WebView window
 *
 * @example
 * ```typescript
 * const handle = await webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Use the ID for subsequent operations
 * await webviewEval(handle.id, "console.log('Hello!')");
 * await webviewExit(handle.id);
 * ```
 */
export interface WebViewHandle {
  /** Unique identifier for the WebView window */
  id: string;
}

/**
 * Create a new WebView window.
 *
 * Creates a new window with an embedded WebView that renders web content.
 * The WebView can load URLs from any supported protocol (http://, https://,
 * file://, app://). Returns a handle containing the window ID for use in
 * subsequent operations.
 *
 * This operation is translated to WindowCmd::Create and sent through the
 * ext_window command channel.
 *
 * @param opts - Configuration options for the WebView (title, URL, size, etc.)
 * @returns Handle containing the WebView window ID
 *
 * @throws Error [9000] if window creation fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * // Create a resizable browser window
 * const browser = await webviewNew({
 *   title: "Web Browser",
 *   url: "https://example.com",
 *   width: 1024,
 *   height: 768,
 *   resizable: true,
 *   debug: true,
 *   frameless: false
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Create a frameless app window
 * const app = await webviewNew({
 *   title: "My App",
 *   url: "app://index.html",
 *   width: 600,
 *   height: 400,
 *   resizable: false,
 *   debug: false,
 *   frameless: true
 * });
 * ```
 */
function webviewNew(opts: WebViewOptions): WebViewHandle {
  return core.ops.op_host_webview_new(opts);
}

/**
 * Close a WebView window.
 *
 * Closes the WebView window identified by the given ID. This operation is
 * translated to WindowCmd::Close and sent through the ext_window command channel.
 *
 * After closing, the window ID becomes invalid and should not be used for
 * further operations.
 *
 * @param id - WebView window ID from webviewNew()
 *
 * @throws Error [9000] if window close fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Close when done
 * await webviewExit(view.id);
 * ```
 */
function webviewExit(id: string): void {
  core.ops.op_host_webview_exit({ id });
}

/**
 * Execute JavaScript code in a WebView window.
 *
 * Evaluates the provided JavaScript code in the WebView's execution context.
 * This operation is translated to WindowCmd::EvalJs and sent through the
 * ext_window command channel.
 *
 * The JavaScript code runs asynchronously in the WebView. Return values are
 * not captured - use this for side effects only (DOM manipulation, logging, etc.).
 *
 * @param id - WebView window ID from webviewNew()
 * @param js - JavaScript code to execute
 *
 * @throws Error [9000] if script evaluation fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "about:blank",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Inject content
 * await webviewEval(view.id, `
 *   document.body.innerHTML = '<h1>Hello, World!</h1>';
 * `);
 *
 * // Add styles
 * await webviewEval(view.id, `
 *   document.body.style.backgroundColor = '#f0f0f0';
 *   document.body.style.fontFamily = 'Arial, sans-serif';
 * `);
 * ```
 */
function webviewEval(id: string, js: string): void {
  core.ops.op_host_webview_eval({ id, js });
}

/**
 * Set the background color of a WebView window.
 *
 * Sets the WebView's background color by injecting a CSS rule targeting the
 * body element. This operation is translated to WindowCmd::InjectCss and sent
 * through the ext_window command channel.
 *
 * The color is specified as RGBA values (red, green, blue, alpha). Each channel
 * ranges from 0-255. Alpha channel controls opacity (0 = transparent, 255 = opaque).
 *
 * @param id - WebView window ID from webviewNew()
 * @param r - Red channel (0-255)
 * @param g - Green channel (0-255)
 * @param b - Blue channel (0-255)
 * @param a - Alpha channel (0-255, 0 = transparent, 255 = opaque)
 *
 * @throws Error [9000] if color setting fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "about:blank",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Set light blue background
 * await webviewSetColor(view.id, 240, 240, 255, 255);
 *
 * // Set semi-transparent white
 * await webviewSetColor(view.id, 255, 255, 255, 128);
 * ```
 */
function webviewSetColor(id: string, r: number, g: number, b: number, a: number): void {
  core.ops.op_host_webview_set_color({ id, r, g, b, a });
}

/**
 * Set the title of a WebView window.
 *
 * Updates the window title displayed in the title bar. This operation is
 * translated to WindowCmd::SetTitle and sent through the ext_window command channel.
 *
 * For frameless windows, the title is not visible but may still be used by
 * the operating system (task switcher, accessibility, etc.).
 *
 * @param id - WebView window ID from webviewNew()
 * @param title - New window title
 *
 * @throws Error [9000] if title setting fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Initial Title",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Update title dynamically
 * await webviewSetTitle(view.id, "Updated Title - Page Loaded");
 *
 * // Update based on content
 * await webviewSetTitle(view.id, `Viewing: ${currentUrl}`);
 * ```
 */
function webviewSetTitle(id: string, title: string): void {
  core.ops.op_host_webview_set_title({ id, title });
}

/**
 * Toggle fullscreen mode for a WebView window.
 *
 * Switches the window between fullscreen and normal windowed mode. This operation
 * is translated to WindowCmd::SetFullscreen and sent through the ext_window
 * command channel.
 *
 * In fullscreen mode, the window occupies the entire screen with all decorations
 * (title bar, borders, etc.) hidden.
 *
 * @param id - WebView window ID from webviewNew()
 * @param fullscreen - true to enter fullscreen, false to exit
 *
 * @throws Error [9000] if fullscreen toggle fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // Enter fullscreen mode
 * await webviewSetFullscreen(view.id, true);
 *
 * // Exit fullscreen mode
 * await webviewSetFullscreen(view.id, false);
 * ```
 */
function webviewSetFullscreen(id: string, fullscreen: boolean): void {
  core.ops.op_host_webview_set_fullscreen({ id, fullscreen });
}

/**
 * Event loop shim (no-op in Forge).
 *
 * This function exists for API compatibility with reference WebView plugins,
 * but performs no operation in Forge. The Forge runtime uses a centralized
 * event loop that handles all window and WebView events automatically.
 *
 * Always returns { code: 0 } to indicate success.
 *
 * @param id - WebView window ID from webviewNew()
 * @param blocking - Loop behavior (0 = non-blocking, 1 = blocking) - ignored in Forge
 * @returns Promise resolving to { code: 0 }
 *
 * @throws Error [9000] if ID validation fails
 * @throws Error [9001] if permission denied for window operations
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // This is a no-op in Forge (for API compatibility)
 * const result = await webviewLoop(view.id, 0);
 * console.log(result.code); // Always 0
 * ```
 */
async function webviewLoop(id: string, blocking: number): Promise<{ code: number }> {
  return await core.ops.op_host_webview_loop({ id, blocking });
}

/**
 * Run loop shim (no-op in Forge).
 *
 * This function exists for API compatibility with reference WebView plugins,
 * but performs no operation in Forge. The Forge runtime uses a centralized
 * event loop that handles all window and WebView events automatically.
 *
 * Always returns immediately with success.
 *
 * @param id - WebView window ID from webviewNew()
 * @returns Promise resolving to void
 *
 * @example
 * ```typescript
 * const view = await webviewNew({
 *   title: "Example",
 *   url: "https://example.com",
 *   width: 800,
 *   height: 600,
 *   resizable: true,
 *   debug: false,
 *   frameless: false
 * });
 *
 * // This is a no-op in Forge (for API compatibility)
 * await webviewRun(view.id);
 * ```
 */
async function webviewRun(id: string): Promise<void> {
  await core.ops.op_host_webview_run({ id });
}

// Aliases with friendlier names
const newWebView = webviewNew;
const exitWebView = webviewExit;
const evalInWebView = webviewEval;
const setWebViewColor = webviewSetColor;
const setWebViewTitle = webviewSetTitle;
const setWebViewFullscreen = webviewSetFullscreen;
const webViewLoop = webviewLoop;
const runWebView = webviewRun;

export {
  // primary names
  webviewNew,
  webviewExit,
  webviewEval,
  webviewSetColor,
  webviewSetTitle,
  webviewSetFullscreen,
  webviewLoop,
  webviewRun,
  // aliases
  newWebView,
  exitWebView,
  evalInWebView,
  setWebViewColor,
  setWebViewTitle,
  setWebViewFullscreen,
  webViewLoop,
  runWebView,
};
