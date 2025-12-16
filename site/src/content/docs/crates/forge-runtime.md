---
title: "forge-runtime"
description: Main runtime binary that embeds Deno and manages windows.
slug: crates/forge-runtime
---

The `forge-runtime` crate is the main runtime executable that runs Forge applications. It embeds the Deno runtime and manages native windows, IPC, and system integration.

## Overview

`forge-runtime` is launched by `forge dev` or bundled into the final application. It handles:

- **Deno runtime** - Embeds JsRuntime for executing TypeScript/JavaScript
- **Window management** - Creates and manages native windows via tao/wry
- **IPC bridge** - Routes messages between Deno and WebView renderers
- **Module loading** - Resolves `runtime:*` imports to extension modules
- **Asset serving** - Serves `app://` protocol from filesystem or embedded assets
- **Hot reload** - WebSocket server for development hot module reload

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                       forge-runtime                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐      ┌─────────────────────────────┐   │
│  │  Deno JsRuntime │◄────►│       Event Loop (tao)      │   │
│  │   (app logic)   │      │   (windows, menus, tray)    │   │
│  └────────┬────────┘      └──────────────┬──────────────┘   │
│           │                              │                  │
│           │ runtime:* ops                │ WebView IPC      │
│           ▼                              ▼                  │
│  ┌─────────────────┐      ┌─────────────────────────────┐   │
│  │   Extensions    │      │     WebView (wry)           │   │
│  │ fs,net,ui,ipc.. │      │  (renders app:// content)   │   │
│  └─────────────────┘      └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Module Loading

The `ForgeModuleLoader` handles module resolution:

1. **`runtime:*` specifiers** - Maps to `ext:runtime_*/init.js`
2. **TypeScript files** - Transpiled via `deno_ast`
3. **JavaScript files** - Loaded directly
4. **JSON files** - Parsed as JSON modules

```rust
// runtime:fs → ext:runtime_fs/init.js
// runtime:window → ext:runtime_window/init.js
```

## Asset Protocol

The `app://` protocol serves web assets:

- **Development mode:** Files read from `{app_dir}/web/`
- **Production mode:** Files embedded in binary via `build.rs`

```rust
// app://index.html → {app_dir}/web/index.html
// app://styles/main.css → {app_dir}/web/styles/main.css
```

## Event Loop

The main event loop (via `tao`) handles:

- Window events (close, resize, focus, move)
- Menu events (app menu, context menu)
- Tray events (click, menu selection)
- IPC messages (renderer → Deno)
- HMR events (file watcher → WebSocket)

## Manifest

Applications are configured via `manifest.app.toml`:

```toml
[app]
name = "My App"
identifier = "com.example.myapp"
version = "0.1.0"
crash_reporting = true

[windows]
width = 800
height = 600
resizable = true

[permissions]
fs = { read = ["~/.myapp/*"], write = ["~/.myapp/*"] }
```

## Key Types

### Manifest

Application configuration parsed from `manifest.app.toml`:

```rust
struct Manifest {
    app: App,
    windows: Option<Windows>,
    permissions: Option<Permissions>,
}

struct App {
    name: String,
    identifier: String,
    version: String,
    crash_reporting: Option<bool>,
}
```

## Preload Script

The preload script (from `sdk/preload.ts`) is injected into every WebView:

- Provides `window.runtime.send()` and `window.runtime.on()` API
- Bridges renderer to Deno via IPC
- Handles `__host_dispatch` for Deno → renderer messages

## File Structure

```text
crates/forge-runtime/
├── src/
│   ├── main.rs         # Entry point, event loop, runtime setup
│   ├── capabilities.rs # Permission system adapters
│   └── crash.rs        # Crash reporting
├── build.rs            # Asset embedding, preload compilation
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | JavaScript runtime |
| `deno_ast` | TypeScript transpilation |
| `tao` | Window management, event loop |
| `wry` | WebView rendering |
| `tokio` | Async runtime |
| `notify` | File watching (HMR) |
| `muda` | Menu system |
| `tray-icon` | System tray |
| `rfd` | File dialogs |

## Related

- [forge](/docs/crates/forge) - CLI (crate `forge_cli`, binary `forge`) that launches forge-runtime
- [ext_window](/docs/crates/ext-window) - Window management extension
- [ext_ipc](/docs/crates/ext-ipc) - IPC extension
- [Architecture](/docs/architecture) - Full system architecture
