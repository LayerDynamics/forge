# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Forge is an Electron-like desktop application framework using Rust + Deno. It embeds Deno for app logic (TypeScript/JavaScript) and uses system WebViews (via wry/tao) for UI rendering. Apps are 100% Deno—no per-app Rust required.

**Status:** Alpha (0.1.0-alpha.1)

## User Commands (after installation)

```bash
# Copy an example to start a new app
cp -r examples/react-app my-new-app

# Run app in dev mode
forge dev my-app

# Build web assets for production
forge build my-app

# Create distributable package (.app/.dmg on macOS, .msix on Windows, AppImage on Linux)
forge bundle my-app

# Code sign a bundled artifact
forge sign my-app/bundle/MyApp.dmg

# Manage app icons
forge icon create my-app/assets/icon.png
forge icon validate my-app
```

## Development Commands (for Forge contributors)

```bash
# Build the CLI and runtime
cargo build -p forge_cli
cargo build -p forge-runtime

# Run sample app via cargo (development)
cargo run -p forge_cli -- dev examples/example-deno-app

# Run tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p ext_fs

# Check code compiles without building
cargo check --workspace

# Format code
cargo fmt --all

# Lint with clippy (CI enforces -D warnings)
cargo clippy --workspace -- -D warnings

# Build everything in release mode
cargo build --workspace --release
```

### Linux System Dependencies

On Linux, install these before building:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev libxdo-dev
```

## Architecture

### Core Crates

| Crate | Purpose |
|-------|---------|
| `forge-runtime` | Main runtime binary. Embeds Deno JsRuntime, creates windows via tao/wry, handles IPC, event loop |
| `forge_cli` | CLI tool (`forge dev/build/bundle/sign/icon`). Contains bundler for all platforms |
| `forge-weld` | Code generation framework for TypeScript bindings from Rust ops |
| `forge-weld-macro` | Proc macros (`#[weld_op]`, `#[weld_struct]`, `#[weld_enum]`) |

### Extension Crates (runtime:* modules)

Each `ext_*` crate provides a `runtime:*` module accessible from TypeScript. There are 27 extension crates total, including:

**Core Extensions (fully implemented):**
- `ext_fs` → `runtime:fs` - File operations (read, write, watch, stat)
- `ext_window` → `runtime:window` - Window management, menus, trays, dialogs
- `ext_ipc` → `runtime:ipc` - Deno ↔ Renderer communication
- `ext_net` → `runtime:net` - HTTP fetch, network operations
- `ext_sys` → `runtime:sys` - System info, clipboard, notifications
- `ext_process` → `runtime:process` - Spawn child processes
- `ext_wasm` → `runtime:wasm` - WebAssembly module loading
- `ext_app` → `runtime:app` - App lifecycle, info
- `ext_crypto` → `runtime:crypto` - Cryptographic operations
- `ext_storage` → `runtime:storage` - Persistent key-value storage

**Additional Extensions:** `ext_shell`, `ext_database`, `ext_webview`, `ext_devtools`, `ext_timers`, `ext_shortcuts`, `ext_signals`, `ext_updater`, `ext_monitor`, `ext_display`, `ext_log`, `ext_trace`, `ext_lock`, `ext_path`, `ext_protocol`, `ext_os_compat`, `ext_debugger`

### Runtime Flow

1. `forge-runtime` parses `manifest.app.toml` from app directory
2. Creates Deno JsRuntime with all `ext_*` extensions registered
3. Executes app's `src/main.ts` which imports from `runtime:*` modules
4. Windows load `app://` URLs served from `web/` directory
5. Renderer communicates with Deno via `window.host.send()/on()` bridge

### Forge Weld (Binding System)

The `forge-weld` system generates TypeScript bindings from Rust:

```rust
// In ext_fs/src/lib.rs - annotate ops for TypeScript generation
#[weld_op(async)]
#[op2(async)]
pub async fn op_fs_read_text(#[string] path: String) -> Result<String, FsError> { ... }

// In ext_fs/build.rs - configure code generation
ExtensionBuilder::new("runtime_fs", "runtime:fs")
    .ts_path("ts/init.ts")
    .ops(&["op_fs_read_text", ...])
    .generate_sdk_module("sdk")
    .use_inventory_types()
    .build()
```

This generates:
- `sdk/runtime.fs.ts` - TypeScript SDK module with full types
- `ts/init.ts` → `init.js` - JavaScript shim loaded by Deno

### IPC Communication

**Renderer → Deno:**
```
window.host.send(channel, data) → WebView IPC → mpsc channel → windowEvents()
```

**Deno → Renderer:**
```
sendToWindow(windowId, channel, data) → evaluate_script() → window.__host_dispatch()
```

### Asset Embedding

Build with `FORGE_EMBED_DIR` env var to embed web assets into the binary:
- **Unset**: Dev mode, assets served from filesystem
- **Set**: Assets embedded via generated `assets.rs`

## SDK Structure

TypeScript SDK files in `sdk/`:
- `runtime.*.ts` - Generated SDK modules (one per extension)
- `generated/*.d.ts` - Type declarations

Apps import from `runtime:*` specifiers which resolve to extension modules.

## App Structure

```
my-app/
├── manifest.app.toml   # App metadata, window config, permissions
├── deno.json           # Deno configuration
├── src/main.ts         # Deno entry point
└── web/                # Static assets served via app:// protocol
    └── index.html
```

## Key Implementation Files

- `crates/forge-runtime/src/main.rs` - Runtime entry, event loop, module loader
- `crates/forge-runtime/src/capabilities.rs` - Permission system
- `crates/forge_cli/src/main.rs` - CLI commands
- `crates/forge_cli/src/bundler/` - Platform bundling (macos.rs, windows.rs, linux.rs)
- `crates/ext_window/src/manager.rs` - WindowManager implementation
- `crates/forge-weld/src/build/extension.rs` - ExtensionBuilder for code generation

## Adding a New Extension

1. Create `crates/ext_<name>/` with `Cargo.toml`, `src/lib.rs`, `build.rs`, `ts/init.ts`
2. Use `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros on Rust types
3. Configure `ExtensionBuilder` in `build.rs` to generate SDK
4. Register extension in `forge-runtime/src/main.rs`
5. Initialize state in the runtime's op_state setup
