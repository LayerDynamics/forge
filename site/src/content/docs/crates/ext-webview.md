---
title: "ext_webview"
description: Lightweight WebView creation and management extension for the Forge runtime.
slug: crates/ext-webview
---

The `ext_webview` crate provides a simple API for creating and controlling WebView windows through the `runtime:webview` module. Built as a lightweight wrapper around [ext_window](/docs/crates/ext-window), it offers a streamlined interface for common WebView operations.

## Quick Start

```typescript
import { webviewNew, webviewEval, webviewExit } from "runtime:webview";

// Create a WebView window
const webview = await webviewNew({
  title: "My App",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Execute JavaScript
await webviewEval(webview.id, "console.log('Hello!')");

// Close when done
await webviewExit(webview.id);
```

## API Reference

### Creating WebViews

#### `webviewNew(options)`

Creates a new WebView window with the specified configuration.

**Parameters:**
- `title` (string) - Window title displayed in title bar
- `url` (string) - Initial URL to load (http://, https://, file://, app://)
- `width` (number) - Window width in pixels
- `height` (number) - Window height in pixels
- `resizable` (boolean) - Allow user to resize window
- `debug` (boolean) - Enable DevTools for debugging
- `frameless` (boolean) - Remove window decorations (title bar, borders)

**Returns:** `WebViewHandle` containing the window ID

**Throws:**
- Error [9000] if window creation fails
- Error [9001] if permission denied

```typescript
const webview = await webviewNew({
  title: "Web Browser",
  url: "about:blank",
  width: 1024,
  height: 768,
  resizable: true,
  debug: true,  // Enable DevTools
  frameless: false
});
```

**Alias:** `newWebView()`

### Controlling WebViews

#### `webviewExit(id)`

Closes the WebView window. The window ID becomes invalid after closing.

```typescript
await webviewExit(webview.id);
```

**Alias:** `exitWebView()`

#### `webviewEval(id, javascript)`

Executes JavaScript code in the WebView context. Return values are not captured - use for side effects only.

```typescript
// Inject content
await webviewEval(webview.id, `
  document.body.innerHTML = '<h1>Hello, World!</h1>';
`);

// Add styles
await webviewEval(webview.id, `
  document.body.style.backgroundColor = '#f0f0f0';
`);
```

**Alias:** `evalInWebView()`

#### `webviewSetTitle(id, title)`

Updates the window title. For frameless windows, the title may not be visible but is still used by the OS.

```typescript
await webviewSetTitle(webview.id, "Updated Title");
```

**Alias:** `setWebViewTitle()`

#### `webviewSetColor(id, r, g, b, a)`

Sets the WebView background color using RGBA values (0-255 for each channel).

```typescript
// Light blue background
await webviewSetColor(webview.id, 240, 240, 255, 255);

// Semi-transparent white
await webviewSetColor(webview.id, 255, 255, 255, 128);
```

**Alias:** `setWebViewColor()`

#### `webviewSetFullscreen(id, fullscreen)`

Toggles fullscreen mode. In fullscreen, the window occupies the entire screen with all decorations hidden.

```typescript
// Enter fullscreen
await webviewSetFullscreen(webview.id, true);

// Exit fullscreen
await webviewSetFullscreen(webview.id, false);
```

**Alias:** `setWebViewFullscreen()`

### Event Loop Compatibility

#### `webviewLoop(id, blocking)`

Event loop shim (no-op in Forge). Exists for API compatibility with reference WebView plugins. Always returns `{ code: 0 }`.

```typescript
const result = await webviewLoop(webview.id, 0);
// Always returns { code: 0 }
```

**Alias:** `webViewLoop()`

#### `webviewRun(id)`

Run loop shim (no-op in Forge). Exists for API compatibility. Returns immediately.

```typescript
await webviewRun(webview.id);
```

**Alias:** `runWebView()`

## Common Patterns

### Browser-Like Window

```typescript
const browser = await webviewNew({
  title: "Web Browser",
  url: "about:blank",
  width: 1024,
  height: 768,
  resizable: true,
  debug: true,  // Enable DevTools
  frameless: false
});

// Users can navigate using browser controls
```

### Frameless Application Window

```typescript
const app = await webviewNew({
  title: "My App",
  url: "app://index.html",
  width: 600,
  height: 400,
  resizable: false,
  debug: false,
  frameless: true  // No title bar or borders
});

// Custom UI for close/minimize buttons via JavaScript
```

### Dynamic Content Injection

```typescript
const view = await webviewNew({
  title: "Dynamic Content",
  url: "about:blank",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Inject HTML
await webviewEval(view.id, `
  document.body.innerHTML = '<h1>Hello, World!</h1>';
`);

// Style the page
await webviewSetColor(view.id, 240, 240, 255, 255);
```

### Title Updates

```typescript
const view = await webviewNew({
  title: "Initial Title",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Update based on content
await webviewSetTitle(view.id, `Viewing: ${currentUrl}`);
```

## Architecture

ext_webview is a lightweight wrapper around ext_window:

```
TypeScript Application
  ↓
runtime:webview
  ↓ (translates to WindowCmd messages)
runtime:window
  ↓ (wry/tao)
Native Window System
```

All WebView operations are translated to window commands:

| WebView Operation | Window Command | Description |
|-------------------|---------------|-------------|
| `webviewNew()` | `WindowCmd::Create` | Create window |
| `webviewExit()` | `WindowCmd::Close` | Close window |
| `webviewEval()` | `WindowCmd::EvalJs` | Execute JavaScript |
| `webviewSetColor()` | `WindowCmd::InjectCss` | Inject CSS for background color |
| `webviewSetTitle()` | `WindowCmd::SetTitle` | Update title |
| `webviewSetFullscreen()` | `WindowCmd::SetFullscreen` | Toggle fullscreen |

## Error Handling

All operations use structured error codes:

| Code | Error | Description |
|------|-------|-------------|
| 9000 | Generic | General WebView operation failure |
| 9001 | PermissionDenied | Window creation permission denied |

```typescript
try {
  const webview = await webviewNew({
    title: "Example",
    url: "https://example.com",
    width: 800,
    height: 600,
    resizable: true,
    debug: false,
    frameless: false
  });
} catch (error) {
  // Error 9000: Creation failed
  // Error 9001: Permission denied
  console.error("Failed to create WebView:", error);
}
```

## Permissions

WebView operations require window creation permissions in `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for WebView operations
```

Operations fail with error 9001 if permissions are not granted.

## Platform Support

| Platform | WebView Backend | Status |
|----------|----------------|--------|
| macOS | WebKit (WKWebView) | ✅ Full support |
| Windows | WebView2 (Edge) | ✅ Full support |
| Linux | WebKitGTK | ✅ Full support |

Platform-specific behavior is handled by the underlying [wry](https://docs.rs/wry) crate.

## Common Pitfalls

### Using Invalid Window IDs

```typescript
// ❌ ERROR: Using ID after window closed
await webviewExit(webview.id);
await webviewEval(webview.id, "..."); // ID is now invalid

// ✅ CORRECT: Don't use ID after closing
await webviewEval(webview.id, "...");
await webviewExit(webview.id);  // Close last
```

### Expecting JavaScript Return Values

```typescript
// ❌ INCORRECT: webviewEval doesn't return results
const result = await webviewEval(webview.id, "2 + 2"); // undefined

// ✅ CORRECT: Use for side effects only
await webviewEval(webview.id, `
  console.log(2 + 2);  // Log the result
`);
```

### Manual Event Loop Management

```typescript
// ❌ UNNECESSARY: These are no-ops in Forge
await webviewLoop(webview.id, 0);
await webviewRun(webview.id);

// ✅ CORRECT: Forge handles event loop automatically
const webview = await webviewNew({ ... });
```

## Implementation Details

### Window Creation

`webviewNew()` converts parameters to `WindowOpts` and sends `WindowCmd::Create`:

1. Check permissions via `WindowCapabilities`
2. Convert parameters (frameless -> !decorations, debug -> devtools)
3. Send `WindowCmd::Create` to ext_window command channel
4. Await response with window ID
5. Return handle containing window ID

### JavaScript Evaluation

`webviewEval()` sends JavaScript through the window command channel:

1. Check permissions
2. Send `WindowCmd::EvalJs` with window ID and script
3. Await completion (no return value captured)

JavaScript runs asynchronously in the WebView context. Use for side effects only.

### Background Color

`webviewSetColor()` uses CSS injection:

1. Convert RGBA (0-255) to CSS rgba() format
2. Generate: `body { background-color: rgba(r,g,b,a); }`
3. Send `WindowCmd::InjectCss` to inject the rule

### Event Loop

`webviewLoop()` and `webviewRun()` are no-ops for API compatibility. Forge uses a centralized event loop that handles all window events automatically.

## Testing

```bash
# Run tests
cargo test -p ext_webview

# With debug logging
RUST_LOG=ext_webview=debug cargo test -p ext_webview -- --nocapture
```

## Related

- [ext_window](/docs/crates/ext-window) - Window management extension
- [ext_ipc](/docs/crates/ext-ipc) - IPC communication
- [ext_devtools](/docs/crates/ext-devtools) - Developer tools
- [Architecture](/docs/architecture) - System architecture
