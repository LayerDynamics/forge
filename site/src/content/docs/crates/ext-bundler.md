---
title: "ext_bundler"
description: App packaging and icons extension providing the forge:bundler module.
slug: crates/ext-bundler
---

The `ext_bundler` crate provides app packaging, icon management, and manifest utilities for Forge applications through the `forge:bundler` module.

## Overview

ext_bundler provides runtime access to:

- **Icon management** - Create, validate, and resize app icons
- **Manifest parsing** - Parse and validate manifest.app.toml files
- **Platform info** - Get platform-specific bundle requirements
- **Build configuration** - Manage build settings and paths

## Module: `forge:bundler`

```typescript
import {
  info,
  iconCreate,
  iconValidate,
  iconResize,
  manifestParse,
  sanitizeName,
  platformInfo,
  iconRequirements,
  pathInfo,
  pathJoin
} from "forge:bundler";
```

## Key Types

### Extension Info

```typescript
interface ExtensionInfo {
  name: string;
  version: string;
  capabilities: string[];
}
```

### Icon Types

```typescript
interface IconCreateOptions {
  size?: number;    // Size in pixels (default: 1024)
  color?: string;   // Primary color (hex, default: "#3C5AB8")
}

interface IconValidation {
  width: number;
  height: number;
  isSquare: boolean;
  meetsMinimum: boolean;      // >= 512px
  meetsRecommended: boolean;  // >= 1024px
  hasTransparency: boolean;
  warnings: string[];
  errors: string[];
}

interface IconResizeOptions {
  width: number;
  height: number;
}
```

### Manifest Types

```typescript
interface AppManifest {
  name: string;
  identifier: string;
  version: string;
  icon?: string;
}
```

### Platform Types

```typescript
interface PlatformInfo {
  os: string;            // "macos", "windows", "linux"
  arch: string;          // "x86_64", "aarch64"
  bundleFormat: string;  // "dmg", "msix", "appimage"
  supported: boolean;
}

type BundleFormat =
  | "App"       // macOS .app bundle
  | "Dmg"       // macOS .dmg disk image
  | "Pkg"       // macOS .pkg installer
  | "Msix"      // Windows .msix package
  | "AppImage"  // Linux AppImage
  | "Tarball"   // Compressed tarball
  | "Zip";      // ZIP archive
```

### Build Configuration

```typescript
interface BuildConfig {
  appDir: string;              // App directory path
  outputDir?: string;          // Output directory for bundle
  format?: BundleFormat;       // Target bundle format
  sign?: boolean;              // Whether to sign the bundle
  signingIdentity?: string;    // Code signing identity
}
```

### Path Utilities

```typescript
interface PathInfo {
  path: string;
  exists: boolean;
  isDir: boolean;
  isFile: boolean;
  extension?: string;
  fileName?: string;
  parent?: string;
}
```

## Constants

```typescript
const MIN_ICON_SIZE = 512;           // Minimum recommended icon size
const RECOMMENDED_ICON_SIZE = 1024;  // Optimal icon size for all platforms
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_bundler_info` | `info()` | Get extension information |
| `op_bundler_icon_create` | `iconCreate(opts?)` | Create placeholder icon PNG |
| `op_bundler_icon_validate` | `iconValidate(data)` | Validate icon dimensions and format |
| `op_bundler_icon_resize` | `iconResize(data, opts)` | Resize icon to target dimensions |
| `op_bundler_manifest_parse` | `manifestParse(content)` | Parse manifest.app.toml content |
| `op_bundler_sanitize_name` | `sanitizeName(name)` | Sanitize name for executables |
| `op_bundler_platform_info` | `platformInfo()` | Get current platform bundle info |
| `op_bundler_icon_requirements` | `iconRequirements(platform)` | Get required icon sizes for platform |
| `op_bundler_set_app_dir` | `setAppDir(path)` | Set current app directory |
| `op_bundler_get_app_dir` | `getAppDir()` | Get current app directory |
| `op_bundler_set_build_config` | `setBuildConfig(config)` | Set build configuration |
| `op_bundler_get_build_config` | `getBuildConfig()` | Get build configuration |
| `op_bundler_path_info` | `pathInfo(path)` | Analyze a path |
| `op_bundler_path_join` | `pathJoin(...components)` | Join path components |
| `op_bundler_manifest_path` | `manifestPath(appDir)` | Get manifest path for app |
| `op_bundler_cache_manifest` | `cacheManifest(path, manifest)` | Cache parsed manifest |
| `op_bundler_get_cached_manifest` | `getCachedManifest(path)` | Get cached manifest |

## Usage Examples

### Create a Placeholder Icon

```typescript
import { iconCreate, iconValidate } from "forge:bundler";
import { writeFile } from "runtime:fs";

// Create a 1024x1024 placeholder icon
const iconData = iconCreate({ size: 1024 });

// Validate the icon
const validation = iconValidate(iconData);
console.log(`Icon size: ${validation.width}x${validation.height}`);
console.log(`Valid: ${validation.errors.length === 0}`);

// Save to file
await writeFile("app-icon.png", iconData);
```

### Get Icon Requirements for Platform

```typescript
import { iconRequirements, iconResize } from "forge:bundler";

// Get required sizes for macOS
const macosRequirements = iconRequirements("macos");
// Returns: [16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024]

// Resize icon for each required size
for (const { width, height } of macosRequirements) {
  const resized = iconResize(originalIcon, { width, height });
  await writeFile(`icon_${width}x${height}.png`, resized);
}
```

### Parse App Manifest

```typescript
import { manifestParse, manifestPath } from "forge:bundler";
import { readTextFile } from "runtime:fs";

// Get manifest path
const path = manifestPath("/path/to/my-app");
// Returns: "/path/to/my-app/manifest.app.toml"

// Parse manifest
const content = await readTextFile(path);
const manifest = manifestParse(content);

console.log(`App: ${manifest.name}`);
console.log(`Version: ${manifest.version}`);
console.log(`Identifier: ${manifest.identifier}`);
```

### Configure Build

```typescript
import {
  setBuildConfig,
  getBuildConfig,
  platformInfo
} from "forge:bundler";

// Get platform info
const platform = platformInfo();
console.log(`Platform: ${platform.os}/${platform.arch}`);
console.log(`Bundle format: ${platform.bundleFormat}`);

// Configure build
setBuildConfig({
  appDir: "/path/to/my-app",
  outputDir: "/path/to/output",
  format: "Dmg",
  sign: true,
  signingIdentity: "Developer ID Application: My Company"
});

// Retrieve config later
const config = getBuildConfig();
```

### Path Utilities

```typescript
import { pathInfo, pathJoin, sanitizeName } from "forge:bundler";

// Analyze a path
const info = pathInfo("/path/to/file.txt");
console.log(`Exists: ${info.exists}`);
console.log(`Extension: ${info.extension}`);  // "txt"
console.log(`Filename: ${info.fileName}`);    // "file.txt"

// Join paths
const fullPath = pathJoin("/path", "to", "file.txt");
// Returns: "/path/to/file.txt"

// Sanitize app name for use as executable
const sanitized = sanitizeName("My Awesome App!");
// Returns: "my-awesome-app"
```

## Icon Size Requirements by Platform

### macOS
| Size | Usage |
|------|-------|
| 16x16 | Finder, Dock (retina @1x) |
| 32x32 | Finder, Dock (retina @2x for 16pt) |
| 64x64 | Finder |
| 128x128 | Finder |
| 256x256 | Finder |
| 512x512 | Finder |
| 1024x1024 | App Store |

### Windows
| Size | Usage |
|------|-------|
| 44x44 | App list |
| 50x50 | Start menu |
| 150x150 | Start tiles |
| 300x300 | Start tiles (large) |
| 600x600 | Store listing |

### Linux
| Size | Usage |
|------|-------|
| 16x16 | Tray icons |
| 32x32 | Desktop icons |
| 48x48 | Desktop icons |
| 64x64 | Application menu |
| 128x128 | High-DPI displays |
| 256x256 | High-DPI displays |
| 512x512 | Software center |

## File Structure

```text
crates/ext_bundler/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `image` | Image processing (resize, encode) |
| `toml` | Manifest parsing |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `serde`, `serde_json` | Serialization |
| `thiserror` | Error types |
| `deno_error` | JavaScript error conversion |
| `tracing` | Logging |
| `linkme` | Compile-time symbol collection |

## Related

- [forge:weld](/docs/crates/ext-weld) - Related forge module
- [forge_cli](/docs/crates/forge) - CLI bundling commands
- [forge-weld](/docs/crates/forge-weld) - Build system
