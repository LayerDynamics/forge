---
title: "forge"
description: Command-line interface for scaffolding, building, and bundling Forge apps.
slug: docs/crates/forge
---

The `forge` CLI (crate name `forge_cli`, binary `forge`, path `crates/forge_cli`) is the command-line interface for Forge. It provides commands for creating, developing, building, and distributing Forge applications.

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

### `forge smelt`

Ahead-of-time compile an app's TypeScript to JavaScript:

```bash
forge smelt <app-dir> [--out <dir>] [--embed]
```

- `--out <dir>` - Output directory for the compiled `.js` tree (defaults next to the source).
- `--embed` - Also write the bootstrap shim for embedding the app into a standalone binary.

`forge build` runs smelt automatically, so production bundles ship compiled JavaScript with no launch-time transpile. See [forge-smelt](/docs/crates/forge-smelt) for details.

### `forge docs`

Generate API documentation from extension TypeScript/Rust source:

```bash
forge docs <app-dir>                 # Document an app's src/main.ts
forge docs --extension fs            # Document a single extension
forge docs --all-extensions          # Document every runtime extension
```

Options:

- `--output, -o <dir>` - Output directory (default: `docs`).
- `--format, -f <astro|html|both>` - Output format (default: `astro`).

The extension list is discovered from `crates/ext_*` (no hardcoded list), so new extensions are documented automatically.

## CLI reference

The reference below is generated from the `forge_cli` clap command model — the same definition the binary parses — so it always matches the arguments and options `forge` actually accepts. Run `make docs-cli` to refresh it.

<!-- forge:cli -->
<!-- generated from the forge_cli clap model — run `make docs-cli` to refresh -->

**`forge build`** — Build an app's web assets for production

```text
forge build <APP_DIR>
```

Arguments:
- `<APP_DIR>` — App directory

**`forge bundle`** — Package an app into a platform distributable (.app/.dmg, .msix, AppImage)

```text
forge bundle <APP_DIR>
```

Arguments:
- `<APP_DIR>` — App directory

**`forge dev`** — Run an app in development mode (hot reload, full debugging)

```text
forge dev <APP_DIR>
```

Arguments:
- `<APP_DIR>` — App directory (contains manifest.app.toml)

**`forge docs`** — Generate API documentation from extension TypeScript/Rust source

```text
forge docs [ARGS]...
```

Arguments:
- `<ARGS>` — Options/target forwarded to the docs generator: --all-extensions, --extension <name>, --output <dir>, --format <astro|html|both>

**`forge icon`** — Manage app icons

```text
forge icon <COMMAND>
```

**`forge icon create`** — Create the default Forge-branded icon at <path>

```text
forge icon create <PATH>
```

Arguments:
- `<PATH>` — Output path for the icon (PNG)

**`forge icon validate`** — Validate an app's icon meets platform requirements

```text
forge icon validate [APP_DIR]
```

Arguments:
- `<APP_DIR>` — App directory (defaults to the current directory)

**`forge sign`** — Code-sign a bundled artifact for distribution

```text
forge sign [OPTIONS] <ARTIFACT>
```

Arguments:
- `<ARTIFACT>` — The bundled artifact to sign

Options:
- `--identity, -i <IDENTITY>` — Signing identity (e.g. "Developer ID Application: Name (TEAM)")

**`forge smelt`** — Ahead-of-time compile an app's TypeScript to JavaScript

```text
forge smelt [OPTIONS] <APP_DIR>
```

Arguments:
- `<APP_DIR>` — App directory

Options:
- `--out, -o <OUT>` — Output directory for the compiled JavaScript tree
- `--embed` — Also write the standalone-binary bootstrap shim
<!-- /forge:cli -->

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
crates/forge_cli/
├── src/
│   ├── main.rs         # CLI entry point and commands
│   ├── lib.rs          # clap command model (introspected to generate CLI docs)
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

- [forge-runtime](/docs/crates/forge-runtime) - Runtime binary launched by `forge dev`
- [Getting Started](/docs/getting-started) - User guide for CLI usage
