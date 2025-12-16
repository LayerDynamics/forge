---
title: "ext_fs"
description: Filesystem operations extension providing the runtime:fs module.
slug: crates/ext-fs
---

The `ext_fs` crate provides filesystem operations for Forge applications through the `runtime:fs` module.

## Overview

ext_fs handles:

- **File reading/writing** - Text and binary file operations
- **Directory operations** - Create, read, and remove directories
- **File watching** - Monitor files and directories for changes
- **File metadata** - Size, modification time, file type
- **Capability-based security** - Path-based permission checks

## Module: `runtime:fs`

```typescript
import {
  readTextFile,
  writeTextFile,
  readDir,
  watch
} from "runtime:fs";
```

## Key Types

### Error Types

```rust
enum FsErrorCode {
    Io = 3000,
    PermissionDenied = 3001,
    NotFound = 3002,
    AlreadyExists = 3003,
    InvalidPath = 3004,
    NotAFile = 3005,
    NotADirectory = 3006,
}

struct FsError {
    code: FsErrorCode,
    message: String,
}
```

### Data Types

```rust
struct FileStat {
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    size: u64,
    modified: Option<u64>,
    accessed: Option<u64>,
    created: Option<u64>,
}

struct DirEntry {
    name: String,
    path: String,
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
}

struct FileEvent {
    kind: WatchEventKind,
    paths: Vec<String>,
}

enum WatchEventKind {
    Create,
    Modify,
    Remove,
    Other,
}
```

### Capability Types

```rust
struct FsCapabilities {
    read_patterns: Vec<String>,
    write_patterns: Vec<String>,
}

trait FsCapabilityChecker {
    fn check_read(&self, path: &str) -> bool;
    fn check_write(&self, path: &str) -> bool;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_fs_read_text` | `readTextFile(path)` | Read file as UTF-8 string |
| `op_fs_read_bytes` | `readFile(path)` | Read file as bytes |
| `op_fs_write_text` | `writeTextFile(path, content)` | Write string to file |
| `op_fs_write_bytes` | `writeFile(path, data)` | Write bytes to file |
| `op_fs_read_dir` | `readDir(path)` | List directory entries |
| `op_fs_mkdir` | `mkdir(path, opts?)` | Create directory |
| `op_fs_remove` | `remove(path, opts?)` | Remove file or directory |
| `op_fs_stat` | `stat(path)` | Get file metadata |
| `op_fs_exists` | `exists(path)` | Check if path exists |
| `op_fs_watch` | `watch(path)` | Watch for file changes |
| `op_fs_watch_recv` | (internal) | Receive watch events |

## File Structure

```text
crates/ext_fs/
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

// Annotate structs for TypeScript interface generation
#[weld_struct]
#[derive(Debug, Serialize)]
pub struct FileStat {
    pub is_file: bool,
    pub is_directory: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

// Annotate ops for TypeScript function generation
// Note: #[weld_op] must come BEFORE #[op2]
#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_fs_read_text(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, FsError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_fs", "runtime:fs")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_fs_read_text",
            "op_fs_write_text",
            "op_fs_read_dir",
            // ...
        ])
        .generate_sdk_module("sdk")   // Generates sdk/runtime.fs.ts
        .use_inventory_types()         // Reads #[weld_*] annotations
        .build()
        .expect("Failed to build runtime_fs extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async filesystem operations |
| `notify` | File watching |
| `globset` | Pattern matching for capabilities |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [runtime:fs API](/docs/api/runtime-fs) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Build system
