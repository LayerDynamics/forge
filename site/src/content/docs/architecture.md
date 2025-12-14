---
title: Architecture
description: An overview of Forge's architecture and how its components interact.
---

This document provides an overview of Forge's architecture, explaining how the runtime works and how components interact.

## Overview

Forge is a desktop application framework that combines:

- **Rust** - Host runtime, native integrations, window management
- **Deno** - JavaScript/TypeScript runtime for app logic
- **WebView** - System web renderer (wry/tao) for UI

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Forge Application                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────┐             ┌─────────────────────────┐  │
│  │    Deno Runtime   │     IPC     │    WebView (Renderer)   │  │
│  │   (src/main.ts)   │   ◄─────►   │    (web/index.html)     │  │
│  └─────────┬─────────┘             └─────────────────────────┘  │
│            │                                                    │
│            │ host:* modules                                     │
│            ▼                                                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │               Forge Host Runtime (Rust)                 │   │
│  │  ┌────────┬────────┬────────┬────────┬────────┬────────┐  │   │
│  │  │ext_win │ext_ipc │ ext_ui │ ext_fs │ ext_net│ext_wasm│  │   │
│  │  └────────┴────────┴────────┴────────┴────────┴────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                      ┌─────────────────┐
                      │    Operating    │
                      │      System     │
                      └─────────────────┘
```

## Runtime Components

### 1. Forge Host (forge-host)

The main Rust binary that:

- Embeds Deno runtime (JsRuntime)
- Creates native windows (tao/wry)
- Handles system events (window, menu, tray)
- Routes IPC messages
- Enforces capability permissions

Location: `crates/forge-host/`

### 2. Deno Runtime

JavaScript/TypeScript execution environment that:

- Runs app's `src/main.ts`
- Provides `host:*` module imports
- Handles business logic
- Communicates with renderers via IPC

### 3. WebView Renderers

System-native web views (WebKit/WebView2/WebKitGTK) that:

- Render HTML/CSS/JS UI
- Load content via `app://` protocol
- Communicate with Deno via `window.host`

---

## Host Module System

Apps access native capabilities through `host:*` module specifiers:

```typescript
import { createWindow, dialog, menu, tray } from "host:window";
import { sendToWindow, onChannel } from "host:ipc";
import { openWindow } from "host:ui";
import { readTextFile } from "host:fs";
import { fetch } from "host:net";
import { compileFile, instantiate } from "host:wasm";
```

### Module Resolution

1. Deno encounters `import from "host:*"`
2. Custom module loader intercepts
3. Returns ESM shim from extension's `ts/init.ts`
4. Shim calls `Deno.core.ops.*`
5. Op calls Rust extension function

```text
TypeScript            ESM Shim                Rust Op
────────────────────────────────────────────────────────────
readTextFile()  ──►  op_fs_read_text()  ──►  ext_fs::read_text()
```

### Extensions

Each `host:*` module has a Rust extension:

| Module | Extension | Description |
|--------|-----------|-------------|
| `host:window` | `ext_window` | Window management, dialogs, menus, tray |
| `host:ipc` | `ext_ipc` | Inter-process communication |
| `host:ui` | `ext_ui` | Basic window operations |
| `host:fs` | `ext_fs` | File system operations |
| `host:net` | `ext_net` | Networking |
| `host:sys` | `ext_sys` | System info, clipboard, notifications |
| `host:process` | `ext_process` | Process spawning |
| `host:wasm` | `ext_wasm` | WebAssembly compilation and execution |

---

## IPC Communication

IPC enables bidirectional messaging between Deno and WebView renderers.

### Renderer → Deno

1. Renderer calls `window.host.send("channel", data)`
2. WebView posts message to Rust handler
3. Rust pushes to Deno's message queue
4. Deno receives via `windowEvents()` generator

```text
Renderer                     Rust                      Deno
─────────────────────────────────────────────────────────────────
window.host.send()  ──►  WebView IPC  ──►  mpsc channel  ──►  windowEvents()
```

### Deno → Renderer

1. Deno calls `sendToWindow(windowId, channel, payload)`
2. Rust serializes and routes to WebView
3. WebView executes `window.__host_dispatch()`
4. Preload script calls registered callbacks

```text
Deno                        Rust                         Renderer
──────────────────────────────────────────────────────────────────────
sendToWindow()  ──►  evaluate_script()  ──►  __host_dispatch()  ──►  callbacks
```

### Channel Allowlists

Channels can be restricted per-window:

```toml
[capabilities.channels]
allowed = ["user:*", "app:state"]
```

Empty allowlist = deny all (security default)

---

## Capability Model

Forge uses capability-based security to restrict app permissions.

### Capability Flow

```text
┌────────────────────────────────────────────────────────────┐
│                     manifest.app.toml                      │
│  [capabilities.fs]                                         │
│  read = ["~/.myapp/*"]                                     │
└────────────────────────────────────────────────────────────┘
                               │
                               ▼ parsed at startup
┌────────────────────────────────────────────────────────────┐
│                    Capabilities Struct                     │
│  FsCapabilities { read_patterns: [...], write_patterns }   │
└────────────────────────────────────────────────────────────┘
                               │
                               ▼ checked on each op call
┌────────────────────────────────────────────────────────────┐
│                    op_fs_read_text(path)                   │
│  1. Resolve path (expand ~, normalize)                     │
│  2. Check against read_patterns                            │
│  3. Return error if denied                                 │
│  4. Perform operation if allowed                           │
└────────────────────────────────────────────────────────────┘
```

### Pattern Matching

Capabilities use glob patterns:

- `*` matches any non-path-separator characters
- `**` matches any characters including `/`
- `~` expands to home directory

Example checks:

```rust
// Capability: read = ["~/.myapp/*"]
check_read("~/.myapp/config.json")  // ✓ Allowed
check_read("~/.myapp/data/file.txt") // ✗ Denied (no **)
check_read("~/.other/file.txt")      // ✗ Denied
```

---

## Asset Loading

### Development Mode

```text
Request: app://index.html
         │
         ▼
┌─────────────────────┐
│  Custom Protocol    │
│  Handler (Rust)     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Read from disk:    │
│  {app_dir}/web/...  │
└─────────────────────┘
```

### Production Mode

```text
FORGE_EMBED_DIR=./web cargo build
         │
         ▼
┌─────────────────────┐
│  build.rs embeds    │
│  files into binary  │
└─────────────────────┘
         │
         ▼
Request: app://index.html
         │
         ▼
┌─────────────────────┐
│  Serve from         │
│  embedded assets    │
└─────────────────────┘
```

---

## Window Management

### Window Lifecycle

```text
createWindow(opts)
    │
    ├── Create tao::Window
    │       │
    │       └── Configure: size, title, decorations
    │
    ├── Create wry::WebView
    │       │
    │       ├── Load app:// URL
    │       ├── Inject preload script
    │       └── Set up IPC handlers
    │
    └── Return Window handle
            │
            ├── Methods: close(), minimize(), maximize()
            ├── Position: getPosition(), setPosition()
            ├── Size: getSize(), setSize()
            └── State: isFullscreen(), isFocused()
```

### Event Loop

The main event loop handles:

1. Window events (close, resize, focus)
2. Menu events (app menu, context menu, tray)
3. IPC messages (renderer → Deno)
4. File watcher events (HMR in dev mode)

```rust
event_loop.run(move |event, target, control_flow| {
    match event {
        Event::WindowEvent { .. } => handle_window_event(),
        Event::MenuEvent { .. } => handle_menu_event(),
        Event::UserEvent(msg) => handle_ipc_message(),
        _ => {}
    }
});
```

---

## Build System

### forge-weld

The `forge-weld` crate provides code generation and binding utilities for Forge extensions. It generates TypeScript type definitions, init modules, and Rust extension macros.

```rust
// In your extension's build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_fs", "host:fs")
        .ts_path("ts/init.ts")
        .ops(&["op_fs_read_text", "op_fs_write_text"])
        .generate_sdk_types("sdk")
        .dts_generator(generate_types)
        .build()
        .expect("Failed to build extension");
}
```

### Extension Build Output

Each extension's build process generates:

1. **TypeScript types** (`sdk/generated/host.*.d.ts`) - Type declarations for the module
2. **Init module** (embedded in binary) - The transpiled TypeScript shim
3. **Rust bindings** - Extension registration and op definitions

```text
Extension Build
       │
       ├── ts/init.ts (TypeScript source)
       │       │
       │       ▼
       ├── Transpile to JavaScript (esbuild)
       │       │
       │       ▼
       ├── Embed in binary (build.rs)
       │
       └── Generate .d.ts (sdk/generated/)
```

### Development

```bash
forge dev my-app
    │
    ├── Start forge-host with --dev flag
    ├── Load manifest.app.toml
    ├── Initialize Deno runtime
    ├── Execute src/main.ts
    ├── Start HMR WebSocket server (port 35729)
    └── Watch for file changes
```

### Production Build

```bash
forge build my-app
    │
    ├── Bundle Deno code (esbuild)
    ├── Copy web assets
    └── Output to dist/

forge bundle my-app
    │
    ├── Build forge-host with FORGE_EMBED_DIR
    ├── Create platform package:
    │   ├── macOS: .app bundle → DMG
    │   ├── Windows: MSIX package
    │   └── Linux: AppImage
    └── Output to dist/{platform}/
```

---

## Security Model

### Sandboxing Layers

1. **Capability System** - Explicit permission grants
2. **Channel Allowlists** - IPC message filtering
3. **CSP Headers** - Content Security Policy
4. **Process Isolation** - WebView in separate process

### CSP Configuration

Development (relaxed for HMR):

```text
default-src 'self' app:;
script-src 'self' 'unsafe-inline' 'unsafe-eval' app:;
connect-src 'self' ws://localhost:35729;
```

Production (strict):

```text
default-src 'self' app:;
script-src 'self' app:;
style-src 'self' 'unsafe-inline' app:;
```

---

## Crate Dependencies

```text
forge-host
├── deno_core      # Deno runtime
├── tao            # Window management
├── wry            # WebView
├── muda           # Menus
├── tray-icon      # System tray
├── rfd            # File dialogs
├── notify         # File watching
├── wasmtime       # WebAssembly runtime
└── ext_*          # Host modules

forge-weld
├── swc_ecma_parser    # TypeScript parsing
├── swc_ecma_codegen   # JavaScript generation
└── linkme             # Build-time code collection
```

---

## Crate Structure

Forge consists of 12 crates:

### Core Crates

| Crate | Purpose |
|-------|---------|
| `forge` | CLI tool (`forge dev/build/bundle`) |
| `forge-host` | Main runtime binary |
| `forge-weld` | Code generation and binding utilities |
| `forge-weld-macro` | Procedural macros for forge-weld |

### Extension Crates

| Crate | Module | Purpose |
|-------|--------|---------|
| `ext_window` | `host:window` | Window management, dialogs, menus, tray |
| `ext_ipc` | `host:ipc` | Inter-process communication |
| `ext_ui` | `host:ui` | Basic window operations |
| `ext_fs` | `host:fs` | File system operations |
| `ext_net` | `host:net` | Networking |
| `ext_sys` | `host:sys` | System info, clipboard, notifications |
| `ext_process` | `host:process` | Process spawning |
| `ext_wasm` | `host:wasm` | WebAssembly compilation and execution |

---

## File Structure

```text
forge/
├── crates/
│   ├── forge-host/          # Main runtime binary
│   │   ├── src/
│   │   │   ├── main.rs      # Entry point, event loop
│   │   │   └── capabilities.rs
│   │   └── build.rs         # Asset embedding
│   │
│   ├── forge/               # CLI tool
│   │   └── src/
│   │       └── main.rs      # CLI commands
│   │
│   ├── forge-weld/          # Code generation
│   │   └── src/
│   │       ├── lib.rs       # Main entry
│   │       ├── ir.rs        # Intermediate representation
│   │       ├── codegen.rs   # Code generators
│   │       └── build.rs     # ExtensionBuilder
│   │
│   ├── forge-weld-macro/    # Procedural macros
│   │
│   ├── ext_window/          # host:window extension
│   │   ├── src/lib.rs       # Window ops
│   │   ├── ts/init.ts       # TypeScript shim
│   │   └── build.rs         # Type generation
│   │
│   ├── ext_ipc/             # host:ipc extension
│   │   ├── src/lib.rs       # IPC ops
│   │   ├── ts/init.ts       # TypeScript shim
│   │   └── build.rs         # Type generation
│   │
│   ├── ext_ui/              # host:ui extension
│   ├── ext_fs/              # host:fs extension
│   ├── ext_net/             # host:net extension
│   ├── ext_sys/             # host:sys extension
│   ├── ext_process/         # host:process extension
│   └── ext_wasm/            # host:wasm extension
│
├── sdk/                     # TypeScript SDK
│   ├── generated/           # Auto-generated types
│   │   ├── host.window.d.ts
│   │   ├── host.ipc.d.ts
│   │   ├── host.ui.d.ts
│   │   ├── host.fs.d.ts
│   │   └── ...
│   └── preload.ts           # Renderer bridge
│
├── examples/                # Example apps
│   └── example-deno-app/
│
└── site/                    # Documentation site
    └── src/content/docs/
```
