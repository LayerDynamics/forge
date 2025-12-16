---
title: Getting Started
description: Learn how to build your first Forge desktop app with TypeScript and Deno.
slug: getting-started
---

Forge is an Electron-like desktop application framework using Rust and Deno. Build cross-platform desktop apps with TypeScript/JavaScript while leveraging native system capabilities through a secure, capability-based API.

## Prerequisites

Before getting started, ensure you have:

- **Deno** 1.40 or later ([deno.land](https://deno.land))
- A code editor (VS Code recommended)

## Installation

Install Forge with a single command:

```bash
curl -fsSL https://forge-deno.com/install.sh | sh
```

Or download manually from [GitHub Releases](https://github.com/LayerDynamics/forge/releases).

## Create Your First App

Copy an example to start a new project:

```bash
# Copy the minimal example
cp -r examples/example-deno-app my-app
cd my-app

# Or use a framework example
cp -r examples/react-app my-app      # React with TypeScript
cp -r examples/nextjs-app my-app     # Next.js-style patterns
cp -r examples/svelte-app my-app     # Svelte with TypeScript
```

This creates a new Forge application with the following structure:

```
my-app/
├── manifest.app.toml   # App configuration and capabilities
├── deno.json           # Deno configuration
├── src/
│   └── main.ts         # Main Deno entry point
└── web/
    └── index.html      # UI entry point
```

## Project Structure

### manifest.app.toml

The manifest defines your app's metadata and capabilities:

```toml
[app]
name = "My App"
identifier = "com.example.myapp"
version = "0.1.0"

[windows]
width = 800
height = 600
resizable = true

# Capability declarations (optional)
[capabilities.fs]
read = ["~/.myapp/*"]
write = ["~/.myapp/*"]

[capabilities.channels]
allowed = ["*"]
```

### src/main.ts

The main Deno entry point handles app logic:

```typescript
import { createWindow, dialog, menu } from "runtime:window";
import { onChannel, sendToWindow } from "runtime:ipc";

// Create the main window
const win = await createWindow({
  url: "app://index.html",
  width: 800,
  height: 600,
  title: "My App",
});

// Set up application menu
await menu.setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "open", label: "Open...", accelerator: "CmdOrCtrl+O" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
]);

// Listen for IPC messages from the renderer
onChannel("hello", async (payload, windowId) => {
  console.log("Received from renderer:", payload);
  await sendToWindow(windowId, "reply", { message: "Hello from Deno!" });
});

// Listen for window events
for await (const event of win.events()) {
  if (event.type === "close") {
    const confirmed = await dialog.confirm("Are you sure you want to quit?");
    if (confirmed) {
      Deno.exit(0);
    }
  }
}
```

### web/index.html

The UI is standard HTML/CSS/JS served via the `app://` protocol:

```html
<!DOCTYPE html>
<html>
<head>
  <title>My App</title>
</head>
<body>
  <h1>Hello, Forge!</h1>
  <script>
    // Send message to Deno backend
    window.runtime.send("hello", { message: "Hi from renderer!" });

    // Listen for messages from backend
    window.runtime.on("reply", (data) => {
      console.log("Received:", data);
    });
  </script>
</body>
</html>
```

## Development Mode

Run your app in development mode with hot reload:

```bash
forge dev .
```

This starts the Forge runtime with:
- Live reload on file changes
- Development-friendly CSP settings
- Console output in terminal

## Runtime Modules

Forge provides native capabilities through `runtime:*` modules. TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld):

### runtime:window - Window Management

Full window control including position, size, state, dialogs, menus, and system tray:

```typescript
import { createWindow, dialog, menu, tray } from "runtime:window";

// Create a window with full control
const win = await createWindow({
  url: "app://index.html",
  title: "My App",
  width: 1024,
  height: 768,
});

// Window manipulation
await win.setPosition(100, 100);
await win.setSize(1280, 720);
await win.maximize();
await win.setAlwaysOnTop(true);

// Dialogs
const files = await dialog.open({
  title: "Select Files",
  multiple: true,
  filters: [{ name: "Images", extensions: ["png", "jpg"] }],
});

// Application menu
await menu.setAppMenu([
  {
    label: "File",
    submenu: [
      { id: "new", label: "New", accelerator: "CmdOrCtrl+N" },
      { id: "quit", label: "Quit", accelerator: "CmdOrCtrl+Q" },
    ],
  },
]);

// System tray
const trayIcon = await tray.create({
  tooltip: "My App",
  menu: [
    { id: "show", label: "Show Window" },
    { id: "quit", label: "Quit" },
  ],
});
```

### runtime:ipc - Inter-Process Communication

Bidirectional messaging between Deno and WebView renderers:

```typescript
import { sendToWindow, onChannel, windowEvents, broadcast } from "runtime:ipc";

// Send to a specific window
await sendToWindow("main", "update", { count: 42 });

// Listen for events on a specific channel
onChannel("button-click", (payload, windowId) => {
  console.log(`Button clicked in ${windowId}:`, payload);
});

// Async generator for all events
for await (const event of windowEvents()) {
  console.log(`[${event.windowId}] ${event.channel}:`, event.payload);
}

// Broadcast to multiple windows
await broadcast(["main", "settings"], "theme-changed", { theme: "dark" });
```

### runtime:fs - File System

```typescript
import { readTextFile, writeTextFile, watch } from "runtime:fs";

// Read a file
const content = await readTextFile("./config.json");

// Write a file
await writeTextFile("./output.txt", "Hello!");

// Watch for changes
const watcher = await watch("./src");
for await (const event of watcher) {
  console.log("File changed:", event.paths);
}
```

### runtime:net - Networking

```typescript
import { fetchJson } from "runtime:net";

const data = await fetchJson("https://api.example.com/data");
```

### runtime:sys - System Operations

```typescript
import { clipboard, notify, info } from "runtime:sys";

// System info
const sysInfo = info();
console.log(sysInfo.os, sysInfo.arch);

// Clipboard
await clipboard.write("Hello");
const text = await clipboard.read();

// Notifications
await notify("Title", "Body text");
```

### runtime:process - Process Management

```typescript
import { spawn } from "runtime:process";

const proc = await spawn("ls", { args: ["-la"] });
for await (const line of proc.stdout) {
  console.log(line);
}
await proc.wait();
```

### runtime:wasm - WebAssembly

```typescript
import { compileFile, instantiate, types } from "runtime:wasm";

// Compile and instantiate a WASM module
const moduleId = await compileFile("./module.wasm");
const instance = await instantiate(moduleId);

// Call exported functions
const [result] = await instance.call("add", 2, 3);
console.log(result); // 5

// With explicit types for large integers
const [bigResult] = await instance.call(
  "multiply",
  types.i64(9007199254740992n),
  types.i64(2n)
);

// WASI support for system access
const wasiInstance = await instantiate(moduleId, {
  preopens: { "/data": "./app-data" },
  args: ["app", "--verbose"],
  inheritStdout: true,
});
await wasiInstance.call("_start");
```

## IPC Communication

Forge uses a message-passing model for communication between Deno and the renderer.

### From Renderer to Deno

```javascript
// In web/index.html
window.runtime.send("channel-name", { data: "value" });
```

### From Deno to Renderer

```typescript
// In src/main.ts
import { sendToWindow } from "runtime:ipc";

await sendToWindow("window-id", "channel-name", { data: "value" });
```

### Listening for Events

```typescript
// In Deno - callback-based
import { onChannel } from "runtime:ipc";

onChannel("user-action", (payload, windowId) => {
  console.log(`Action from ${windowId}:`, payload);
});

// In Deno - async generator
import { windowEvents } from "runtime:ipc";

for await (const event of windowEvents()) {
  if (event.channel === "user-action") {
    // Handle event
  }
}

// In renderer - listen for specific channel
window.runtime.on("update", (data) => {
  // Handle update
});
```

## Building for Production

Build your app for distribution:

```bash
# Build the app bundle
forge build .

# Create platform-specific packages
forge bundle .

# Sign the bundle (macOS/Windows)
forge sign ./bundle/MyApp.app --identity "Developer ID"
```

## Example Apps

Check out the example apps in the `examples/` directory:

- **example-deno-app** - Minimal starter app
- **react-app** - React with TypeScript and IPC demo
- **nextjs-app** - Next.js-style routing patterns
- **svelte-app** - Svelte with TypeScript and todo list
- **todo-app** - File persistence, menus, IPC patterns
- **text-editor** - Full file operations, dialogs, context menus
- **weather-app** - HTTP fetch, notifications, tray icons
- **system-monitor** - System info, multi-window, process management

## Next Steps

- Read the [Architecture Overview](/docs/architecture)
- Explore the [API Reference](/docs/api/runtime-window)
- Check the [Example Apps](https://github.com/LayerDynamics/forge/tree/main/examples)
- For contributors: Learn about [forge-weld](/docs/crates/forge-weld) for building extensions
