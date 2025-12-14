---
title: "ext_fs"
description: Filesystem operations extension providing the host:fs module.
---

The `ext_fs` crate provides filesystem operations for Forge applications through the `host:fs` module.

## Overview

ext_fs handles:

- **File reading/writing** - Text and binary file operations
- **Directory operations** - Create, read, and remove directories
- **File watching** - Monitor files and directories for changes
- **File metadata** - Size, modification time, file type
- **Capability-based security** - Path-based permission checks

## Module: `host:fs`

```typescript
import {
  readTextFile,
  writeTextFile,
  readDir,
  watch
} from "host:fs";
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

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_fs", "host:fs")
        .ts_path("ts/init.ts")
        .ops(&[
            "op_fs_read_text",
            "op_fs_write_text",
            "op_fs_read_dir",
            // ...
        ])
        .generate_sdk_types("sdk")
        .dts_generator(generate_host_fs_types)
        .build()
        .expect("Failed to build host_fs extension");
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

## Related

- [host:fs API](/docs/api/host-fs) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Build system
