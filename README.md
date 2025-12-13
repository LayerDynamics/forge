# Forge

> Build cross-platform desktop apps with TypeScript/JavaScript and native capabilities.

Forge is an Electron-like desktop application framework using **Rust** and **Deno**. Apps are 100% TypeScript/JavaScript - no per-app Rust required. The runtime provides native system access through a secure, capability-based API.

**Status:** Alpha (0.1.0-alpha.1)

## Features

- **Native Performance** - Rust host runtime with system WebViews (not Chromium)
- **TypeScript First** - Write app logic in TypeScript with full type support
- **Capability Security** - Explicit permission model for system access
- **Cross-Platform** - Build for macOS, Windows, and Linux from one codebase
- **Multiple Frameworks** - React, Vue, Svelte, or vanilla JS templates
- **Hot Reload** - Live updates during development

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://forge-deno.com/install.sh | sh
```

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

### From Source (for contributors)

```bash
cargo install --path crates/forge
cargo install --path crates/forge-host
```

## Quick Start

```bash
# Create a new app
forge init my-app
cd my-app

# Run in development mode
forge dev .

# Build for production
forge build .
forge bundle .
```

## Host Modules

Access native capabilities through `host:*` imports:

```typescript
// Window management
import { openWindow, dialog, createTray } from "host:ui";

// File system
import { readTextFile, writeTextFile, watch } from "host:fs";

// Networking
import { fetchJson } from "host:net";

// System operations
import { clipboard, notify, info } from "host:sys";

// Process management
import { spawn } from "host:process";
```

## Project Structure

```
my-app/
├── manifest.app.toml   # App config & capabilities
├── deno.json           # Deno configuration
├── src/
│   └── main.ts         # Deno entry point
└── web/
    └── index.html      # UI entry point
```

## Example

**src/main.ts:**
```typescript
import { openWindow, windowEvents } from "host:ui";

const win = await openWindow({
  url: "app://index.html",
  title: "My App",
  width: 800,
  height: 600
});

for await (const event of windowEvents()) {
  console.log("Event:", event.channel, event.payload);
}
```

**web/index.html:**
```html
<script>
  window.host.send("hello", { message: "Hi!" });
  window.host.on("update", (data) => console.log(data));
  window.host.emit("ready");
</script>
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [API Reference](docs/api/)
- [Architecture](docs/architecture.md)
- [Examples](apps/)

## Example Apps

| App | Demonstrates |
|-----|--------------|
| [todo-app](apps/todo-app) | File persistence, menus, IPC (React) |
| [weather-app](apps/weather-app) | HTTP fetch, notifications, tray (Vue) |
| [text-editor](apps/text-editor) | Dialogs, context menus, file watching |
| [system-monitor](apps/system-monitor) | System info, multi-window |

## Crate Structure

| Crate | Description |
|-------|-------------|
| `forge-host` | Main runtime binary |
| `forge` | CLI tool |
| `ext_ui` | Window management extension |
| `ext_fs` | File system extension |
| `ext_net` | Networking extension |
| `ext_sys` | System operations extension |
| `ext_process` | Process management extension |

## Development (For Contributors)

These commands are for developers contributing to the Forge framework itself.
If you're building apps with Forge, you only need Deno - just use the `forge` CLI commands shown above.

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run the example app
cargo run -p forge -- dev apps/example-deno-app

# Build with release optimizations
cargo build --workspace --release
```

## Requirements

**For App Developers:**
- Deno 1.40+

**For Forge Contributors:**
- Rust 1.70+
- Deno 1.40+

## License

MIT

## Disclaimer

This is alpha software. APIs may change. Not recommended for production use.
