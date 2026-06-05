---
title: "runtime:devtools"
description: "Browser DevTools control for debugging Forge WebView windows"
slug: api/runtime-devtools
---

Browser DevTools control extension for Forge applications. Open, close, and check the state of DevTools panels for WebView windows, enabling programmatic control over the debugging experience.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_devtools](/docs/crates/ext-devtools) for implementation details.

## Features

- Open DevTools panel for any window
- Close DevTools panel programmatically
- Check if DevTools are currently open
- Works with any window created via `runtime:window` or `runtime:webview`
- Multiple windows can have DevTools open simultaneously
- DevTools state persists until explicitly closed or window destroyed

## Architecture

```
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

`ext_devtools` is a thin wrapper around `ext_window`, translating DevTools operations to window commands that are sent through the window management system.

## Import

```typescript
import {
  // Functions
  open,
  close,
  isOpen,
  // Hooks
  onBefore,
  onAfter,
  onError,
} from "runtime:devtools";
```

## API Reference

### open(windowId)

Open the DevTools panel for a window.

Opens the browser DevTools (inspector, console, network monitor, etc.) for the specified window. The DevTools panel appears as a separate docked panel or window depending on the platform and WebView implementation.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `windowId` | `string` | Window ID from `runtime:window` or `runtime:webview` |

**Returns:** `Promise<boolean>` - Resolves to `true` on success

**Throws:**

| Code | Error | Description |
|------|-------|-------------|
| 9100 | Generic | DevTools open operation failed |
| 9101 | PermissionDenied | Permission denied for window operations |

**Example:**

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

---

### close(windowId)

Close the DevTools panel for a window.

Closes the browser DevTools panel if it is currently open for the specified window. If the DevTools are already closed, this operation succeeds without error.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `windowId` | `string` | Window ID from `runtime:window` or `runtime:webview` |

**Returns:** `Promise<boolean>` - Resolves to `true` on success

**Throws:**

| Code | Error | Description |
|------|-------|-------------|
| 9100 | Generic | DevTools close operation failed |
| 9101 | PermissionDenied | Permission denied for window operations |

**Example:**

```typescript
import { open, close } from "runtime:devtools";

// Open DevTools for debugging
await open(windowId);

// User completes debugging...

// Close DevTools to reclaim screen space
await close(windowId);
```

---

### isOpen(windowId)

Check if the DevTools panel is currently open for a window.

Queries the current state of the DevTools panel for the specified window.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `windowId` | `string` | Window ID from `runtime:window` or `runtime:webview` |

**Returns:** `Promise<boolean>` - Resolves to `true` if DevTools are open, `false` otherwise

**Throws:**

| Code | Error | Description |
|------|-------|-------------|
| 9100 | Generic | State query failed |
| 9101 | PermissionDenied | Permission denied for window operations |

**Example:**

```typescript
import { isOpen } from "runtime:devtools";

// Check DevTools state
const devToolsOpen = await isOpen(windowId);
console.log("DevTools open:", devToolsOpen);
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 9100 | Generic | General DevTools operation error |
| 9101 | PermissionDenied | Permission denied for window operations |

## Permission Model

DevTools operations require window management permissions as defined in your app's `manifest.app.toml`:

```toml
[permissions.ui]
windows = true  # Required for DevTools operations
```

Operations will fail with error 9101 if permissions are not granted.

## Lifecycle Hooks

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:devtools";

const unsubscribe = onBefore("open", () => {
  console.log("Opening DevTools...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:devtools";

onAfter("open", () => {
  console.log("DevTools opened");
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:devtools";

onError("open", (error) => {
  console.error("Failed to open DevTools:", error.message);
});
```

**Available operation names:** `"open"`, `"close"`, `"isOpen"`

## Complete Examples

### Toggle DevTools

```typescript
import { open, close, isOpen } from "runtime:devtools";

async function toggleDevTools(windowId: string): Promise<boolean> {
  if (await isOpen(windowId)) {
    await close(windowId);
    console.log("DevTools closed");
    return false;
  } else {
    await open(windowId);
    console.log("DevTools opened");
    return true;
  }
}

// Usage
const nowOpen = await toggleDevTools(mainWindow.id);
```

### Development Mode Auto-Open

```typescript
import { open } from "runtime:devtools";
import { webviewNew } from "runtime:webview";

const isDev = Deno.env.get("FORGE_ENV") !== "production";

const window = await webviewNew({
  title: "My App",
  url: "app://index.html",
  width: 1200,
  height: 800,
  resizable: true,
  debug: isDev,
  frameless: false
});

// Auto-open DevTools in development
if (isDev) {
  await open(window.id);
}
```

### Keyboard Shortcut Toggle

```typescript
import { open, close, isOpen } from "runtime:devtools";
import { on } from "runtime:shortcuts";

let currentWindowId: string;

// F12 to toggle DevTools (common convention)
await on("F12", async () => {
  if (await isOpen(currentWindowId)) {
    await close(currentWindowId);
  } else {
    await open(currentWindowId);
  }
});

// Cmd/Ctrl+Shift+I alternative
await on("CmdOrCtrl+Shift+I", async () => {
  if (await isOpen(currentWindowId)) {
    await close(currentWindowId);
  } else {
    await open(currentWindowId);
  }
});
```

### DevTools Manager Class

```typescript
import { open, close, isOpen } from "runtime:devtools";

class DevToolsManager {
  private windows = new Map<string, boolean>();

  async open(windowId: string): Promise<void> {
    await open(windowId);
    this.windows.set(windowId, true);
  }

  async close(windowId: string): Promise<void> {
    await close(windowId);
    this.windows.set(windowId, false);
  }

  async toggle(windowId: string): Promise<boolean> {
    const currentState = await isOpen(windowId);
    if (currentState) {
      await this.close(windowId);
      return false;
    } else {
      await this.open(windowId);
      return true;
    }
  }

  async isOpen(windowId: string): Promise<boolean> {
    return await isOpen(windowId);
  }

  async closeAll(windowIds: string[]): Promise<void> {
    await Promise.all(
      windowIds.map(async (id) => {
        if (await isOpen(id)) {
          await close(id);
        }
      })
    );
  }
}

// Usage
const devtools = new DevToolsManager();

await devtools.open(mainWindow.id);
await devtools.open(settingsWindow.id);

// Later, close all DevTools
await devtools.closeAll([mainWindow.id, settingsWindow.id]);
```

### UI State Synchronization

```typescript
import { isOpen, onAfter } from "runtime:devtools";
import { send } from "runtime:ipc";

let currentWindowId: string;

// Update UI when DevTools state changes
async function syncDevToolsUI(): Promise<void> {
  const state = await isOpen(currentWindowId);

  // Send state to renderer to update button
  await send(currentWindowId, "devtools:state", { isOpen: state });
}

// Track changes via hooks
onAfter("open", async () => {
  await syncDevToolsUI();
});

onAfter("close", async () => {
  await syncDevToolsUI();
});

// Initial sync
await syncDevToolsUI();
```

### Debug Mode Controller

```typescript
import { open, close, isOpen } from "runtime:devtools";
import { webviewNew } from "runtime:webview";
import { emit, on } from "runtime:ipc";

interface DebugConfig {
  autoOpenDevTools: boolean;
  preserveDevToolsState: boolean;
}

class DebugModeController {
  private config: DebugConfig;
  private windowId: string | null = null;

  constructor(config: DebugConfig) {
    this.config = config;
  }

  async createDebugWindow(url: string): Promise<string> {
    const window = await webviewNew({
      title: "Debug Window",
      url,
      width: 1400,
      height: 900,
      resizable: true,
      debug: true,
      frameless: false
    });

    this.windowId = window.id;

    // Auto-open DevTools if configured
    if (this.config.autoOpenDevTools) {
      await open(window.id);
    }

    return window.id;
  }

  async enableDebugMode(): Promise<void> {
    if (this.windowId) {
      await open(this.windowId);
      console.log("Debug mode enabled - DevTools opened");
    }
  }

  async disableDebugMode(): Promise<void> {
    if (this.windowId) {
      await close(this.windowId);
      console.log("Debug mode disabled - DevTools closed");
    }
  }

  async getDebugState(): Promise<{ windowId: string | null; devToolsOpen: boolean }> {
    if (!this.windowId) {
      return { windowId: null, devToolsOpen: false };
    }

    return {
      windowId: this.windowId,
      devToolsOpen: await isOpen(this.windowId),
    };
  }
}

// Usage
const debugController = new DebugModeController({
  autoOpenDevTools: true,
  preserveDevToolsState: true,
});

const windowId = await debugController.createDebugWindow("app://index.html");
console.log("Debug window created:", windowId);
```

## Best Practices

### Enable Debug Mode for DevTools

```typescript
// DevTools require debug: true in window options
const window = await webviewNew({
  // ... other options
  debug: true,  // Required for DevTools to work
});
```

### Handle Permission Errors

```typescript
import { open } from "runtime:devtools";

try {
  await open(windowId);
} catch (error) {
  if (error.code === 9101) {
    console.warn("DevTools permission denied - check manifest.app.toml");
  } else {
    throw error;
  }
}
```

### Environment-Based DevTools

```typescript
import { open } from "runtime:devtools";

// Only enable DevTools in non-production builds
const allowDevTools = Deno.env.get("FORGE_ENV") !== "production";

if (allowDevTools) {
  await open(windowId);
}
```
