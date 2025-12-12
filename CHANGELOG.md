# Changelog

All notable changes to Forge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1] - 2024-12-12

### Added

#### M0 - Bootstrap
- Initial Forge CLI with `init`, `dev`, `build`, and `bundle` commands
- Deno runtime embedding via `deno_core`
- Basic window creation with tao/wry
- `host:*` module loader for native capability access
- Static HTML serving in development mode

#### M1 - UI & Bridge
- `host:ui` module with window management (open, close, send, events)
- Preload script for renderer-side `window.host` API
- `app://` custom protocol handler for asset loading
- Asset embedding for production builds via `FORGE_EMBED_DIR`
- Application menu support via `setAppMenu()`
- Context menu support via `showContextMenu()`
- File dialogs (open, save) and message dialogs
- System tray icons with menus
- React, Vue, Svelte, and Minimal templates

#### M2 - Core Operations
- `host:fs` module with 14 file system operations
  - Read/write text and binary files
  - Directory operations (readDir, mkdir, remove)
  - File operations (stat, exists, copy, rename)
  - File watching with async iterators
- `host:net` module with HTTP fetch capabilities
  - Text and binary response support
  - JSON convenience methods (fetchJson, postJson)
- `host:sys` module with 11 system operations
  - System info (OS, arch, hostname, CPU count)
  - Environment variables (get, set, cwd, homeDir, tempDir)
  - Clipboard read/write
  - System notifications
  - Battery/power information
- `host:process` module with 7 process operations
  - Process spawning with piped I/O
  - Async iterators for stdout/stderr
  - Process control (kill, wait, status)
- Capability-based permission system via manifest.app.toml
- Structured error codes (1000-4000 range)

#### M3 - Packaging
- `forge build` command for bundling Deno code
- `forge bundle` command for platform-specific packages
  - macOS: .app bundle with DMG creation
  - Windows: MSIX package support
  - Linux: AppImage generation
- `forge sign` command for code signing
- Build-time asset embedding

#### M4 - Framework Transforms & HMR
- Vue SFC transform support via `@vue/compiler-sfc`
- Svelte component transform support
- Development server with hot module replacement (HMR)
- WebSocket-based live reload on port 35729
- File watcher integration for automatic rebuilds

#### M5 - Polishing & Documentation
- Example applications demonstrating key features:
  - Todo App (React) - File persistence, menus, IPC
  - Weather App (Vue) - HTTP fetch, notifications, tray icons
  - Text Editor (Svelte) - Full file ops, dialogs, context menus
  - System Monitor - System info, multi-window
- Comprehensive documentation:
  - Getting Started guide
  - API Reference for all host modules
  - Architecture overview
  - Manifest schema reference
- Default-deny channel allowlists for security
- Template enhancements with capability examples

### Security

- Capability-based permission model
- Explicit channel allowlists for IPC (default-deny)
- Content Security Policy (strict in production, relaxed for HMR in dev)
- Path validation and glob pattern matching for file access

### Developer Experience

- `tracing` with `env-filter` for configurable logging via `FORGE_LOG`
- Structured error messages with capability context
- TypeScript type definitions for all host modules
- Multiple framework template options

## [Unreleased]

### Planned
- CI/CD workflows for automated testing and releases
- Additional platform-specific features
- Plugin system for extensibility
- Enhanced debugging tools
