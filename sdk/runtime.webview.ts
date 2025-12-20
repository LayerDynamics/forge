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


// ============================================================================
// Extensibility API (auto-generated)
// ============================================================================

/** Registry of operations with their argument and result types */
interface OpRegistry {
  webviewNew: { args: []; result: void };
  webviewExit: { args: []; result: void };
  webviewEval: { args: []; result: void };
  webviewSetColor: { args: []; result: void };
  webviewSetTitle: { args: []; result: void };
  webviewSetFullscreen: { args: []; result: void };
  webviewLoop: { args: []; result: void };
  webviewRun: { args: []; result: void };
}

/** Extract argument types for an operation */
type OpArgs<T extends keyof OpRegistry> = OpRegistry[T]['args'];

/** Extract result type for an operation */
type OpResult<T extends keyof OpRegistry> = OpRegistry[T]['result'];

/** Valid operation names for this extension */
type OpName = "webviewNew" | "webviewExit" | "webviewEval" | "webviewSetColor" | "webviewSetTitle" | "webviewSetFullscreen" | "webviewLoop" | "webviewRun";

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

