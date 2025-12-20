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
- `runtime:*` module loader for native capability access
- Static HTML serving in development mode

#### M1 - UI & Bridge

- `runtime:window` module with window management (open, close, send, events)
- Preload script for renderer-side `window.host` API
- `app://` custom protocol handler for asset loading
- Asset embedding for production builds via `FORGE_EMBED_DIR`
- Application menu support via `setAppMenu()`
- Context menu support via `showContextMenu()`
- File dialogs (open, save) and message dialogs
- System tray icons with menus
- React, Vue, Svelte, and Minimal templates

#### M2 - Core Operations

- `runtime:fs` module with 14 file system operations
  - Read/write text and binary files
  - Directory operations (readDir, mkdir, remove)
  - File operations (stat, exists, copy, rename)
  - File watching with async iterators
- `runtime:net` module with HTTP fetch capabilities
  - Text and binary response support
  - JSON convenience methods (fetchJson, postJson)
- `runtime:sys` module with 11 system operations
  - System info (OS, arch, hostname, CPU count)
  - Environment variables (get, set, cwd, homeDir, tempDir)
  - Clipboard read/write
  - System notifications
  - Battery/power information
- `runtime:process` module with 7 process operations
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
  - API Reference for all runtime modules
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
- TypeScript type definitions for all runtime modules
- Multiple framework template options

## [1.0.0p-steel-donut] - 2025-12-19

### Added

#### Extension Registry System

- 4-tier extension initialization architecture for predictable dependency resolution
  - Tier 0: ExtensionOnly (no state initialization)
  - Tier 1: SimpleState (basic state, no external dependencies)
  - Tier 2: CapabilityBased (requires capability adapter injection)
  - Tier 3: ComplexContext (requires channels, app info, IPC)
- `ExtensionRegistry` in `crates/forge-runtime/src/ext_registry.rs`
- Automatic state initialization by tier with detailed debug logging

#### New Extensions

- `ext_bundler` → `runtime:bundler` - App bundling operations
- `ext_codesign` → `runtime:codesign` - Code signing for macOS, Windows, Linux
- `ext_dock` → `runtime:dock` - macOS dock integration
- `ext_encoding` → `runtime:encoding` - Text encoding/decoding utilities
- `ext_etcher` → `runtime:etcher` - Documentation generation runtime access
- `ext_image_tools` → `runtime:image_tools` - PNG, SVG, WebP, ICO conversion and manipulation
- `ext_svelte` → `runtime:svelte` - SvelteKit integration
- `ext_web_inspector` → `runtime:web_inspector` - Chrome DevTools Protocol bridge
- `ext_weld` → `runtime:weld` - Runtime access to the binding system

#### Shell Extension Expansion

- Complete shell command implementations in `ext_shell`:
  - `cat`, `cd`, `cp`, `mv`, `echo`, `head`, `mkdir`, `pwd`, `rm`, `sleep`, `xargs`
  - Cross-platform command execution without platform-specific binaries
  - Child process tracking and management
  - Command parsing and execution engine

#### forge-smelt Crate

- New `forge-smelt` crate for binary compilation
- TypeScript/Deno code transpilation
- Binary parsing and compilation utilities

#### forge-etch Restructure

- Complete TypeScript parser implementation
- HTML generation system with Handlebars templates
- AST-based documentation extraction
- Symbol resolution and type analysis
- Visibility and decorator handling

#### Documentation

- `docs/DOCUMENTATION.md` - Comprehensive API documentation guide
- `docs/DOCUMENTATION_PROGRESS.md` - Documentation coverage tracking
- `docs/DOCUMENTATION_STYLE_GUIDE.md` - Style guidelines for docs
- `docs/DOCUMENTATION_TEMPLATES.md` - Documentation templates
- `docs/PERFORMANCE.md` - Performance optimization guide
- `docs/TROUBLESHOOTING.md` - Common issues and solutions
- `docs/TYPE_MAPPING.md` - Rust ↔ TypeScript type mapping reference
- README files for all core extensions (fs, database, debugger, devtools, monitor, path, process, shell, storage, trace, wasm, webview)

#### Examples

- `examples/developer-toolkit` - New developer toolkit example
- `examples/svelte-app` - Restructured with full SvelteKit build system

### Changed

- All extension `build.rs` files updated for improved code generation
- forge-weld extensibility system enhancements
- Capability adapter system improvements in `crates/forge-runtime/src/capabilities.rs`

### Infrastructure

- `.cargo/config.toml` - Cargo configuration for workspace
- Enhanced extension build system with preload generation

## [Unreleased]

### Planned

- CI/CD workflows for automated testing and releases
- Additional platform-specific features
- Enhanced debugging tools
