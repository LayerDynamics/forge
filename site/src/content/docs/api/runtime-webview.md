---
title: "runtime:webview"
description: "Lightweight WebView creation and management for Forge applications"
slug: api/runtime-webview
---

Lightweight WebView creation and management extension for Forge applications. Provides a streamlined interface for creating and controlling WebView windows without requiring direct window management.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_webview](/docs/crates/ext-webview) for implementation details.

## Overview

The `runtime:webview` module provides a simplified API for creating WebView windows, built as a wrapper around the `runtime:window` system. It handles common WebView operations like:

- Creating WebView windows with customizable dimensions
- Executing JavaScript in the WebView context
- Setting window title and background color
- Toggling fullscreen mode
- Closing WebView windows

## Architecture

```
TypeScript Application
  |
  | webviewNew(), webviewEval()
  v
runtime:webview (ext_webview)
  |
  | WindowCmd::Create, WindowCmd::EvalJs
  v
runtime:window (ext_window)
  |
  | wry/tao window management
  v
Native Window System
```

All WebView operations are translated to window commands and sent through ext_window's command channel, ensuring consistent behavior and centralized window management.

## Permissions

WebView operations require window creation permissions in `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for WebView operations
```

Operations fail with error code 9001 if permissions are not granted.

## Import

```typescript
import {
  webviewNew,
  webviewExit,
  webviewEval,
  webviewSetColor,
  webviewSetTitle,
  webviewSetFullscreen,
  webviewLoop,
  webviewRun,
  // Aliases
  newWebView,
  exitWebView,
  evalInWebView,
  setWebViewColor,
  setWebViewTitle,
  setWebViewFullscreen,
  type WebViewOptions,
  type WebViewHandle
} from "runtime:webview";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.webview.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
webviewNew(opts: WebViewOptions): WebViewHandle
webviewExit(id: string): void
webviewEval(id: string, js: string): void
webviewSetColor(id: string, r: number, g: number, b: number, a: number): void
webviewSetTitle(id: string, title: string): void
webviewSetFullscreen(id: string, fullscreen: boolean): void
webviewLoop(id: string, blocking: number): Promise<
webviewRun(id: string): Promise<void>
```
<!-- /forge:api -->

### webviewNew(options)

Create a new WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `options` | `WebViewOptions` | Configuration for the WebView window |

**WebViewOptions:**

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `title` | `string` | Yes | Window title displayed in title bar |
| `url` | `string` | Yes | Initial URL to load (http://, https://, file://, app://) |
| `width` | `number` | Yes | Window width in pixels |
| `height` | `number` | Yes | Window height in pixels |
| `resizable` | `boolean` | Yes | Allow user to resize window |
| `debug` | `boolean` | Yes | Enable DevTools for debugging |
| `frameless` | `boolean` | Yes | Remove window decorations (title bar, borders) |

**Returns:** `WebViewHandle` - Handle containing the WebView window ID

**Throws:**
- Error [9000] - Window creation failed
- Error [9001] - Permission denied

**Example:**

```typescript
import { webviewNew } from "runtime:webview";

// Create a resizable browser window
const browser = await webviewNew({
  title: "Web Browser",
  url: "https://example.com",
  width: 1024,
  height: 768,
  resizable: true,
  debug: true,  // Enable DevTools
  frameless: false
});

console.log(`Window created with ID: ${browser.id}`);
```

---

### webviewExit(id)

Close a WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID from `webviewNew()` |

**Throws:**
- Error [9000] - Window close failed
- Error [9001] - Permission denied

**Example:**

```typescript
import { webviewNew, webviewExit } from "runtime:webview";

const view = await webviewNew({
  title: "Example",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Close when done
await webviewExit(view.id);
```

---

### webviewEval(id, js)

Execute JavaScript code in a WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID |
| `js` | `string` | JavaScript code to execute |

**Throws:**
- Error [9000] - Script evaluation failed
- Error [9001] - Permission denied

> **Note:** The JavaScript runs asynchronously. Return values are not captured - use this for side effects only (DOM manipulation, logging, etc.).

**Example:**

```typescript
import { webviewNew, webviewEval } from "runtime:webview";

const view = await webviewNew({
  title: "Dynamic Content",
  url: "about:blank",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Inject HTML content
await webviewEval(view.id, `
  document.body.innerHTML = '<h1>Hello, World!</h1>';
`);

// Add styles
await webviewEval(view.id, `
  document.body.style.backgroundColor = '#f0f0f0';
  document.body.style.fontFamily = 'Arial, sans-serif';
  document.body.style.padding = '20px';
`);

// Add interactivity
await webviewEval(view.id, `
  document.body.innerHTML += '<button id="btn">Click Me</button>';
  document.getElementById('btn').onclick = () => alert('Clicked!');
`);
```

---

### webviewSetColor(id, r, g, b, a)

Set the background color of a WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID |
| `r` | `number` | Red channel (0-255) |
| `g` | `number` | Green channel (0-255) |
| `b` | `number` | Blue channel (0-255) |
| `a` | `number` | Alpha channel (0-255, 0 = transparent, 255 = opaque) |

**Throws:**
- Error [9000] - Color setting failed
- Error [9001] - Permission denied

**Example:**

```typescript
import { webviewNew, webviewSetColor } from "runtime:webview";

const view = await webviewNew({
  title: "Colored Background",
  url: "about:blank",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Set light blue background (fully opaque)
await webviewSetColor(view.id, 240, 240, 255, 255);

// Set semi-transparent white
await webviewSetColor(view.id, 255, 255, 255, 128);

// Set fully transparent
await webviewSetColor(view.id, 0, 0, 0, 0);
```

---

### webviewSetTitle(id, title)

Set the title of a WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID |
| `title` | `string` | New window title |

**Throws:**
- Error [9000] - Title setting failed
- Error [9001] - Permission denied

> **Note:** For frameless windows, the title is not visible but may still be used by the operating system (task switcher, accessibility, etc.).

**Example:**

```typescript
import { webviewNew, webviewSetTitle } from "runtime:webview";

const view = await webviewNew({
  title: "Loading...",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Update title based on state
await webviewSetTitle(view.id, "My App - Ready");

// Update with dynamic content
const pageTitle = "Dashboard";
await webviewSetTitle(view.id, `My App - ${pageTitle}`);
```

---

### webviewSetFullscreen(id, fullscreen)

Toggle fullscreen mode for a WebView window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID |
| `fullscreen` | `boolean` | `true` to enter fullscreen, `false` to exit |

**Throws:**
- Error [9000] - Fullscreen toggle failed
- Error [9001] - Permission denied

**Example:**

```typescript
import { webviewNew, webviewSetFullscreen } from "runtime:webview";

const view = await webviewNew({
  title: "Video Player",
  url: "app://player.html",
  width: 1280,
  height: 720,
  resizable: true,
  debug: false,
  frameless: false
});

// Enter fullscreen mode
await webviewSetFullscreen(view.id, true);

// Exit fullscreen mode
await webviewSetFullscreen(view.id, false);
```

---

### webviewLoop(id, blocking) / webviewRun(id)

Event loop shims (no-op in Forge).

These functions exist for API compatibility with reference WebView plugins but perform no operation in Forge. The Forge runtime uses a centralized event loop that handles all window and WebView events automatically.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `string` | WebView window ID |
| `blocking` | `number` | Loop behavior (ignored in Forge) |

**Returns:**
- `webviewLoop`: `Promise<{ code: number }>` - Always returns `{ code: 0 }`
- `webviewRun`: `Promise<void>`

**Example:**

```typescript
// These are no-ops in Forge (for API compatibility only)
await webviewLoop(view.id, 0);
await webviewRun(view.id);
```

## Type Definitions

```typescript
/**
 * Configuration options for creating a WebView window.
 */
interface WebViewOptions {
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
 * Handle to a WebView window.
 */
interface WebViewHandle {
  /** Unique identifier for the WebView window */
  id: string;
}
```

## Lifecycle Hooks

WebView operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:webview";

onBefore("webviewNew", (args) => {
  console.log("Creating new WebView...");
});

onBefore("webviewEval", (args) => {
  console.log("Executing JavaScript in WebView");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:webview";

onAfter("webviewNew", (result) => {
  console.log("WebView created with ID:", result.id);
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:webview";

onError("webviewNew", (error) => {
  console.error("Failed to create WebView:", error.message);
});
```

### removeAllHooks(opName?)

```typescript
import { removeAllHooks } from "runtime:webview";

// Remove hooks for specific operation
removeAllHooks("webviewEval");

// Remove all hooks
removeAllHooks();
```

**Available operation names:** `"webviewNew"`, `"webviewExit"`, `"webviewEval"`, `"webviewSetColor"`, `"webviewSetTitle"`, `"webviewSetFullscreen"`, `"webviewLoop"`, `"webviewRun"`

## Handler System

Register custom handlers for WebView operations.

### registerHandler(name, handler)

```typescript
import { registerHandler, invokeHandler, webviewNew, webviewEval } from "runtime:webview";

registerHandler("createBrowser", async (url: string) => {
  const view = await webviewNew({
    title: "Browser",
    url,
    width: 1024,
    height: 768,
    resizable: true,
    debug: false,
    frameless: false
  });
  return view;
});

// Later...
const browser = await invokeHandler("createBrowser", "https://example.com");
```

### listHandlers() / hasHandler(name) / removeHandler(name)

```typescript
import { listHandlers, hasHandler, removeHandler } from "runtime:webview";

const handlers = listHandlers();
console.log("Registered handlers:", handlers);

if (hasHandler("createBrowser")) {
  removeHandler("createBrowser");
}
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 9000 | Generic | General WebView operation error |
| 9001 | PermissionDenied | Permission denied for window operations |

## Function Aliases

The module provides friendly aliases for all functions:

| Primary Name | Alias |
|--------------|-------|
| `webviewNew` | `newWebView` |
| `webviewExit` | `exitWebView` |
| `webviewEval` | `evalInWebView` |
| `webviewSetColor` | `setWebViewColor` |
| `webviewSetTitle` | `setWebViewTitle` |
| `webviewSetFullscreen` | `setWebViewFullscreen` |
| `webviewLoop` | `webViewLoop` |
| `webviewRun` | `runWebView` |

## Complete Example

### Multi-Window Application

```typescript
import {
  webviewNew,
  webviewExit,
  webviewEval,
  webviewSetTitle,
  webviewSetColor
} from "runtime:webview";

interface AppWindow {
  id: string;
  type: "main" | "settings" | "help";
}

const windows: AppWindow[] = [];

async function createMainWindow(): Promise<AppWindow> {
  const view = await webviewNew({
    title: "My Application",
    url: "app://index.html",
    width: 1200,
    height: 800,
    resizable: true,
    debug: false,
    frameless: false
  });

  await webviewSetColor(view.id, 255, 255, 255, 255);

  const window: AppWindow = { id: view.id, type: "main" };
  windows.push(window);
  return window;
}

async function createSettingsWindow(): Promise<AppWindow> {
  const view = await webviewNew({
    title: "Settings",
    url: "app://settings.html",
    width: 600,
    height: 400,
    resizable: false,
    debug: false,
    frameless: false
  });

  const window: AppWindow = { id: view.id, type: "settings" };
  windows.push(window);
  return window;
}

async function createHelpWindow(): Promise<AppWindow> {
  const view = await webviewNew({
    title: "Help",
    url: "https://docs.example.com",
    width: 800,
    height: 600,
    resizable: true,
    debug: false,
    frameless: false
  });

  const window: AppWindow = { id: view.id, type: "help" };
  windows.push(window);
  return window;
}

async function closeWindow(windowId: string): Promise<void> {
  const index = windows.findIndex(w => w.id === windowId);
  if (index !== -1) {
    await webviewExit(windowId);
    windows.splice(index, 1);
  }
}

async function closeAllWindows(): Promise<void> {
  for (const window of [...windows]) {
    await closeWindow(window.id);
  }
}

async function updateWindowTitle(windowId: string, suffix: string): Promise<void> {
  const window = windows.find(w => w.id === windowId);
  if (window) {
    const baseTitle = window.type === "main" ? "My Application" :
                      window.type === "settings" ? "Settings" : "Help";
    await webviewSetTitle(windowId, `${baseTitle} - ${suffix}`);
  }
}

// Application entry point
async function main() {
  // Create main window
  const mainWindow = await createMainWindow();
  console.log("Main window created:", mainWindow.id);

  // Inject initialization script
  await webviewEval(mainWindow.id, `
    console.log('Application initialized');
    window.appVersion = '1.0.0';
  `);

  // Update title with version
  await updateWindowTitle(mainWindow.id, "v1.0.0");
}

main().catch(console.error);
```

### Frameless App with Custom Title Bar

```typescript
import { webviewNew, webviewEval, webviewExit } from "runtime:webview";

async function createFramelessApp() {
  const app = await webviewNew({
    title: "Frameless App",
    url: "about:blank",
    width: 800,
    height: 600,
    resizable: true,
    debug: true,
    frameless: true  // No native title bar
  });

  // Inject custom title bar
  await webviewEval(app.id, `
    document.body.style.margin = '0';
    document.body.style.fontFamily = 'system-ui, sans-serif';

    // Custom title bar
    const titleBar = document.createElement('div');
    titleBar.id = 'titlebar';
    titleBar.style.cssText = \`
      height: 32px;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 12px;
      -webkit-app-region: drag;
      user-select: none;
    \`;

    // Title
    const title = document.createElement('span');
    title.textContent = 'My Frameless App';
    title.style.cssText = 'color: white; font-weight: 500;';
    titleBar.appendChild(title);

    // Close button
    const closeBtn = document.createElement('button');
    closeBtn.textContent = '×';
    closeBtn.style.cssText = \`
      background: none;
      border: none;
      color: white;
      font-size: 20px;
      cursor: pointer;
      -webkit-app-region: no-drag;
      padding: 4px 8px;
      border-radius: 4px;
    \`;
    closeBtn.onmouseover = () => closeBtn.style.background = 'rgba(255,255,255,0.2)';
    closeBtn.onmouseout = () => closeBtn.style.background = 'none';
    titleBar.appendChild(closeBtn);

    // Content area
    const content = document.createElement('div');
    content.id = 'content';
    content.style.cssText = \`
      padding: 20px;
      height: calc(100vh - 72px);
      overflow: auto;
    \`;
    content.innerHTML = '<h1>Welcome to Frameless App</h1><p>This window has a custom title bar.</p>';

    document.body.appendChild(titleBar);
    document.body.appendChild(content);
  `);

  return app;
}

createFramelessApp().catch(console.error);
```

## Best Practices

### Use Appropriate Window Types

```typescript
// Standard app window
await webviewNew({
  title: "My App",
  url: "app://index.html",
  width: 1024,
  height: 768,
  resizable: true,
  debug: false,
  frameless: false
});

// Fixed-size dialog
await webviewNew({
  title: "Preferences",
  url: "app://prefs.html",
  width: 500,
  height: 400,
  resizable: false,
  debug: false,
  frameless: false
});

// Frameless overlay
await webviewNew({
  title: "Widget",
  url: "app://widget.html",
  width: 300,
  height: 200,
  resizable: false,
  debug: false,
  frameless: true
});
```

### Enable Debug Mode During Development

```typescript
const isDev = process.env.NODE_ENV === "development";

await webviewNew({
  title: "My App",
  url: "app://index.html",
  width: 1024,
  height: 768,
  resizable: true,
  debug: isDev,  // Enable DevTools in development
  frameless: false
});
```

### Clean Up Windows

```typescript
const activeWindows: string[] = [];

async function createWindow(): Promise<string> {
  const view = await webviewNew({ /* options */ });
  activeWindows.push(view.id);
  return view.id;
}

async function cleanup(): Promise<void> {
  for (const id of activeWindows) {
    try {
      await webviewExit(id);
    } catch (error) {
      // Window may already be closed
    }
  }
  activeWindows.length = 0;
}
```

## See Also

- [runtime:window](/docs/api/runtime-window) - Full window management API
- [ext_webview](/docs/crates/ext-webview) - Implementation details
- [ext_window](/docs/crates/ext-window) - Underlying window system
