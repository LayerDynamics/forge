# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Forge is an Electron-like desktop application framework using Rust + Deno. It embeds Deno for app logic (TypeScript/JavaScript) and uses system WebViews (via wry/tao) for UI rendering. Apps are 100% Deno—no per-app Rust required.

## User Commands (after installation)

```bash
# Scaffold a new app
forge init my-new-app

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

## Development Commands (for Forge contributors only)

**Note:** End users building apps with Forge do NOT need Rust or cargo. They install Forge via the install script (`curl -fsSL https://forge-deno.com/install.sh | sh`) and use only the `forge` CLI commands above.

These cargo commands are only for developers contributing to the Forge framework itself:

```bash
# Build the CLI and host runtime
cargo build -p forge
cargo build -p forge-host

# Run sample app via cargo (development)
cargo run -p forge -- dev apps/example-deno-app

# Run tests
cargo test
```

## Architecture

### Crate Structure

- **`crates/forge-host`**: Main runtime binary. Embeds Deno JsRuntime, creates windows via tao/wry, handles IPC between Deno and WebView renderers. Contains `build.rs` for asset embedding.
- **`crates/forge`**: CLI tool (`forge init/dev/build/bundle`). Scaffolds apps and orchestrates forge-host.
- **`crates/ext_fs`**: Rust extension providing `host:fs` module (file operations exposed to Deno).
- **`crates/ext_ui`**: Rust extension providing `host:ui` module (window management, IPC bridge).

### Runtime Flow

1. `forge-host` parses `manifest.app.toml` from app directory
2. Creates Deno JsRuntime with `ext_fs` and `ext_ui` extensions
3. Executes app's `src/main.ts` which calls `host:ui` to open windows
4. Windows load `app://` URLs served from `web/` directory (filesystem in dev, embedded in release)
5. Renderer communicates with Deno via `window.host.send()/on()` bridge (IPC through wry)

### Host Module System

Apps import native capabilities from `host:*` specifiers:

```typescript
import { readTextFile } from "host:fs";
import { openWindow, sendToWindow, windowEvents } from "host:ui";
```

These resolve to ESM shims in `crates/ext_*/js/*.js` that call Rust ops via `Deno.core.ops.*`.

### IPC Channels (ext_ui)

- **Deno → Renderer**: `sendToWindow(windowId, channel, payload)` triggers `window.__host_dispatch`
- **Renderer → Deno**: `window.host.send(channel, payload)` posts to IPC handler
- **Event loop**: `windowEvents()` async generator yields incoming renderer messages

### Asset Embedding

`build.rs` checks `FORGE_EMBED_DIR` env var:

- **Set**: Embeds all files from that directory into binary via generated `assets.rs`
- **Unset**: Dev mode, assets served from filesystem

## App Structure

```text
apps/example-deno-app/
├── manifest.app.toml   # App metadata, window config, permissions
├── deno.json           # Deno config
├── src/main.ts         # Deno entry point (calls host:ui, host:fs)
└── web/                # Static assets served via app:// protocol
    └── index.html
```

## Key Files

- `crates/forge-host/src/main.rs`: Runtime entry, event loop, WebView creation
- `crates/ext_ui/src/lib.rs`: Window ops (`op_ui_open_window`, `op_ui_window_send/recv`)
- `crates/ext_ui/js/preload.js`: Injected into WebView, provides `window.host` API
- `crates/ext_fs/src/lib.rs`: File ops (`op_fs_read_text`)

## Current State

Implemented features:
- Window creation and `app://` loading
- `forge init` with templates (minimal, react, vue, svelte)
- `forge dev` for development mode
- `forge build` for web asset bundling (esbuild via Deno)
- `forge bundle` for platform packaging (macOS .app/.dmg, Windows .msix, Linux AppImage)
- `forge sign` for code signing
- `forge icon` for icon management

Planned features:
- Permissions/capabilities system
- Additional host modules (host:net, host:sys, host:process)
- Hot reload during dev
