# Getting Started with Forge

Forge is an Electron-like desktop application framework using Rust and Deno. Build cross-platform desktop apps with TypeScript/JavaScript while leveraging native system capabilities through a secure, capability-based API.

## Prerequisites

Before getting started, ensure you have:

- **Deno** 1.40 or later ([deno.land](https://deno.land))
- A code editor (VS Code recommended)

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://forge-deno.com/install.sh | sh
```

This installs both `forge` and `forge-host` to `~/.forge/bin/`.

### Manual Download

Download the latest release for your platform from [GitHub Releases](https://github.com/LayerDynamics/forge/releases) and extract to `~/.forge/bin/`:

```bash
# Linux
tar -xzf forge-x86_64-unknown-linux-gnu.tar.gz -C ~/.forge/bin/

# macOS
tar -xzf forge-aarch64-apple-darwin.tar.gz -C ~/.forge/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.forge/bin:$PATH"
```

### For Contributors (Building from Source)

If you're contributing to Forge itself, you'll need Rust 1.70+:

```bash
git clone https://github.com/LayerDynamics/forge.git
cd forge
cargo build --workspace --release
cargo install --path crates/forge
cargo install --path crates/forge-host
```

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
import { openWindow, windowEvents } from "host:ui";

// Open the main window
const win = await openWindow({
  url: "app://index.html",
  width: 800,
  height: 600,
  title: "My App"
});

// Listen for events from the renderer
for await (const event of windowEvents()) {
  console.log("Event:", event.channel, event.payload);
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
    // Communicate with Deno backend
    window.host.send("hello", { message: "Hi from renderer!" });

    // Listen for messages from backend
    window.host.on("reply", (data) => {
      console.log("Received:", data);
    });

    // Signal ready
    window.host.emit("ready");
  </script>
</body>
</html>
```

## Development Mode

Run your app in development mode with hot reload:

```bash
forge dev my-app
```

This starts the Forge runtime with:
- Live reload on file changes
- Development-friendly CSP settings
- Console output in terminal

## Host Modules

Forge provides native capabilities through `host:*` modules:

### host:ui - Window Management

```typescript
import { openWindow, dialog, createTray } from "host:ui";

// Open a window
const win = await openWindow({ url: "app://index.html" });

// Show a dialog
const path = await dialog.open({ title: "Select File" });

// Create a tray icon
const tray = await createTray({ tooltip: "My App" });
```

### host:fs - File System

```typescript
import { readTextFile, writeTextFile, watch } from "host:fs";

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

### host:net - Networking

```typescript
import { fetchJson } from "host:net";

const data = await fetchJson("https://api.example.com/data");
```

### host:sys - System Operations

```typescript
import { clipboard, notify, info } from "host:sys";

// System info
const sysInfo = info();
console.log(sysInfo.os, sysInfo.arch);

// Clipboard
await clipboard.write("Hello");
const text = await clipboard.read();

// Notifications
await notify("Title", "Body text");
```

### host:process - Process Management

```typescript
import { spawn } from "host:process";

const proc = await spawn("ls", { args: ["-la"] });
for await (const line of proc.stdout) {
  console.log(line);
}
await proc.wait();
```

## IPC Communication

Forge uses a simple message-passing model for communication between Deno and the renderer:

### From Renderer to Deno

```javascript
// In web/index.html
window.host.send("channel-name", { data: "value" });
```

### From Deno to Renderer

```typescript
// In src/main.ts
win.send("channel-name", { data: "value" });
```

### Listening for Events

```typescript
// In Deno - listen for all window events
for await (const event of windowEvents()) {
  if (event.channel === "user-action") {
    // Handle event
  }
}

// In renderer - listen for specific channel
window.host.on("update", (data) => {
  // Handle update
});
```

## Building for Production

Build your app for distribution:

```bash
# Build the app bundle
forge build my-app

# Create platform-specific packages
forge bundle my-app

# Sign the bundle (macOS/Windows)
forge sign my-app/bundle/MyApp.dmg --identity "Developer ID"
```

### App Icon

Your app needs an icon for bundling. Create one before release:

```bash
# Create a placeholder icon
forge icon create my-app/assets/icon.png

# Validate your icon
forge icon validate my-app
```

**Icon requirements:** 1024x1024 PNG with transparency. See the [Icons Guide](./guides/icons.md) for details.

### Code Signing

Code signing is required for macOS distribution and recommended for Windows. Configure signing in your manifest:

```toml
[bundle.macos]
sign = true
signing_identity = "Developer ID Application: Your Name (TEAMID)"
```

See the [Code Signing Guide](./guides/code-signing.md) for platform-specific instructions.

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

- Read the [Architecture Overview](./architecture.md)
- Explore the [API Reference](./api/)
- Check the [Example Apps](../examples/)

## Getting Help

- GitHub Issues: [github.com/LayerDynamics/forge/issues](https://github.com/LayerDynamics/forge/issues)
- Documentation: [forge-deno.com/docs](https://forge-deno.com/docs)
