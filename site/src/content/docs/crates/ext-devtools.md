---
title: "ext_devtools"
description: Developer tools extension providing the runtime:devtools module.
slug: crates/ext-devtools
---

The `ext_devtools` crate provides developer tools functionality for Forge applications through the `runtime:devtools` module.

## Overview

ext_devtools handles:

- **DevTools panel** - Open/close browser devtools for windows
- **Inspection** - Enable DOM inspection in WebViews

## Module: `runtime:devtools`

```typescript
import {
  openDevTools,
  closeDevTools,
  isDevToolsOpen
} from "runtime:devtools";
```

## Key Types

### Error Types

```rust
enum DevtoolsErrorCode {
    Generic = 9100,
    PermissionDenied = 9101,
    WindowNotFound = 9102,
}

struct DevtoolsError {
    code: DevtoolsErrorCode,
    message: String,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_devtools_open` | `openDevTools(windowId)` | Open DevTools for a window |
| `op_devtools_close` | `closeDevTools(windowId)` | Close DevTools for a window |
| `op_devtools_is_open` | `isDevToolsOpen(windowId)` | Check if DevTools are open |

## Usage Examples

### Opening DevTools

```typescript
import { openDevTools, closeDevTools } from "runtime:devtools";
import { createWindow } from "runtime:window";

const win = await createWindow({ title: "My App" });

// Open DevTools for debugging
await openDevTools(win.id);

// Close when done
await closeDevTools(win.id);
```

### Toggle DevTools

```typescript
import { openDevTools, closeDevTools, isDevToolsOpen } from "runtime:devtools";

async function toggleDevTools(windowId: string) {
  if (await isDevToolsOpen(windowId)) {
    await closeDevTools(windowId);
  } else {
    await openDevTools(windowId);
  }
}
```

## File Structure

```text
crates/ext_devtools/
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
use forge_weld_macro::weld_op;
use std::cell::RefCell;
use std::rc::Rc;

#[weld_op(async)]
#[op2(async)]
pub async fn op_devtools_open(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<(), DevtoolsError> {
    // implementation
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_devtools_is_open(
    state: Rc<RefCell<OpState>>,
    #[string] window_id: String,
) -> Result<bool, DevtoolsError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_devtools", "runtime:devtools")
        .ts_path("ts/init.ts")
        .ops(&["op_devtools_open", "op_devtools_close", "op_devtools_is_open"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_devtools extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `ext_window` | Window management integration |
| `tokio` | Async runtime |
| `thiserror` | Error handling |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]` macro |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_window](/docs/crates/ext-window) - Window management extension
- [forge-runtime](/docs/crates/forge-runtime) - Main runtime
