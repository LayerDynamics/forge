---
title: "ext_shell"
description: Shell operations extension providing the runtime:shell module.
slug: crates/ext-shell
---

The `ext_shell` crate provides shell and desktop integration for Forge applications through the `runtime:shell` module.

## Overview

ext_shell handles:

- **Open external** - Open URLs in default browser
- **Open path** - Open files/folders in default app
- **Show in folder** - Reveal file in file manager
- **Trash** - Move items to trash/recycle bin
- **System beep** - Play system alert sound

## Module: `runtime:shell`

```typescript
import {
  openExternal,
  openPath,
  showInFolder,
  moveToTrash,
  beep
} from "runtime:shell";
```

## Key Types

### Error Types

```rust
enum ShellErrorCode {
    Generic = 8800,
    OpenFailed = 8801,
    TrashFailed = 8802,
    NotFound = 8803,
    PermissionDenied = 8804,
}

struct ShellError {
    code: ShellErrorCode,
    message: String,
}
```

### Options Types

```rust
struct OpenExternalOptions {
    activate: Option<bool>,   // Bring app to front
    work_dir: Option<String>, // Working directory
}

struct TrashOptions {
    permanent: Option<bool>,  // Skip trash, delete permanently
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_shell_open_external` | `openExternal(url, opts?)` | Open URL in default browser |
| `op_shell_open_path` | `openPath(path)` | Open file/folder in default app |
| `op_shell_show_in_folder` | `showInFolder(path)` | Reveal in file manager |
| `op_shell_move_to_trash` | `moveToTrash(path, opts?)` | Move to trash |
| `op_shell_beep` | `beep()` | Play system beep |

## Usage Examples

### Opening URLs

```typescript
import { openExternal } from "runtime:shell";

// Open in default browser
await openExternal("https://example.com");

// Open mailto link
await openExternal("mailto:support@example.com?subject=Help");

// With options
await openExternal("https://docs.example.com", { activate: true });
```

### Opening Files

```typescript
import { openPath, showInFolder } from "runtime:shell";

// Open file in default application
await openPath("/home/user/document.pdf");

// Open folder
await openPath("/home/user/downloads");

// Reveal file in file manager
await showInFolder("/home/user/documents/report.pdf");
```

### Trash Operations

```typescript
import { moveToTrash } from "runtime:shell";

// Move to trash
await moveToTrash("/home/user/old-file.txt");

// Delete permanently (skip trash)
await moveToTrash("/home/user/temp.txt", { permanent: true });
```

### System Beep

```typescript
import { beep } from "runtime:shell";

// Alert user
await beep();
```

## Platform Behavior

| Operation | macOS | Windows | Linux |
|-----------|-------|---------|-------|
| `openExternal` | `open` command | `start` command | `xdg-open` |
| `openPath` | `open` command | `explorer` | `xdg-open` |
| `showInFolder` | Finder `reveal` | Explorer `/select` | File manager |
| `moveToTrash` | `.Trash` folder | Recycle Bin | `trash-cli` or delete |
| `beep` | System sound | Message beep | Console bell |

## File Structure

```text
crates/ext_shell/
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
#[derive(Debug, Deserialize)]
pub struct OpenExternalOptions {
    pub activate: Option<bool>,
    pub work_dir: Option<String>,
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_shell_open_external(
    state: Rc<RefCell<OpState>>,
    #[string] url: String,
    #[serde] opts: Option<OpenExternalOptions>,
) -> Result<(), ShellError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_shell", "runtime:shell")
        .ts_path("ts/init.ts")
        .ops(&["op_shell_open_external", "op_shell_open_path", "op_shell_move_to_trash", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_shell extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `open` | Cross-platform open |
| `trash` | Trash operations |
| `tokio` | Async runtime |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_fs](/docs/crates/ext-fs) - File system operations
- [ext_process](/docs/crates/ext-process) - Process spawning
- [Architecture](/docs/architecture) - Full system architecture
