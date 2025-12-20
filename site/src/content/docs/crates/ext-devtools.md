---
title: "ext_devtools"
description: Developer tools control extension for the Forge runtime.
slug: crates/ext-devtools
---

The `ext_devtools` crate provides a simple API for controlling browser DevTools through the `runtime:devtools` module. Built as a lightweight wrapper around [ext_window](/docs/crates/ext-window), it offers programmatic control over the DevTools panel used for debugging WebView windows.

## Quick Start

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

## API Reference

### DevTools Control

#### `open(windowId)`

Opens the DevTools panel for the specified window. The DevTools panel appears as a separate docked panel or window depending on the platform and WebView implementation.

This operation is translated to `WindowCmd::OpenDevTools` and sent through the ext_window command channel.

**Parameters:**
- `windowId` (string) - Window ID from ext_window or ext_webview

**Returns:** `Promise<boolean>` - true on success

**Throws:**
- Error [9100] if DevTools open fails
- Error [9101] if permission denied for window operations

```typescript
import { open } from "runtime:devtools";
import { webviewNew } from "runtime:webview";

// Create window with debug mode enabled
const window = await webviewNew({
  title: "My App",
  url: "app://index.html",
  width: 1200,
  height: 800,
  resizable: true,
  debug: true,  // DevTools available
  frameless: false
});

// Open DevTools programmatically
await open(window.id);
```

**Development Mode Example:**

```typescript
import { open } from "runtime:devtools";

// Open DevTools only in development mode
const isDev = Deno.env.get("MODE") === "development";
if (isDev) {
  await open(windowId);
}
```

#### `close(windowId)`

Closes the DevTools panel for the specified window. If the DevTools are already closed, this operation succeeds without error.

This operation is translated to `WindowCmd::CloseDevTools` and sent through the ext_window command channel.

**Parameters:**
- `windowId` (string) - Window ID from ext_window or ext_webview

**Returns:** `Promise<boolean>` - true on success

**Throws:**
- Error [9100] if DevTools close fails
- Error [9101] if permission denied for window operations

```typescript
import { open, close } from "runtime:devtools";

// Open DevTools for debugging
await open(windowId);

// User completes debugging...

// Close DevTools to reclaim screen space
await close(windowId);
```

**Conditional Close:**

```typescript
import { close, isOpen } from "runtime:devtools";

// Close DevTools if open
if (await isOpen(windowId)) {
  await close(windowId);
}
```

#### `isOpen(windowId)`

Checks if the DevTools panel is currently open for the specified window. Returns true if DevTools are open, false otherwise.

This operation is translated to `WindowCmd::IsDevToolsOpen` and sent through the ext_window command channel.

**Parameters:**
- `windowId` (string) - Window ID from ext_window or ext_webview

**Returns:** `Promise<boolean>` - true if DevTools are open, false otherwise

**Throws:**
- Error [9100] if state query fails
- Error [9101] if permission denied for window operations

```typescript
import { isOpen } from "runtime:devtools";

// Check DevTools state
const devToolsOpen = await isOpen(windowId);
console.log("DevTools open:", devToolsOpen);
```

**UI State Example:**

```typescript
import { isOpen } from "runtime:devtools";

// Conditional UI state based on DevTools
const devToolsButton = document.getElementById("devtools-toggle");
if (await isOpen(windowId)) {
  devToolsButton.textContent = "Close DevTools";
  devToolsButton.classList.add("active");
} else {
  devToolsButton.textContent = "Open DevTools";
  devToolsButton.classList.remove("active");
}
```

## Common Patterns

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

// Use with keyboard shortcut or button
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

### Conditional DevTools (Development Mode)

```typescript
import { open } from "runtime:devtools";

const isDev = Deno.env.get("MODE") === "development";

if (isDev) {
  await open(windowId);
}
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

### DevTools with Event Handler

```typescript
import { open, close, isOpen } from "runtime:devtools";

// Button to toggle DevTools
document.getElementById("devtools-toggle")?.addEventListener("click", async () => {
  const currentState = await isOpen(windowId);
  if (currentState) {
    await close(windowId);
  } else {
    await open(windowId);
  }
});
```

## Architecture

ext_devtools is a thin wrapper around ext_window:

```
TypeScript Application
  ↓
runtime:devtools (open, close, isOpen)
  ↓ (translates to WindowCmd messages)
runtime:window
  ↓ (wry DevTools API)
Native WebView DevTools
```

All DevTools operations are translated to window commands:

| DevTools Operation | Window Command | Description |
|-------------------|---------------|-------------|
| `open()` | `WindowCmd::OpenDevTools` | Open DevTools panel |
| `close()` | `WindowCmd::CloseDevTools` | Close DevTools panel |
| `isOpen()` | `WindowCmd::IsDevToolsOpen` | Check DevTools state |

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

## Error Handling

All operations use structured error codes:

| Code | Error | Description |
|------|-------|-------------|
| 9100 | Generic | General DevTools operation error |
| 9101 | PermissionDenied | Window management permission denied |

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

## Permissions

DevTools operations require window management permissions in `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for DevTools operations
```

Operations fail with error 9101 if permissions are not granted.

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

## Common Pitfalls

### 1. Using Invalid Window IDs

```typescript
// ❌ ERROR: Using ID of destroyed window
await windowExit(windowId);
await open(windowId); // Window no longer exists

// ✅ CORRECT: Only use DevTools with valid windows
await open(windowId);
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
await open(window.id); // May fail or have no effect

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
await open(window.id); // Works
```

### 3. Not Checking State Before Toggle

```typescript
// ❌ RISKY: Assuming current state
await close(windowId); // What if already closed?

// ✅ CORRECT: Check state first
if (await isOpen(windowId)) {
  await close(windowId);
}

// ✅ BETTER: Use toggle pattern
async function toggleDevTools(windowId: string) {
  if (await isOpen(windowId)) {
    await close(windowId);
  } else {
    await open(windowId);
  }
}
```

## Related

- [ext_window](/docs/crates/ext-window) - Window management extension
- [ext_webview](/docs/crates/ext-webview) - WebView creation extension
- [ext_shortcuts](/docs/crates/ext-shortcuts) - Keyboard shortcuts
- [Architecture](/docs/architecture) - System architecture
