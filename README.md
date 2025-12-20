# Forge

> Build cross-platform desktop apps with TypeScript and native capabilities as well as wasm, bundle and weld support.

Forge is an Electron-like desktop application framework using **Rust** and **Deno**. Apps are 100% TypeScript - no per-app Rust required. The runtime provides native system access through a secure, capability-based API.

**Status:** Alpha (v1.0.0p-steel-donut 🍩)

## Features

- **Native Performance** - Rust runtime runtime with system WebViews (not Chromium)
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
cargo install --path crates/forge_cli
cargo install --path crates/forge-runtime
```

## Quick Start

```bash
# Copy an example to start a new app
cp -r examples/react-app my-app
cd my-app

# Run in development mode
forge dev .

# Build for production
forge build .
forge bundle .
```

## Host Modules

Access native capabilities through `runtime:*` imports:

```typescript
// Window management
import { openWindow, dialog, createTray } from "runtime:ui";

// File system
import { readTextFile, writeTextFile, watch } from "runtime:fs";

// Networking
import { fetchJson } from "runtime:net";

// System operations
import { clipboard, notify, info } from "runtime:sys";

// Process management
import { spawn } from "runtime:process";
```

## Project Structure

```ascii
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
import { openWindow, windowEvents } from "runtime:ui";

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
- [Examples](examples/)

## Example Apps

| App | Demonstrates |
|-----|--------------|
| [example-deno-app](examples/example-deno-app) | Minimal starter app |
| [react-app](examples/react-app) | React with TypeScript and IPC |
| [nextjs-app](examples/nextjs-app) | Next.js-style routing patterns |
| [svelte-app](examples/svelte-app) | Svelte with TypeScript |
| [todo-app](examples/todo-app) | File persistence, menus, IPC |
| [text-editor](examples/text-editor) | Dialogs, context menus, file watching |
| [weather-app](examples/weather-app) | HTTP fetch, notifications, tray |
| [system-monitor](examples/system-monitor) | System info, multi-window |

## Crate Structure

### Core Crates

| Crate | Description |
|-------|-------------|
| `forge-runtime` | Main runtime binary with extension registry |
| `forge_cli` | CLI tool (dev, build, bundle, sign, icon, docs) |
| `forge-weld` | Code generation framework for TypeScript bindings |
| `forge-weld-macro` | Proc macros (#[weld_op], #[weld_struct], #[weld_enum]) |
| `forge-etch` | Documentation generation and TypeScript parsing |
| `forge-smelt` | Binary compilation and transpilation |

### Extensions (runtime:* modules)

| Extension | Module | Description |
|-----------|--------|-------------|
| `ext_window` | `runtime:window` | Window management, menus, trays, dialogs |
| `ext_fs` | `runtime:fs` | File operations (read, write, watch, stat) |
| `ext_ipc` | `runtime:ipc` | Deno ↔ Renderer communication |
| `ext_net` | `runtime:net` | HTTP fetch, network operations |
| `ext_sys` | `runtime:sys` | System info, clipboard, notifications |
| `ext_process` | `runtime:process` | Spawn child processes |
| `ext_app` | `runtime:app` | App lifecycle and info |
| `ext_crypto` | `runtime:crypto` | Cryptographic operations |
| `ext_storage` | `runtime:storage` | Persistent key-value storage |
| `ext_database` | `runtime:database` | Database operations |
| `ext_shell` | `runtime:shell` | Cross-platform shell commands |
| `ext_wasm` | `runtime:wasm` | WebAssembly module loading |
| `ext_bundler` | `runtime:bundler` | App bundling operations |
| `ext_codesign` | `runtime:codesign` | Code signing (macOS/Windows/Linux) |
| `ext_dock` | `runtime:dock` | macOS dock integration |
| `ext_encoding` | `runtime:encoding` | Text encoding/decoding |
| `ext_etcher` | `runtime:etcher` | Documentation generation |
| `ext_image_tools` | `runtime:image_tools` | Image conversion (PNG, SVG, WebP, ICO) |
| `ext_svelte` | `runtime:svelte` | SvelteKit integration |
| `ext_web_inspector` | `runtime:web_inspector` | Chrome DevTools Protocol bridge |
| `ext_weld` | `runtime:weld` | Runtime binding system access |
| `ext_devtools` | `runtime:devtools` | Developer tools integration |
| `ext_webview` | `runtime:webview` | WebView management |
| `ext_updater` | `runtime:updater` | App update system |
| `ext_monitor` | `runtime:monitor` | System monitoring |
| `ext_display` | `runtime:display` | Display information |
| `ext_log` | `runtime:log` | Logging operations |
| `ext_trace` | `runtime:trace` | Tracing and diagnostics |
| `ext_lock` | `runtime:lock` | File locking |
| `ext_path` | `runtime:path` | Path manipulation |
| `ext_protocol` | `runtime:protocol` | Custom protocol handlers |
| `ext_os_compat` | `runtime:os_compat` | OS compatibility layer |
| `ext_debugger` | `runtime:debugger` | Debugger integration |
| `ext_shortcuts` | `runtime:shortcuts` | Keyboard shortcuts |
| `ext_signals` | `runtime:signals` | Signal handling |
| `ext_timers` | `runtime:timers` | Timer operations |

## Development (For Contributors)

These commands are for developers contributing to the Forge framework itself.
If you're building apps with Forge, you only need Deno - just use the `forge` CLI commands shown above.

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Run the example app
cargo run -p forge_cli -- dev examples/example-deno-app

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
