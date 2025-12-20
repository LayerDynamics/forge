---
title: "ext_path - Path Manipulation"
description: Cross-platform path manipulation utilities for Forge applications
slug: crates/ext-path
---

The `ext_path` crate provides pure string-based path manipulation utilities through the `runtime:path` module. All operations work consistently across platforms without requiring filesystem access or permissions.

## Overview

Path manipulation is a fundamental need in desktop applications. The `runtime:path` module provides cross-platform utilities for building, parsing, and extracting components from file paths.

**Key Capabilities:**
- 🔀 Join path segments with platform-appropriate separators
- 📂 Extract directory names, basenames, and extensions
- 🔍 Parse paths into structured components
- 🌍 Automatic cross-platform separator handling
- ⚡ Pure string operations - no filesystem access
- 🛡️ No permissions required

## Quick Start

```typescript
import { join, dirname, basename, extname, parts } from "runtime:path";

// Join path segments - automatically uses correct separators
const configPath = join("./data", "config.json");
// Unix: "./data/config.json"
// Windows: ".\\data\\config.json"

// Extract components
const dir = dirname("/usr/local/bin/node");   // "/usr/local/bin"
const file = basename("/usr/local/bin/node"); // "node"
const ext = extname("document.pdf");          // ".pdf"

// Parse complete path
const p = parts("./logs/app.log");
console.log(p.dir);  // "./logs"
console.log(p.base); // "app.log"
console.log(p.ext);  // ".log"
```

## Module: `runtime:path`

Import path manipulation functions:

```typescript
import {
  join,      // Combine path segments
  dirname,   // Extract directory path
  basename,  // Extract filename
  extname,   // Extract file extension
  parts      // Parse into components
} from "runtime:path";
```

## Core Concepts

### Pure String Operations

All path operations are **pure functions** - they transform strings without accessing the filesystem:

```typescript
// These work even if the paths don't exist
const path1 = join("./nonexistent", "file.txt");
const path2 = dirname("/fake/path/file.txt");
const ext = extname("imaginary.jpg");
```

### Platform-Appropriate Separators

The extension automatically uses the correct path separator for your platform:

```typescript
// Same code, different output per platform
const path = join("data", "config", "app.json");

// Unix (macOS/Linux): "data/config/app.json"
// Windows: "data\\config\\app.json"
```

### Empty Results for Missing Components

Functions return empty strings when components don't exist (never errors):

```typescript
dirname("file.txt");      // "" (no directory)
extname("README");        // "" (no extension)
basename("/path/to/");    // "" (ends with separator)
```

## API Reference

### Types

#### `PathParts`

Result of parsing a path into components:

```typescript
interface PathParts {
  dir: string;   // Directory path (empty if no directory)
  base: string;  // Base filename including extension
  ext: string;   // File extension including dot (empty if no extension)
}
```

### Functions

#### `join(base, ...segments): string`

Joins path segments into a single path using platform-appropriate separators.

**Parameters:**
- `base: string` - The base path to start from
- `...segments: string[]` - Additional path segments to append

**Returns:** `string` - Combined path with platform-appropriate separators

**Examples:**

```typescript
// Basic joining
join("./data", "config.json")
// Unix: "./data/config.json"
// Windows: ".\\data\\config.json"

// Multiple segments
join("./assets", "images", "logo.png")
// Unix: "./assets/images/logo.png"
// Windows: ".\\assets\\images\\logo.png"

// Absolute paths
join("/usr", "local", "bin", "node")
// Unix: "/usr/local/bin/node"
```

#### `dirname(path): string`

Extracts the directory path from a file path.

**Parameters:**
- `path: string` - The path to extract the directory from

**Returns:** `string` - The directory portion, or empty string if none

**Examples:**

```typescript
dirname("/usr/local/bin/node")   // "/usr/local/bin"
dirname("./data/config.json")    // "./data"
dirname("file.txt")              // "" (no directory)
dirname("/path/to/")             // "/path/to"
```

#### `basename(path): string`

Extracts the final component of a path (filename with extension).

**Parameters:**
- `path: string` - The path to extract the basename from

**Returns:** `string` - The filename portion, or empty string if none

**Examples:**

```typescript
basename("/usr/local/bin/node")  // "node"
basename("./data/config.json")   // "config.json"
basename("readme.md")            // "readme.md"
basename("/path/to/")            // "" (ends with separator)
```

#### `extname(path): string`

Extracts the file extension from a path.

**Parameters:**
- `path: string` - The path to extract the extension from

**Returns:** `string` - The extension including the dot, or empty string if none

**Examples:**

```typescript
extname("file.txt")           // ".txt"
extname("archive.tar.gz")     // ".gz" (only last extension)
extname("README")             // "" (no extension)
extname(".gitignore")         // "" (dot prefix is not an extension)
extname(".config.json")       // ".json" (has real extension)
```

#### `parts(path): PathParts`

Parses a path into its directory, basename, and extension components.

**Parameters:**
- `path: string` - The path to parse

**Returns:** `PathParts` - Object with `dir`, `base`, and `ext` properties

**Examples:**

```typescript
parts("/usr/local/bin/node")
// { dir: "/usr/local/bin", base: "node", ext: "" }

parts("./data/config.json")
// { dir: "./data", base: "config.json", ext: ".json" }

parts("file.txt")
// { dir: "", base: "file.txt", ext: ".txt" }
```

## Usage Examples

### Building Dynamic File Paths

Construct paths programmatically with correct separators:

```typescript
import { join } from "runtime:path";

function getLogPath(appName: string, date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');

  return join("./logs", appName, `${year}-${month}-${day}.log`);
}

const logPath = getLogPath("myapp", new Date());
// "./logs/myapp/2025-12-19.log"
```

### Validating File Extensions

Check file types using extension extraction:

```typescript
import { extname } from "runtime:path";

function isImageFile(path: string): boolean {
  const ext = extname(path).toLowerCase();
  return ['.jpg', '.jpeg', '.png', '.gif', '.webp'].includes(ext);
}

function isMarkdownFile(path: string): boolean {
  return extname(path).toLowerCase() === '.md';
}

console.log(isImageFile("photo.jpg"));     // true
console.log(isImageFile("document.pdf"));  // false
console.log(isMarkdownFile("README.md"));  // true
```

### Generating Output Paths

Create modified versions of existing paths:

```typescript
import { parts, join } from "runtime:path";

// Add suffix before extension
function getOutputPath(inputPath: string, suffix: string): string {
  const p = parts(inputPath);
  const baseName = p.base.slice(0, -p.ext.length);
  return join(p.dir, `${baseName}${suffix}${p.ext}`);
}

const output = getOutputPath("./video.mp4", ".compressed");
console.log(output); // "./video.compressed.mp4"

// Create thumbnail path
function getThumbnailPath(imagePath: string): string {
  const p = parts(imagePath);
  return join(p.dir, `thumb_${p.base}`);
}

const thumb = getThumbnailPath("./images/photo.jpg");
console.log(thumb); // "./images/thumb_photo.jpg"
```

### Path Component Analysis

Extract and analyze path components:

```typescript
import { dirname, basename, extname } from "runtime:path";

function analyzeFilePath(path: string) {
  const ext = extname(path);
  const base = basename(path);
  const nameWithoutExt = base.slice(0, -ext.length);

  return {
    directory: dirname(path),
    filename: base,
    extension: ext,
    nameOnly: nameWithoutExt
  };
}

const info = analyzeFilePath("./docs/guide.md");
console.log(info);
// {
//   directory: "./docs",
//   filename: "guide.md",
//   extension: ".md",
//   nameOnly: "guide"
// }
```

### Integration with Filesystem

Combine with `runtime:fs` for file operations:

```typescript
import { join, extname } from "runtime:path";
import { readTextFile, writeTextFile, readDir } from "runtime:fs";

// Load config file from app directory
async function loadConfig(appDir: string): Promise<object> {
  const configPath = join(appDir, "config.json");
  const content = await readTextFile(configPath);
  return JSON.parse(content);
}

// Process all markdown files in directory
async function processMarkdownFiles(dir: string): Promise<void> {
  const entries = await readDir(dir);

  for (const entry of entries) {
    if (entry.isFile && extname(entry.name) === '.md') {
      const filePath = join(dir, entry.name);
      const content = await readTextFile(filePath);
      const processed = content.toUpperCase(); // Example processing
      await writeTextFile(filePath, processed);
    }
  }
}
```

## Best Practices

### ✅ Do: Use `join()` for All Path Construction

Always use `join()` instead of manual string concatenation:

```typescript
// ✅ Correct - cross-platform
const path = join(baseDir, "data", "config.json");

// ❌ Wrong - breaks on Windows
const path = `${baseDir}/data/config.json`;
const path = baseDir + "/data/config.json";
```

### ✅ Do: Let the Extension Handle Separators

Don't hardcode path separators:

```typescript
// ✅ Correct
const parts = ["home", "user", "documents"];
const path = join(...parts);

// ❌ Wrong
const path = parts.join('/');  // Breaks on Windows
```

### ✅ Do: Use `parts()` for Complex Path Manipulation

When you need multiple components, use `parts()`:

```typescript
// ✅ Correct - single operation
const p = parts(filepath);
const newPath = join(p.dir, `modified_${p.base}`);

// ❌ Less efficient - multiple operations
const dir = dirname(filepath);
const base = basename(filepath);
const newPath = join(dir, `modified_${base}`);
```

### ✅ Do: Check for Empty Results

Handle cases where components don't exist:

```typescript
const dir = dirname(filepath);
if (dir === "") {
  // File is in current directory or has no directory component
  console.log("No directory component");
}
```

## Common Pitfalls

### ❌ Assuming Separators

Don't split paths on hardcoded separators:

```typescript
// ❌ Wrong - breaks on Windows
const parts = filepath.split('/');

// ✅ Correct - use dirname and basename
const dir = dirname(filepath);
const file = basename(filepath);
```

### ❌ Concatenating Paths Manually

Don't build paths with string concatenation:

```typescript
// ❌ Wrong - separator issues
const path = dir + '/' + filename;

// ✅ Correct - use join
const path = join(dir, filename);
```

### ❌ Expecting Multiple Extensions

`extname()` only returns the last extension:

```typescript
const ext = extname("archive.tar.gz");  // ".gz" not ".tar.gz"

// If you need all extensions, use string operations:
const fullPath = "archive.tar.gz";
const allExts = fullPath.substring(fullPath.indexOf('.')); // ".tar.gz"
```

### ❌ Treating Dot Prefix as Extension

Hidden files with dot prefixes don't have extensions:

```typescript
extname(".gitignore")  // "" (not ".gitignore")

// Files with both prefix and extension work correctly:
extname(".config.json") // ".json"
```

## Edge Cases

### Hidden Files

Dot prefixes are not treated as file extensions:

```typescript
basename(".gitignore")  // ".gitignore"
extname(".gitignore")   // ""

basename(".config.json")  // ".config.json"
extname(".config.json")   // ".json"
```

### Trailing Separators

Paths ending with separators return empty basename:

```typescript
dirname("/path/to/")   // "/path/to"
basename("/path/to/")  // ""
```

### Root Paths

Root directory behavior:

```typescript
dirname("/")           // "/"
dirname("C:\\")       // "C:\\" (Windows)
basename("/")         // ""
```

### Empty Strings

All functions handle empty input gracefully:

```typescript
dirname("")    // ""
basename("")   // ""
extname("")    // ""
parts("")      // { dir: "", base: "", ext: "" }
```

## Platform Support

### Cross-Platform Separators

| Platform | Separator | Example |
|----------|-----------|---------|
| Unix (macOS/Linux) | `/` | `/usr/local/bin` |
| Windows | `\` | `C:\Program Files` |

The extension automatically uses the correct separator for the current platform.

### Forward Slash Input

You can use forward slashes in your TypeScript code - they work on all platforms as input:

```typescript
// This works on all platforms (forward slashes in input)
const path = join("./data", "config.json");

// Output automatically uses platform separator:
// Unix: "./data/config.json"
// Windows: ".\\data\\config.json"
```

## No Permissions Required

Unlike filesystem operations (`runtime:fs`), path utilities:
- ✅ Don't access the filesystem
- ✅ Don't require permissions in `manifest.app.toml`
- ✅ Work with any path string (existing or not)
- ✅ Are pure string transformations
- ✅ Never fail or throw errors

**No configuration needed:**

```toml
# manifest.app.toml
# No [permissions.path] section needed - operations are always allowed
```

## Implementation

### Rust Backend

Operations use Rust's `std::path::Path` for platform-specific handling:

```rust
use std::path::{Path, PathBuf};

fn path_join(base: String, segments: Vec<String>) -> String {
    let mut pb = PathBuf::from(base);
    for seg in segments {
        pb.push(seg);
    }
    pb.to_string_lossy().to_string()
}

fn path_dirname(path: &str) -> String {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn path_extname(path: &str) -> String {
    Path::new(&path)
        .extension()
        .map(|p| format!(".{}", p.to_string_lossy()))
        .unwrap_or_default()
}
```

### Extension Registration

Registered as a **Tier 0 (ExtensionOnly)** extension - no state initialization required:

```rust
ExtensionDescriptor {
    id: "runtime_path",
    init_fn: ExtensionInitFn::ExtensionOnly(path_extension),
    tier: ExtensionTier::ExtensionOnly,
    required: false,
}
```

### Code Generation

TypeScript bindings generated via `forge-weld`:

```rust
// build.rs
ExtensionBuilder::new("runtime_path", "runtime:path")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_path_join",
        "op_path_dirname",
        "op_path_basename",
        "op_path_extname",
        "op_path_parts"
    ])
    .use_inventory_types()
    .generate_sdk_module("../../sdk")
    .build()
```

## File Structure

```text
crates/ext_path/
├── src/
│   └── lib.rs        # Path manipulation implementation
├── ts/
│   └── init.ts       # TypeScript module with JSDoc
├── build.rs          # forge-weld configuration
├── Cargo.toml        # Crate metadata
└── README.md         # Developer documentation
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions and extension system |
| `serde` | Serialization for PathParts struct |
| `forge-weld-macro` | `#[weld_op]` and `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related Extensions

- [ext_fs](/docs/crates/ext-fs) - Filesystem operations (read, write, stat)
- [ext_process](/docs/crates/ext-process) - Process spawning with working directories
- [ext_os_compat](/docs/crates/ext-os-compat) - OS compatibility utilities

## See Also

- [Getting Started Guide](/docs/getting-started) - Introduction to Forge
- [Architecture](/docs/architecture) - System architecture overview
- [Rust std::path](https://doc.rust-lang.org/std/path/index.html) - Underlying Rust implementation
