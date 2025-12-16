---
title: "ext_path"
description: Path manipulation extension providing the runtime:path module.
slug: crates/ext-path
---

The `ext_path` crate provides cross-platform path manipulation utilities for Forge applications through the `runtime:path` module.

## Overview

ext_path handles:

- **Path joining** - Combine path segments
- **Path parsing** - Extract dirname, basename, extension
- **Path normalization** - Resolve `.` and `..` segments
- **Absolute paths** - Convert relative to absolute
- **Path comparison** - Check path relationships

## Module: `runtime:path`

```typescript
import {
  join,
  dirname,
  basename,
  extname,
  normalize,
  resolve,
  isAbsolute,
  relative,
  parse,
  format
} from "runtime:path";
```

## Key Types

### Error Types

```rust
enum PathErrorCode {
    Generic = 8700,
    InvalidPath = 8701,
    ResolveFailed = 8702,
}

struct PathError {
    code: PathErrorCode,
    message: String,
}
```

### Path Types

```rust
struct ParsedPath {
    root: String,      // "/" or "C:\\"
    dir: String,       // "/home/user"
    base: String,      // "file.txt"
    name: String,      // "file"
    ext: String,       // ".txt"
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_path_join` | `join(...segments)` | Join path segments |
| `op_path_dirname` | `dirname(path)` | Get directory name |
| `op_path_basename` | `basename(path, ext?)` | Get base name |
| `op_path_extname` | `extname(path)` | Get extension |
| `op_path_normalize` | `normalize(path)` | Normalize path |
| `op_path_resolve` | `resolve(...paths)` | Resolve to absolute |
| `op_path_is_absolute` | `isAbsolute(path)` | Check if absolute |
| `op_path_relative` | `relative(from, to)` | Get relative path |
| `op_path_parse` | `parse(path)` | Parse path into parts |
| `op_path_format` | `format(parts)` | Format parts into path |

## Usage Examples

### Joining Paths

```typescript
import { join, resolve } from "runtime:path";

const configPath = await join("home", "user", ".config", "app.json");
// "/home/user/.config/app.json" (Unix)
// "home\\user\\.config\\app.json" (Windows)

const absolute = await resolve(".", "src", "main.ts");
// "/current/working/dir/src/main.ts"
```

### Parsing Paths

```typescript
import { dirname, basename, extname, parse } from "runtime:path";

const filePath = "/home/user/documents/report.pdf";

const dir = await dirname(filePath);      // "/home/user/documents"
const base = await basename(filePath);    // "report.pdf"
const ext = await extname(filePath);      // ".pdf"

const parts = await parse(filePath);
// { root: "/", dir: "/home/user/documents", base: "report.pdf", name: "report", ext: ".pdf" }
```

### Normalizing Paths

```typescript
import { normalize, relative } from "runtime:path";

const normalized = await normalize("/home/user/../user/./documents");
// "/home/user/documents"

const rel = await relative("/home/user", "/home/user/documents/file.txt");
// "documents/file.txt"
```

### Building Paths

```typescript
import { format } from "runtime:path";

const path = await format({
  root: "/",
  dir: "/home/user",
  base: "file.txt"
});
// "/home/user/file.txt"
```

## File Structure

```text
crates/ext_path/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct ParsedPath {
    pub root: String,
    pub dir: String,
    pub base: String,
    pub name: String,
    pub ext: String,
}

#[weld_op]
#[op2]
#[string]
pub fn op_path_join(
    #[serde] segments: Vec<String>,
) -> Result<String, PathError> {
    // implementation
}

#[weld_op]
#[op2]
#[serde]
pub fn op_path_parse(
    #[string] path: String,
) -> Result<ParsedPath, PathError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_path", "runtime:path")
        .ts_path("ts/init.ts")
        .ops(&["op_path_join", "op_path_dirname", "op_path_basename", "op_path_extname", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_path extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_fs](/docs/crates/ext-fs) - File system operations
- [ext_os_compat](/docs/crates/ext-os-compat) - OS compatibility
- [Architecture](/docs/architecture) - Full system architecture
