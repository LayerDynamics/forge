---
title: Architecture
description: An overview of Forge's architecture and how its components interact.
slug: architecture
---

This document provides an overview of Forge's architecture, explaining how the runtime works and how components interact.

> **For Contributors:** See the [Implementation Reference](/internals) for detailed file paths and line numbers.

## Overview

Forge is a desktop application framework that combines:

- **Rust** - Host runtime, native integrations, window management
- **Deno** - JavaScript/TypeScript runtime for app logic
- **WebView** - System web renderer (wry/tao) for UI

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Forge Application                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────┐           ┌───────────────────────────┐  │
│  │    Deno Runtime   │    IPC    │    WebView (Renderer)     │  │
│  │   (src/main.ts)   │  ◄─────►  │    (web/index.html)       │  │
│  └─────────┬─────────┘           └───────────────────────────┘  │
│            │                                                    │
│            │ runtime:* modules                                  │
│            ▼                                                    │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                   Forge Runtime (Rust)                    │  │
│  │  ┌─────────┬─────────┬─────────┬─────────┬─────────────┐  │  │
│  │  │ ext_win │ ext_ipc │  ext_fs │ ext_net │ ext_sys ... │  │  │
│  │  └─────────┴─────────┴─────────┴─────────┴─────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                      ┌─────────────────┐
                      │    Operating    │
                      │      System     │
                      └─────────────────┘
```

## Runtime Components

### 1. Forge Host (forge-runtime)

The main Rust binary that orchestrates all runtime components:

- **Embeds Deno runtime** (JsRuntime) with all extension crates registered
- **Creates native windows** via tao/wry, handling the platform event loop
- **Bridges async operations** - Extensions define ops and channel-based commands; forge-runtime handles the main thread event loop integration
- **Initializes extension state** - Sets up IPC channels, capability checkers, and resource limits for each extension
- **Routes IPC messages** - Forwards commands from Deno ops to platform APIs via tokio channels and UserEvent enums
- **Enforces capability permissions** - Parses manifest.app.toml and creates capability adapters for each extension

**Key architectural pattern**: Extensions do NOT reimplement logic in forge-runtime. Extensions define the ops and data structures; forge-runtime initializes their state and provides the event loop bridge. For example:

- `ext_window` defines `WindowCmd` enum and ops
- `forge-runtime` receives `WindowCmd` via channel and translates to `UserEvent` for the tao event loop

Location: `crates/forge-runtime/`

### 2. Deno Runtime

JavaScript/TypeScript execution environment that:

- Runs app's `src/main.ts`
- Provides `runtime:*` module imports
- Handles business logic
- Communicates with renderers via IPC

### 3. WebView Renderers

System-native web views (WebKit/WebView2/WebKitGTK) that:

- Render HTML/CSS/JS UI
- Load content via `app://` protocol
- Communicate with Deno via `window.runtime`

---

## Host Module System

Apps access native capabilities through `runtime:*` module specifiers:

```typescript
import { createWindow, dialog, menu, tray } from "runtime:window";
import { sendToWindow, windowEvents } from "runtime:ipc";
import { readTextFile, writeTextFile } from "runtime:fs";
import { fetch } from "runtime:net";
import { compileFile, instantiate } from "runtime:wasm";
import { getSystemInfo, clipboard } from "runtime:sys";
```

### Module Resolution

1. Deno encounters `import from "runtime:*"`
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

Each `runtime:*` module has a Rust extension with structured error codes. All extensions use the **forge-weld** macro system for automatic TypeScript SDK generation:

| Module | Extension | Error Range | Description |
|--------|-----------|-------------|-------------|
| `runtime:fs` | `ext_fs` | 1000-1999 | File system operations |
| `runtime:ipc` | `ext_ipc` | 7000-7999 | Inter-process communication |
| `runtime:net` | `ext_net` | 3000-3999 | Networking |
| `runtime:process` | `ext_process` | 4000-4999 | Process spawning |
| `runtime:wasm` | `ext_wasm` | 5000-5999 | WebAssembly compilation and execution |
| `runtime:window` | `ext_window` | 6000-6999 | Window management, dialogs, menus, tray |
| `runtime:sys` | `ext_sys` | 8000-8999 | System info, clipboard, notifications |
| `runtime:app` | `ext_app` | - | App lifecycle, info |
| `runtime:crypto` | `ext_crypto` | - | Cryptographic operations |
| `runtime:storage` | `ext_storage` | - | Persistent key-value storage |
| `runtime:shell` | `ext_shell` | - | Shell command execution |
| `runtime:database` | `ext_database` | - | Database operations |
| `runtime:webview` | `ext_webview` | 9000-9999 | WebView manipulation |
| `runtime:devtools` | `ext_devtools` | - | Developer tools |
| `runtime:timers` | `ext_timers` | - | setTimeout/setInterval |
| `runtime:shortcuts` | `ext_shortcuts` | - | Global keyboard shortcuts |
| `runtime:signals` | `ext_signals` | - | OS signal handling |
| `runtime:updater` | `ext_updater` | - | Auto-update functionality |
| `runtime:monitor` | `ext_monitor` | - | Display/monitor info |
| `runtime:display` | `ext_display` | - | Display management |
| `runtime:log` | `ext_log` | - | Logging infrastructure |
| `runtime:trace` | `ext_trace` | - | Tracing/telemetry |
| `runtime:lock` | `ext_lock` | - | File/resource locking |
| `runtime:path` | `ext_path` | - | Path manipulation |
| `runtime:protocol` | `ext_protocol` | - | Custom protocol handlers |
| `runtime:os_compat` | `ext_os_compat` | - | OS compatibility layer |
| `runtime:debugger` | `ext_debugger` | - | Debugging support |

### Forge Tool Modules

In addition to `runtime:*` modules for app functionality, Forge provides `forge:*` modules for development and build tooling:

| Module | Extension | Error Range | Description |
|--------|-----------|-------------|-------------|
| `forge:weld` | `ext_weld` | 8000-8099 | Code generation, TypeScript transpilation |
| `forge:bundler` | `ext_bundler` | 9000-9099 | Icon management, manifest parsing, bundling utilities |

---

## IPC Communication

IPC enables bidirectional messaging between Deno and WebView renderers.

### Renderer → Deno

1. Renderer calls `window.runtime.send("channel", data)`
2. WebView posts message to Rust handler
3. Rust pushes to Deno's message queue
4. Deno receives via `windowEvents()` generator

```text
Renderer                     Rust                      Deno
─────────────────────────────────────────────────────────────────
window.runtime.send()  ──►  WebView IPC  ──►  mpsc channel  ──►  windowEvents()
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

The `forge-weld` crate provides code generation and binding utilities for Forge extensions. It uses a **macro-based inventory system** to automatically generate TypeScript SDK modules from Rust annotations.

#### Macro Annotations

Extensions use three proc-macro attributes to annotate their Rust code:

```rust
use forge_weld_macro::{weld_op, weld_struct, weld_enum};

// Annotate output structs for TypeScript generation
#[weld_struct]
#[derive(Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

// Annotate enums for TypeScript union types
#[weld_enum]
#[derive(Serialize)]
pub enum PathType {
    File,
    Directory,
    Symlink,
}

// Annotate ops for TypeScript function signatures
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_fs_read_text(#[string] path: String) -> Result<String, FsError> {
    // ...
}
```

#### Build Configuration

```rust
// In your extension's build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_fs", "runtime:fs")
        .ts_path("ts/init.ts")
        .ops(&["op_fs_read_text", "op_fs_write_text", "op_fs_stat"])
        .generate_sdk_module("sdk")  // Generates sdk/runtime.fs.ts
        .use_inventory_types()       // Uses linkme distributed slices
        .build()
        .expect("Failed to build extension");
}
```

#### How It Works

1. `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros generate metadata registration code
2. The `linkme` crate's `distributed_slice` collects metadata at compile time
3. `ExtensionBuilder` reads the inventory during `build.rs` execution
4. TypeScript SDK modules are generated with full type information

### Extension Build Output

Each extension's build process generates:

1. **TypeScript SDK** (`sdk/runtime.*.ts`) - Fully typed module with functions and interfaces
2. **Init module** (`ts/init.ts` → `init.js`) - JavaScript shim loaded by Deno runtime
3. **Extension glue** (`extension.rs`) - Generated deno_core extension registration

```text
Extension Build (compile-time)
       │
       ├── Rust source with #[weld_*] macros
       │       │
       │       ▼ linkme distributed_slice
       ├── Metadata inventory collected
       │       │
       │       ▼ build.rs
       ├── ExtensionBuilder reads inventory
       │       │
       │       ├── Generate sdk/runtime.*.ts (TypeScript SDK)
       │       ├── Transpile ts/init.ts → init.js (esbuild)
       │       └── Generate extension.rs (Deno registration)
       │
       └── include!(extension.rs) in lib.rs
```

### Development

```bash
forge dev my-app
    │
    ├── Start forge-runtime with --dev flag
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
    ├── Build forge-runtime with FORGE_EMBED_DIR
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
connect-src 'self' ws://localruntime:35729;
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
forge-runtime
├── deno_core          # Deno runtime
├── deno_ast           # TypeScript transpilation
├── tao                # Window management (cross-platform)
├── wry                # WebView
├── muda               # Menus
├── tray-icon          # System tray
├── rfd                # File dialogs
├── notify             # File watching (HMR)
├── tokio-tungstenite  # WebSocket for HMR
├── tracing            # Logging
└── ext_*              # Host modules (registered as Deno extensions)

forge-weld
├── deno_ast           # TypeScript transpilation via deno_ast
├── linkme             # Compile-time symbol collection (distributed slices)
├── serde              # Type serialization
└── thiserror          # Error types

forge-weld-macro
├── syn                # Rust syntax parsing
├── quote              # Token generation
└── proc-macro2        # Proc macro utilities
```

---

## Crate Structure

Forge consists of 30+ crates organized into core and extension crates:

### Core Crates

| Crate | Purpose |
|-------|---------|
| `forge_cli` | CLI tool (`forge dev/build/bundle/sign/icon`) |
| `forge-runtime` | Main runtime binary |
| `forge-weld` | Code generation and binding utilities |
| `forge-weld-macro` | Procedural macros (`#[weld_op]`, `#[weld_struct]`, `#[weld_enum]`) |

### Extension Crates

All extension crates use forge-weld macros for automatic TypeScript SDK generation:

| Crate | Module | Purpose |
|-------|--------|---------|
| `ext_window` | `runtime:window` | Window management, dialogs, menus, tray |
| `ext_ipc` | `runtime:ipc` | Inter-process communication |
| `ext_fs` | `runtime:fs` | File system operations |
| `ext_net` | `runtime:net` | HTTP fetch, networking |
| `ext_sys` | `runtime:sys` | System info, clipboard, notifications |
| `ext_process` | `runtime:process` | Process spawning |
| `ext_wasm` | `runtime:wasm` | WebAssembly compilation and execution |
| `ext_app` | `runtime:app` | App lifecycle, info |
| `ext_crypto` | `runtime:crypto` | Cryptographic operations |
| `ext_storage` | `runtime:storage` | Persistent key-value storage |
| `ext_shell` | `runtime:shell` | Shell command execution |
| `ext_database` | `runtime:database` | Database operations |
| `ext_webview` | `runtime:webview` | WebView manipulation |
| `ext_devtools` | `runtime:devtools` | Developer tools |
| `ext_timers` | `runtime:timers` | setTimeout/setInterval |
| `ext_shortcuts` | `runtime:shortcuts` | Global keyboard shortcuts |
| `ext_signals` | `runtime:signals` | OS signal handling |
| `ext_updater` | `runtime:updater` | Auto-update functionality |
| `ext_monitor` | `runtime:monitor` | Display/monitor info |
| `ext_display` | `runtime:display` | Display management |
| `ext_log` | `runtime:log` | Logging infrastructure |
| `ext_trace` | `runtime:trace` | Tracing/telemetry |
| `ext_lock` | `runtime:lock` | File/resource locking |
| `ext_path` | `runtime:path` | Path manipulation |
| `ext_protocol` | `runtime:protocol` | Custom protocol handlers |
| `ext_os_compat` | `runtime:os_compat` | OS compatibility layer |
| `ext_debugger` | `runtime:debugger` | Debugging support |

### Forge Tool Extension Crates

| Crate | Module | Purpose |
|-------|--------|---------|
| `ext_weld` | `forge:weld` | Runtime code generation, TypeScript transpilation |
| `ext_bundler` | `forge:bundler` | Icon management, manifest parsing, bundling utilities |

---

## File Structure

```text
forge/
├── crates/
│   ├── forge-runtime/          # Main runtime binary
│   │   ├── src/
│   │   │   ├── main.rs         # Entry point, event loop
│   │   │   └── capabilities.rs # Permission system
│   │   └── build.rs            # Asset embedding
│   │
│   ├── forge_cli/              # CLI tool
│   │   └── src/
│   │       ├── main.rs         # CLI commands
│   │       └── bundler/        # Platform bundling
│   │
│   ├── forge-weld/             # Code generation
│   │   └── src/
│   │       ├── lib.rs          # Main entry
│   │       ├── ir/             # Intermediate representation
│   │       ├── codegen/        # Code generators
│   │       └── build/          # ExtensionBuilder
│   │
│   ├── forge-weld-macro/       # Procedural macros
│   │   └── src/
│   │       ├── lib.rs          # Macro exports
│   │       ├── weld_op.rs      # #[weld_op] impl
│   │       └── weld_struct.rs  # #[weld_struct], #[weld_enum] impl
│   │
│   ├── ext_window/             # runtime:window extension
│   │   ├── src/lib.rs          # Window ops with #[weld_op]
│   │   ├── ts/init.ts          # TypeScript shim
│   │   └── build.rs            # ExtensionBuilder config
│   │
│   ├── ext_ipc/                # runtime:ipc extension
│   ├── ext_fs/                 # runtime:fs extension
│   ├── ext_net/                # runtime:net extension
│   ├── ext_sys/                # runtime:sys extension
│   ├── ext_process/            # runtime:process extension
│   ├── ext_wasm/               # runtime:wasm extension
│   ├── ext_app/                # runtime:app extension
│   ├── ext_crypto/             # runtime:crypto extension
│   ├── ext_storage/            # runtime:storage extension
│   ├── ext_shell/              # runtime:shell extension
│   ├── ext_database/           # runtime:database extension
│   ├── ext_webview/            # runtime:webview extension
│   ├── ext_devtools/           # runtime:devtools extension
│   ├── ext_timers/             # runtime:timers extension
│   ├── ext_weld/               # forge:weld extension
│   ├── ext_bundler/            # forge:bundler extension
│   └── ... (27+ extension crates)
│
├── sdk/                        # TypeScript SDK (auto-generated)
│   ├── generated/              # Type declarations
│   │   ├── runtime.window.d.ts
│   │   ├── runtime.ipc.d.ts
│   │   └── ...
│   ├── runtime.fs.ts           # Generated from ext_fs
│   ├── runtime.window.ts       # Generated from ext_window
│   ├── runtime.ipc.ts          # Generated from ext_ipc
│   ├── runtime.net.ts          # Generated from ext_net
│   ├── runtime.sys.ts          # Generated from ext_sys
│   ├── runtime.app.ts          # Generated from ext_app
│   ├── runtime.crypto.ts       # Generated from ext_crypto
│   ├── runtime.weld.ts         # Generated from ext_weld (forge:weld)
│   ├── runtime.bundler.ts      # Generated from ext_bundler (forge:bundler)
│   ├── ... (27+ runtime modules)
│   └── preload.ts              # Renderer bridge
│
├── examples/                   # Example apps
│   ├── example-deno-app/
│   ├── react-app/
│   ├── svelte-app/
│   └── ...
│
└── site/                       # Documentation site
    └── src/content/docs/
```
