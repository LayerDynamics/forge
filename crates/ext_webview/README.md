# ext_webview

Lightweight WebView creation and management extension for Forge runtime.

## Overview

`ext_webview` provides a simple API for creating and controlling WebView windows, built as a wrapper around the [ext_window](../ext_window/) runtime. This extension offers a streamlined interface for common WebView operations without requiring direct window management.

**Runtime Module:** `runtime:webview`

## Features

### WebView Creation
- Create WebView windows with customizable dimensions and behavior
- Configure title, URL, size, resizable state
- Support for frameless windows and debug mode
- Automatic window management integration

### WebView Control
- Execute JavaScript code in the WebView context
- Set window title and background color dynamically
- Toggle fullscreen mode
- Close WebView windows

### Event Loop Integration
- Centralized event loop (loop and run operations are no-ops)
- All WebView events handled by Forge's main event loop
- No manual event loop management required

## Usage

### Basic WebView Creation

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

// Execute JavaScript in the WebView
await webviewEval(webview.id, "console.log('Hello from WebView!')");

// Close when done
await webviewExit(webview.id);
```

### Browser-Like Window

```typescript
import { webviewNew } from "runtime:webview";

const browser = await webviewNew({
  title: "Web Browser",
  url: "about:blank",
  width: 1024,
  height: 768,
  resizable: true,
  debug: true,  // Enable DevTools
  frameless: false
});
```

### Frameless Application Window

```typescript
import { webviewNew } from "runtime:webview";

const app = await webviewNew({
  title: "Frameless App",
  url: "app://index.html",
  width: 600,
  height: 400,
  resizable: false,
  debug: false,
  frameless: true  // No title bar or borders
});
```

### Dynamic Content Injection

```typescript
import { webviewNew, webviewEval, webviewSetColor } from "runtime:webview";

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

// Style the page
await webviewSetColor(view.id, 240, 240, 255, 255);  // Light blue
```

### Window Title Management

```typescript
import { webviewNew, webviewSetTitle } from "runtime:webview";

const view = await webviewNew({
  title: "Initial Title",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Update title dynamically
await webviewSetTitle(view.id, "Updated Title - Page Loaded");
```

### Fullscreen Mode

```typescript
import { webviewNew, webviewSetFullscreen } from "runtime:webview";

const view = await webviewNew({
  title: "Example",
  url: "https://example.com",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,
  frameless: false
});

// Enter fullscreen mode
await webviewSetFullscreen(view.id, true);

// Exit fullscreen mode
await webviewSetFullscreen(view.id, false);
```

## Architecture

ext_webview is built as a lightweight wrapper around runtime:window:

```text
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

All WebView operations are translated to window commands and sent through ext_window's command channel. This ensures consistent behavior and centralized window management.

### Operation Mapping

| WebView Operation | Window Command | Purpose |
|-------------------|---------------|---------|
| `op_host_webview_new` | `WindowCmd::Create` | Create new WebView window |
| `op_host_webview_exit` | `WindowCmd::Close` | Close WebView window |
| `op_host_webview_eval` | `WindowCmd::EvalJs` | Execute JavaScript in WebView |
| `op_host_webview_set_color` | `WindowCmd::InjectCss` | Set background color |
| `op_host_webview_set_title` | `WindowCmd::SetTitle` | Update window title |
| `op_host_webview_set_fullscreen` | `WindowCmd::SetFullscreen` | Toggle fullscreen |
| `op_host_webview_loop` | *No-op* | Event loop compatibility shim |
| `op_host_webview_run` | *No-op* | Run loop compatibility shim |

## Error Handling

All operations return structured errors with machine-readable error codes (9000-9001).

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 9000 | Generic | General WebView operation error |
| 9001 | PermissionDenied | Permission denied for window operations |

### Error Handling Patterns

```typescript
import { webviewNew, webviewEval } from "runtime:webview";

// Handle creation errors
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

// Handle eval errors
try {
  await webviewEval(webview.id, "invalid javascript code");
} catch (error) {
  // Error 9000: Script evaluation failed
  console.error("Failed to execute script:", error);
}
```

## Permission Model

WebView operations require window creation permissions as defined in your app's `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for WebView operations
```

Operations will fail with error 9001 if permissions are not granted.

## Implementation Details

### Window Creation

`op_host_webview_new` converts `WebViewNewParams` to `WindowOpts` and sends a `WindowCmd::Create` command:

1. Check permissions via `check_window_caps()`
2. Convert parameters (frameless -> !decorations, debug -> devtools)
3. Send `WindowCmd::Create` to ext_window command channel
4. Await response with window ID
5. Return `WebViewNewResult` containing window ID

### JavaScript Evaluation

`op_host_webview_eval` sends the JavaScript code through the window command channel:

1. Check permissions
2. Send `WindowCmd::EvalJs` with window ID and script
3. Await completion (no return value captured)

The JavaScript code runs asynchronously in the WebView. Return values are not captured - use this for side effects only (DOM manipulation, logging, etc.).

### Background Color Setting

`op_host_webview_set_color` uses CSS injection to set the body background:

1. Convert RGBA values to CSS rgba() format (alpha: 0-255 -> 0.0-1.0)
2. Generate CSS rule: `body { background-color: rgba(...); }`
3. Send `WindowCmd::InjectCss` to inject the rule

Example CSS generated:
```css
body { background-color: rgba(240,240,255,1.000); }
```

### Event Loop No-Ops

`op_host_webview_loop` and `op_host_webview_run` exist for API compatibility with reference WebView plugins. They perform no operation in Forge:

- **webviewLoop**: Validates window ID, returns `{ code: 0 }` immediately
- **webviewRun**: Returns immediately with success

Forge uses a centralized event loop that handles all window and WebView events automatically, eliminating the need for manual event loop management.

## Platform Support

| Platform | WebView Backend | Status |
|----------|----------------|--------|
| macOS (x64) | WebKit (WKWebView) | ✅ Full support |
| macOS (ARM) | WebKit (WKWebView) | ✅ Full support |
| Windows (x64) | WebView2 (Edge) | ✅ Full support |
| Windows (ARM) | WebView2 (Edge) | ✅ Full support |
| Linux (x64) | WebKitGTK | ✅ Full support |
| Linux (ARM) | WebKitGTK | ✅ Full support |

Platform-specific behavior is handled by the underlying [wry](https://docs.rs/wry) crate.

## Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `deno_core` | 0.373 | Op definitions and runtime integration |
| `ext_window` | 0.1.0-alpha.1 | Window management and command channel |
| `tokio` | 1.x | Async oneshot channels for command responses |
| `serde` | 1.x | Serialization framework |
| `thiserror` | 2.x | Error type definitions |
| `deno_error` | 0.x | JavaScript error conversion |
| `forge-weld-macro` | 0.1 | TypeScript binding generation |
| `forge-weld` | 0.1 | Build-time code generation |
| `linkme` | 0.3 | Compile-time symbol collection |

## Testing

```bash
# Run all tests
cargo test -p ext_webview

# Run with output
cargo test -p ext_webview -- --nocapture

# Run specific test
cargo test -p ext_webview test_webview_creation

# With debug logging
RUST_LOG=ext_webview=debug cargo test -p ext_webview -- --nocapture
```

## Common Pitfalls

### 1. Using Invalid Window IDs

```typescript
// ❌ ERROR: Using ID after window closed
await webviewExit(webview.id);
await webviewEval(webview.id, "..."); // ID is now invalid

// ✅ CORRECT: Don't use ID after closing
await webviewEval(webview.id, "...");
await webviewExit(webview.id);  // Close last
```

### 2. Expecting JavaScript Return Values

```typescript
// ❌ INCORRECT: webviewEval doesn't return script results
const result = await webviewEval(webview.id, "2 + 2"); // undefined

// ✅ CORRECT: Use for side effects only
await webviewEval(webview.id, `
  console.log(2 + 2);  // Log the result
  document.title = "Result: " + (2 + 2);  // Store in DOM
`);
```

### 3. Missing Permissions

```typescript
// ❌ Will fail with error 9001 if permissions not granted in manifest.app.toml

// ✅ CORRECT: Ensure manifest.app.toml has:
// [permissions.ui]
// windows = true
```

### 4. Manual Event Loop Management

```typescript
// ❌ UNNECESSARY: These are no-ops in Forge
await webviewLoop(webview.id, 0);
await webviewRun(webview.id);

// ✅ CORRECT: Forge handles event loop automatically
// Just create your WebView and it works:
const webview = await webviewNew({ ... });
```

## See Also

- [ext_window](../ext_window/) - Window management extension
- [ext_ipc](../ext_ipc/) - IPC communication extension
- [ext_devtools](../ext_devtools/) - Developer tools extension
- [wry documentation](https://docs.rs/wry) - WebView rendering library
- [tao documentation](https://docs.rs/tao) - Cross-platform window creation
- [Forge Documentation](../../site/) - Full framework documentation

## License

Part of the Forge project. See the repository root for license information.
