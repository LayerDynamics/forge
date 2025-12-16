---
title: "ext_app"
description: Application lifecycle extension providing the runtime:app module.
slug: crates/ext-app
---

The `ext_app` crate provides application lifecycle management for Forge applications through the `runtime:app` module.

## Overview

ext_app handles:

- **Application lifecycle** - Quit, exit, relaunch
- **App metadata** - Version, name, identifier
- **Special paths** - App data, cache, config directories
- **Single instance** - Ensure only one instance runs
- **Window visibility** - Show/hide all windows
- **Badge count** - macOS dock badge
- **Locale** - System locale information

## Module: `runtime:app`

```typescript
import {
  quit,
  exit,
  relaunch,
  getVersion,
  getName,
  getPath,
  requestSingleInstanceLock
} from "runtime:app";
```

## Key Types

### Error Types

```rust
enum AppErrorCode {
    QuitFailed = 8300,
    ExitFailed = 8301,
    RelaunchFailed = 8302,
    InfoFailed = 8303,
    PathFailed = 8304,
    LockFailed = 8305,
    FocusFailed = 8306,
    HideFailed = 8307,
    ShowFailed = 8308,
    BadgeFailed = 8309,
    UserModelIdFailed = 8310,
    InvalidPathType = 8311,
    PermissionDenied = 8312,
    NotSupported = 8313,
    NotInitialized = 8314,
}

struct AppError {
    code: AppErrorCode,
    message: String,
}
```

### Path Types

```rust
enum PathType {
    AppData,     // ~/.myapp or AppData/Roaming/myapp
    Cache,       // ~/.cache/myapp or AppData/Local/myapp
    Config,      // ~/.config/myapp
    Temp,        // System temp directory
    Home,        // User home directory
    Desktop,     // User desktop
    Documents,   // User documents
    Downloads,   // User downloads
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_app_quit` | `quit()` | Gracefully quit the application |
| `op_app_exit` | `exit(code?)` | Exit with status code |
| `op_app_relaunch` | `relaunch(opts?)` | Relaunch the application |
| `op_app_get_version` | `getVersion()` | Get app version |
| `op_app_get_name` | `getName()` | Get app name |
| `op_app_get_identifier` | `getIdentifier()` | Get app identifier |
| `op_app_get_path` | `getPath(type)` | Get special directory path |
| `op_app_request_single_instance_lock` | `requestSingleInstanceLock()` | Request single instance |
| `op_app_has_single_instance_lock` | `hasSingleInstanceLock()` | Check if lock held |
| `op_app_release_single_instance_lock` | `releaseSingleInstanceLock()` | Release lock |
| `op_app_focus` | `focus()` | Focus application |
| `op_app_hide` | `hide()` | Hide all windows |
| `op_app_show` | `show()` | Show all windows |
| `op_app_set_badge_count` | `setBadgeCount(count)` | Set dock badge (macOS) |
| `op_app_get_locale` | `getLocale()` | Get system locale |

## Usage Examples

### Application Lifecycle

```typescript
import { quit, exit, relaunch } from "runtime:app";

// Graceful quit (allows cleanup)
await quit();

// Exit immediately with code
await exit(1);

// Relaunch with new arguments
await relaunch({ args: ["--reset"] });
```

### Special Paths

```typescript
import { getPath } from "runtime:app";

const appData = await getPath("appData");   // ~/.myapp
const cache = await getPath("cache");       // ~/.cache/myapp
const config = await getPath("config");     // ~/.config/myapp
```

### Single Instance

```typescript
import { requestSingleInstanceLock, hasSingleInstanceLock } from "runtime:app";

const gotLock = await requestSingleInstanceLock();
if (!gotLock) {
  console.log("Another instance is already running");
  await exit(0);
}
```

## File Structure

```text
crates/ext_app/
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
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathType {
    AppData,
    Cache,
    Config,
    Temp,
    Home,
    Desktop,
    Documents,
    Downloads,
}

#[weld_op(async)]
#[op2(async)]
#[string]
pub async fn op_app_get_path(
    state: Rc<RefCell<OpState>>,
    #[serde] path_type: PathType,
) -> Result<String, AppError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_app", "runtime:app")
        .ts_path("ts/init.ts")
        .ops(&["op_app_quit", "op_app_get_path", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_app extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `dirs` | Standard directories |
| `single-instance` | Single instance lock |
| `sys-locale` | System locale |
| `tokio` | Async runtime |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [forge-runtime](/docs/crates/forge-runtime) - Runtime that manages app lifecycle
- [forge-weld](/docs/crates/forge-weld) - Code generation library
- [Architecture](/docs/architecture) - Full system architecture
