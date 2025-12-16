---
title: "ext_os_compat"
description: OS compatibility helpers extension providing the runtime:os_compat module.
slug: crates/ext-os-compat
---

The `ext_os_compat` crate provides cross-platform OS compatibility utilities for Forge applications through the `runtime:os_compat` module.

## Overview

ext_os_compat handles:

- **Platform detection** - OS name, version, architecture
- **Path separators** - Platform-specific path/env separators
- **Line endings** - Platform-appropriate line terminators
- **Home directory** - User home directory resolution

## Module: `runtime:os_compat`

```typescript
import {
  getInfo,
  pathSep,
  envSep,
  lineEnding,
  homeDir
} from "runtime:os_compat";
```

## Key Types

### Error Types

```rust
enum OsCompatErrorCode {
    Generic = 8600,
    InfoFailed = 8601,
    NotSupported = 8602,
}

struct OsCompatError {
    code: OsCompatErrorCode,
    message: String,
}
```

### OS Info Types

```rust
struct OsInfo {
    name: String,        // "macos", "windows", "linux"
    version: String,     // "14.0.0", "10.0.22621", etc.
    arch: String,        // "x86_64", "aarch64"
    family: String,      // "unix", "windows"
}

struct PathInfo {
    separator: char,     // '/' or '\\'
    delimiter: char,     // ':' or ';'
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_os_compat_info` | `getInfo()` | Get OS information |
| `op_os_compat_path_sep` | `pathSep()` | Get path separator |
| `op_os_compat_env_sep` | `envSep()` | Get environment path delimiter |
| `op_os_compat_line_ending` | `lineEnding()` | Get platform line ending |
| `op_os_compat_home_dir` | `homeDir()` | Get user home directory |

## Usage Examples

### Platform Detection

```typescript
import { getInfo } from "runtime:os_compat";

const os = await getInfo();
console.log(`Running on ${os.name} ${os.version} (${os.arch})`);

if (os.name === "macos") {
  // macOS-specific code
} else if (os.name === "windows") {
  // Windows-specific code
}
```

### Cross-Platform Paths

```typescript
import { pathSep, envSep } from "runtime:os_compat";

const sep = await pathSep();
const path = ["home", "user", "documents"].join(sep);

const envDelim = await envSep();
const paths = process.env.PATH?.split(envDelim) || [];
```

### Line Endings

```typescript
import { lineEnding } from "runtime:os_compat";

const eol = await lineEnding();
const lines = ["line 1", "line 2", "line 3"];
const content = lines.join(eol);
```

## Platform Values

| Platform | `pathSep()` | `envSep()` | `lineEnding()` |
|----------|-------------|------------|----------------|
| macOS | `/` | `:` | `\n` |
| Linux | `/` | `:` | `\n` |
| Windows | `\` | `;` | `\r\n` |

## File Structure

```text
crates/ext_os_compat/
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
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub family: String,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct PathInfo {
    pub separator: char,
    pub delimiter: char,
}

#[weld_op]
#[op2]
#[serde]
pub fn op_os_compat_info() -> Result<OsInfo, OsCompatError> {
    // implementation
}

#[weld_op]
#[op2]
#[string]
pub fn op_os_compat_path_sep() -> String {
    std::path::MAIN_SEPARATOR.to_string()
}

#[weld_op]
#[op2]
#[string]
pub fn op_os_compat_home_dir() -> Result<String, OsCompatError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_os_compat", "runtime:os_compat")
        .ts_path("ts/init.ts")
        .ops(&["op_os_compat_info", "op_os_compat_path_sep", "op_os_compat_env_sep", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_os_compat extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `dirs` | Home directory |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_sys](/docs/crates/ext-sys) - System information
- [ext_path](/docs/crates/ext-path) - Path utilities
- [Architecture](/docs/architecture) - Full system architecture
