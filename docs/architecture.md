# Forge Architecture

This document provides an overview of Forge's architecture, explaining how the runtime works and how components interact.

## Overview

Forge is a desktop application framework that combines:

- **Rust** - Host runtime, native integrations, window management
- **Deno** - JavaScript/TypeScript runtime for app logic
- **WebView** - System web renderer (wry/tao) for UI

```
┌─────────────────────────────────────────────────────────────┐
│                     Forge Application                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │   Deno Runtime  │   IPC   │   WebView (Renderer)    │   │
│  │   (src/main.ts) │ ◄─────► │   (web/index.html)      │   │
│  └────────┬────────┘         └─────────────────────────┘   │
│           │                                                  │
│           │ host:* modules                                   │
│           ▼                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Forge Host Runtime (Rust)               │   │
│  │  ┌─────────┬─────────┬─────────┬─────────┬───────┐  │   │
│  │  │ ext_ui  │ ext_fs  │ ext_net │ ext_sys │ext_proc│ │   │
│  │  └─────────┴─────────┴─────────┴─────────┴───────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Operating      │
                    │  System         │
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
import { openWindow } from "host:ui";
import { readTextFile } from "host:fs";
import { fetch } from "host:net";
```

### Module Resolution

1. Deno encounters `import from "host:*"`
2. Custom module loader intercepts
3. Returns ESM shim from `sdk/*.ts`
4. Shim calls `Deno.core.ops.*`
5. Op calls Rust extension function

```
TypeScript          ESM Shim              Rust Op
─────────────────────────────────────────────────────────
readTextFile() ──► op_fs_read_text() ──► ext_fs::read_text()
```

### Extensions

Each `host:*` module has a Rust extension:

| Module | Extension | Location |
|--------|-----------|----------|
| `host:ui` | `ext_ui` | `crates/ext_ui/` |
| `host:fs` | `ext_fs` | `crates/ext_fs/` |
| `host:net` | `ext_net` | `crates/ext_net/` |
| `host:sys` | `ext_sys` | `crates/ext_sys/` |
| `host:process` | `ext_process` | `crates/ext_process/` |

---

## IPC Communication

### Renderer → Deno

1. Renderer calls `window.host.send("channel", data)`
2. WebView posts message to Rust handler
3. Rust pushes to Deno's message queue
4. Deno receives via `windowEvents()` generator

```
Renderer                    Rust                    Deno
────────────────────────────────────────────────────────────
window.host.send() ──► WebView IPC ──► mpsc channel ──► windowEvents()
```

### Deno → Renderer

1. Deno calls `win.send("channel", data)` or `sendToWindow()`
2. Rust serializes and routes to WebView
3. WebView executes `window.__host_dispatch()`
4. Preload script calls registered callbacks

```
Deno                        Rust                    Renderer
────────────────────────────────────────────────────────────
sendToWindow() ──► evaluate_script() ──► __host_dispatch() ──► callbacks
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

```
┌──────────────────────────────────────────────────────────┐
│                    manifest.app.toml                      │
│  [capabilities.fs]                                        │
│  read = ["~/.myapp/*"]                                   │
└──────────────────────────────────────────────────────────┘
                              │
                              ▼ parsed at startup
┌──────────────────────────────────────────────────────────┐
│                   Capabilities Struct                     │
│  FsCapabilities { read_patterns: [...], write_patterns }  │
└──────────────────────────────────────────────────────────┘
                              │
                              ▼ checked on each op call
┌──────────────────────────────────────────────────────────┐
│                 op_fs_read_text(path)                     │
│  1. Resolve path (expand ~, normalize)                   │
│  2. Check against read_patterns                          │
│  3. Return error if denied                               │
│  4. Perform operation if allowed                         │
└──────────────────────────────────────────────────────────┘
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

```
Request: app://index.html
    │
    ▼
┌───────────────────┐
│ Custom Protocol   │
│ Handler (Rust)    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Read from disk:   │
│ {app_dir}/web/... │
└───────────────────┘
```

### Production Mode

```
FORGE_EMBED_DIR=./web cargo build
    │
    ▼
┌───────────────────┐
│ build.rs embeds   │
│ files into binary │
└───────────────────┘
    │
    ▼
Request: app://index.html
    │
    ▼
┌───────────────────┐
│ Serve from        │
│ embedded assets   │
└───────────────────┘
```

---

## Window Management

### Window Lifecycle

```
openWindow(opts)
    │
    ├── Create tao::Window
    │       │
    │       └── Configure: size, title, decorations
    │
    ├── Create wry::WebView
    │       │
    │       ├── Load app:// URL
    │       ├── Inject preload.js
    │       └── Set up IPC handlers
    │
    └── Return WindowHandle
            │
            ├── send()/emit() ──► WebView
            ├── setTitle() ──► Window
            └── close() ──► Destroy both
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

## Build Pipeline

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
```
default-src 'self' app:;
script-src 'self' 'unsafe-inline' 'unsafe-eval' app:;
connect-src 'self' ws://localhost:35729;
```

Production (strict):
```
default-src 'self' app:;
script-src 'self' app:;
style-src 'self' 'unsafe-inline' app:;
```

---

## Crate Dependencies

```
forge-host
├── deno_core      # Deno runtime
├── tao            # Window management
├── wry            # WebView
├── muda           # Menus
├── tray-icon      # System tray
├── rfd            # File dialogs
├── notify         # File watching
└── ext_*          # Host modules
```

---

## File Structure

```
forge/
├── crates/
│   ├── forge-host/        # Main runtime binary
│   │   ├── src/
│   │   │   ├── main.rs    # Entry point, event loop
│   │   │   └── capabilities.rs  # Permission system
│   │   └── build.rs       # Asset embedding
│   │
│   ├── forge/             # CLI tool
│   │   └── src/
│   │       ├── main.rs    # CLI commands
│   │       └── tpl/       # App templates
│   │
│   ├── ext_ui/           # host:ui extension
│   ├── ext_fs/           # host:fs extension
│   ├── ext_net/          # host:net extension
│   ├── ext_sys/          # host:sys extension
│   └── ext_process/      # host:process extension
│
├── sdk/                   # TypeScript SDK
│   ├── host.d.ts         # Type definitions
│   ├── host.ui.ts        # UI module
│   ├── host.fs.ts        # FS module
│   └── preload.ts        # Renderer bridge
│
├── apps/                  # Example apps
│   ├── todo-app/
│   ├── weather-app/
│   ├── text-editor/
│   └── system-monitor/
│
└── docs/                  # Documentation
    ├── getting-started.md
    ├── architecture.md
    └── api/
```
