# ext_devtools

Developer tools control extension for Forge runtime.

## Overview

`ext_devtools` provides a simple API for opening, closing, and checking the state of browser DevTools for WebView windows. Built as a thin wrapper around the [ext_window](../ext_window/) runtime, it offers programmatic control over the DevTools panel that developers use for debugging and inspecting web content.

**Runtime Module:** `runtime:devtools`

## Features

### DevTools Control
- Open DevTools panel for any window
- Close DevTools panel programmatically
- Check if DevTools are currently open
- Simple boolean return values for all operations

### Window Integration
- Works with any window created via ext_window or ext_webview
- DevTools state persists until explicitly closed or window destroyed
- Multiple windows can have DevTools open simultaneously

## Usage

### Basic DevTools Control

```typescript
import { open, close, isOpen } from "runtime:devtools";
import { webviewNew } from "runtime:webview";

// Create window with DevTools available
const window = await webviewNew({
  title: "Debug Window",
  url: "app://index.html",
  width: 1200,
  height: 800,
  resizable: true,
  debug: true,  // DevTools available
  frameless: false
});

// Open DevTools programmatically
await open(window.id);

// Check state
const devToolsOpen = await isOpen(window.id);
console.log("DevTools open:", devToolsOpen); // true

// Close when done
await close(window.id);
```

### Conditional DevTools (Development Mode)

```typescript
import { open } from "runtime:devtools";

const isDev = Deno.env.get("MODE") === "development";

if (isDev) {
  await open(windowId);
}
```

### Toggle DevTools

```typescript
import { open, close, isOpen } from "runtime:devtools";

async function toggleDevTools(windowId: string) {
  if (await isOpen(windowId)) {
    await close(windowId);
    console.log("DevTools closed");
  } else {
    await open(windowId);
    console.log("DevTools opened");
  }
}

// Use with keyboard shortcut
await toggleDevTools(windowId);
```

### DevTools Keyboard Shortcut

```typescript
import { open, close, isOpen } from "runtime:devtools";
import { on } from "runtime:shortcuts";

// F12 to toggle DevTools
await on("F12", async () => {
  if (await isOpen(currentWindowId)) {
    await close(currentWindowId);
  } else {
    await open(currentWindowId);
  }
});
```

### UI State Based on DevTools

```typescript
import { isOpen } from "runtime:devtools";

// Update UI to reflect DevTools state
const devToolsButton = document.getElementById("devtools-toggle");

if (await isOpen(windowId)) {
  devToolsButton.textContent = "Close DevTools";
  devToolsButton.classList.add("active");
} else {
  devToolsButton.textContent = "Open DevTools";
  devToolsButton.classList.remove("active");
}
```

## Architecture

ext_devtools is a thin wrapper around ext_window:

```text
TypeScript Application
  |
  | open(), close(), isOpen()
  v
runtime:devtools (ext_devtools)
  |
  | WindowCmd::OpenDevTools, WindowCmd::CloseDevTools, WindowCmd::IsDevToolsOpen
  v
runtime:window (ext_window)
  |
  | wry DevTools API
  v
Native WebView DevTools
```

All DevTools operations are translated to window commands:

| DevTools Operation | Window Command | Description |
|-------------------|---------------|-------------|
| `open()` | `WindowCmd::OpenDevTools` | Open DevTools panel |
| `close()` | `WindowCmd::CloseDevTools` | Close DevTools panel |
| `isOpen()` | `WindowCmd::IsDevToolsOpen` | Check DevTools state |

## Error Handling

All operations return structured errors with machine-readable error codes (9100-9101).

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 9100 | Generic | General DevTools operation error |
| 9101 | PermissionDenied | Window management permission denied |

### Error Handling Patterns

```typescript
import { open, close, isOpen } from "runtime:devtools";

// Handle open errors
try {
  await open(windowId);
} catch (error) {
  // Error 9100: Operation failed
  // Error 9101: Permission denied
  console.error("Failed to open DevTools:", error);
}

// Handle state query errors
try {
  const devToolsOpen = await isOpen(windowId);
} catch (error) {
  // Error 9100: Query failed
  console.error("Failed to check DevTools state:", error);
}
```

## Permission Model

DevTools operations require window management permissions in `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for DevTools operations
```

Operations fail with error 9101 if permissions are not granted.

## Implementation Details

### Opening DevTools

`open()` sends `WindowCmd::OpenDevTools` through the window command channel:

1. Check permissions via `check_window_caps()`
2. Send `WindowCmd::OpenDevTools` to ext_window command channel
3. Await response confirmation
4. Return `true` on success

### Closing DevTools

`close()` sends `WindowCmd::CloseDevTools`:

1. Check permissions
2. Send `WindowCmd::CloseDevTools` with window ID
3. Await response confirmation
4. Return `true` on success

### Checking State

`isOpen()` queries the DevTools state:

1. Check permissions
2. Send `WindowCmd::IsDevToolsOpen` with window ID
3. Await boolean response from window manager
4. Return DevTools open state (defaults to `false` if query fails)

## Platform Support

| Platform | DevTools Backend | Status |
|----------|-----------------|--------|
| macOS (x64) | WebKit Inspector | ✅ Full support |
| macOS (ARM) | WebKit Inspector | ✅ Full support |
| Windows (x64) | Edge DevTools (F12) | ✅ Full support |
| Windows (ARM) | Edge DevTools (F12) | ✅ Full support |
| Linux (x64) | WebKit Inspector | ✅ Full support |
| Linux (ARM) | WebKit Inspector | ✅ Full support |

Platform-specific DevTools behavior is handled by the underlying [wry](https://docs.rs/wry) crate.

## Dependencies

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `deno_core` | 0.373 | Op definitions and runtime integration |
| `ext_window` | 0.1.0-alpha.1 | Window management and command channel |
| `tokio` | 1.x | Async oneshot channels for command responses |
| `thiserror` | 2.x | Error type definitions |
| `deno_error` | 0.x | JavaScript error conversion |
| `forge-weld-macro` | 0.1 | TypeScript binding generation |
| `forge-weld` | 0.1 | Build-time code generation |
| `linkme` | 0.3 | Compile-time symbol collection |

## Testing

```bash
# Run all tests
cargo test -p ext_devtools

# Run with output
cargo test -p ext_devtools -- --nocapture

# Run specific test
cargo test -p ext_devtools test_devtools_open

# With debug logging
RUST_LOG=ext_devtools=debug cargo test -p ext_devtools -- --nocapture
```

## Common Pitfalls

### 1. Using Invalid Window IDs

```typescript
// ❌ ERROR: Using ID of destroyed window
await windowExit(windowId);
await devtools.open(windowId); // Window no longer exists

// ✅ CORRECT: Only use DevTools with valid windows
await devtools.open(windowId);
// ... later ...
await windowExit(windowId);  // Close window last
```

### 2. Missing Debug Flag

```typescript
// ❌ INCORRECT: DevTools may not be available
const window = await webviewNew({
  title: "App",
  url: "app://index.html",
  width: 800,
  height: 600,
  resizable: true,
  debug: false,  // DevTools disabled!
  frameless: false
});
await devtools.open(window.id); // May fail or have no effect

// ✅ CORRECT: Enable debug mode for DevTools
const window = await webviewNew({
  title: "App",
  url: "app://index.html",
  width: 800,
  height: 600,
  resizable: true,
  debug: true,  // DevTools available
  frameless: false
});
await devtools.open(window.id); // Works
```

### 3. Not Checking State Before Toggle

```typescript
// ❌ RISKY: Assuming current state
await devtools.close(windowId); // What if already closed?

// ✅ CORRECT: Check state first
if (await devtools.isOpen(windowId)) {
  await devtools.close(windowId);
}

// ✅ BETTER: Use toggle pattern
async function toggleDevTools(windowId: string) {
  if (await devtools.isOpen(windowId)) {
    await devtools.close(windowId);
  } else {
    await devtools.open(windowId);
  }
}
```

## See Also

- [ext_window](../ext_window/) - Window management extension
- [ext_webview](../ext_webview/) - WebView creation extension
- [wry documentation](https://docs.rs/wry) - WebView rendering library
- [Forge Documentation](../../site/) - Full framework documentation

## License

Part of the Forge project. See the repository root for license information.
