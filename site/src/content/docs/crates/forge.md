---
title: "forge"
description: Command-line interface for scaffolding, building, and bundling Forge apps.
---

The `forge` crate is the command-line interface (CLI) for Forge. It provides commands for creating, developing, building, and distributing Forge applications.

## Overview

The CLI is the primary entry point for Forge users. It handles:

- **Development mode** - Run apps with hot reload
- **Production builds** - Bundle assets for distribution
- **Platform packaging** - Create native installers
- **Code signing** - Sign packages for distribution
- **Icon management** - Validate and generate app icons

## Getting Started

Copy an example to start a new project:

```bash
# Copy an example
cp -r examples/react-app my-app
cd my-app

# Available examples
# - examples/example-deno-app   Minimal TypeScript
# - examples/react-app          React with TypeScript
# - examples/nextjs-app         Next.js-style patterns
# - examples/svelte-app         Svelte with TypeScript
```

## Commands

### `forge dev`

Run an app in development mode with hot reload:

```bash
forge dev <app-dir>
```

Features:
- Live reload on file changes
- Development-friendly CSP settings
- Console output in terminal

### `forge build`

Build web assets for production:

```bash
forge build <app-dir>
```

Process:
1. Detect framework (React, Vue, Svelte, Minimal)
2. Bundle with esbuild via Deno
3. Transform SFC files (Vue/Svelte)
4. Output to `dist/` directory

### `forge bundle`

Create platform-specific distributable packages:

```bash
forge bundle <app-dir>
```

**Output formats:**
- **macOS:** `.app` bundle + `.dmg` disk image
- **Windows:** `.msix` package
- **Linux:** `.AppImage` or `.tar.gz`

### `forge sign`

Sign a bundled artifact for distribution:

```bash
forge sign [--identity <IDENTITY>] <artifact>
```

Supports:
- macOS code signing with Developer ID
- Windows code signing with certificates
- Notarization for macOS

### `forge icon`

Manage app icons:

```bash
forge icon create <path>     # Create placeholder icon
forge icon validate <app-dir> # Validate icon requirements
```

**Icon requirements:**
- Format: PNG with transparency (RGBA)
- Size: 1024x1024 pixels (minimum 512x512)
- Shape: Square (1:1 aspect ratio)

## Key Types

### Framework

Detected framework type for build configuration:

```rust
enum Framework {
    Minimal,
    React,
    Vue,
    Svelte,
}
```

## File Structure

```text
crates/forge/
├── src/
│   ├── main.rs         # CLI entry point and commands
│   └── bundler/        # Platform bundling logic
│       ├── mod.rs      # Bundler module
│       ├── codesign.rs # Code signing
│       └── icon.rs     # Icon processing
└── build.rs            # Build script
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `anyhow` | Error handling |
| `serde`, `toml` | Manifest parsing |
| `image` | Icon processing |
| `zip` | MSIX package creation |
| `walkdir` | Directory traversal |
| `which` | Binary discovery |
| `dirs` | User directories |

## Related

- [forge-host](/docs/crates/forge-host) - Runtime binary launched by `forge dev`
- [Getting Started](/docs/getting-started) - User guide for CLI usage
