---
title: "ext_storage"
description: SQLite key-value storage extension providing the runtime:storage module.
slug: crates/ext-storage
---

The `ext_storage` crate provides persistent key-value storage backed by SQLite for Forge applications through the `runtime:storage` module.

## Overview

ext_storage handles:

- **Key-value storage** - Get, set, remove values
- **SQLite backend** - Persistent, ACID-compliant storage
- **Multiple stores** - Isolated namespaced storage areas
- **Batch operations** - Efficient bulk operations
- **Type preservation** - JSON serialization for complex values

## Module: `runtime:storage`

```typescript
import {
  get,
  set,
  remove,
  clear,
  keys,
  has,
  entries
} from "runtime:storage";
```

## Key Types

### Error Types

```rust
enum StorageErrorCode {
    Generic = 9000,
    NotFound = 9001,
    WriteFailed = 9002,
    ReadFailed = 9003,
    InvalidKey = 9004,
    DatabaseError = 9005,
    SerializationError = 9006,
}

struct StorageError {
    code: StorageErrorCode,
    message: String,
}
```

### Storage Types

```rust
struct StorageOptions {
    store: Option<String>,  // Named store, default "default"
}

struct StorageEntry {
    key: String,
    value: Value,
}

struct StorageState {
    db: Connection,
    path: PathBuf,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_storage_get` | `get(key, opts?)` | Get value by key |
| `op_storage_set` | `set(key, value, opts?)` | Set key-value pair |
| `op_storage_remove` | `remove(key, opts?)` | Remove by key |
| `op_storage_clear` | `clear(opts?)` | Clear all entries |
| `op_storage_keys` | `keys(opts?)` | List all keys |
| `op_storage_has` | `has(key, opts?)` | Check if key exists |
| `op_storage_entries` | `entries(opts?)` | Get all key-value pairs |

## Usage Examples

### Basic Operations

```typescript
import { get, set, remove, has } from "runtime:storage";

// Store values
await set("user.name", "Alice");
await set("user.preferences", { theme: "dark", fontSize: 14 });

// Retrieve values
const name = await get("user.name");
const prefs = await get("user.preferences");

// Check existence
if (await has("user.name")) {
  console.log("User exists");
}

// Remove value
await remove("user.name");
```

### Named Stores

```typescript
import { get, set, keys, clear } from "runtime:storage";

// Use separate store for settings
await set("theme", "dark", { store: "settings" });
await set("fontSize", 14, { store: "settings" });

// Use separate store for cache
await set("api_response", data, { store: "cache" });

// List keys in specific store
const settingKeys = await keys({ store: "settings" });

// Clear only cache store
await clear({ store: "cache" });
```

### Listing and Iteration

```typescript
import { keys, entries } from "runtime:storage";

// List all keys
const allKeys = await keys();
console.log("Stored keys:", allKeys);

// Get all entries
const allEntries = await entries();
for (const { key, value } of allEntries) {
  console.log(`${key}: ${JSON.stringify(value)}`);
}
```

### Complex Values

```typescript
import { get, set } from "runtime:storage";

// Store arrays
await set("recent_files", [
  "/home/user/doc1.txt",
  "/home/user/doc2.pdf"
]);

// Store nested objects
await set("window_state", {
  bounds: { x: 100, y: 100, width: 800, height: 600 },
  maximized: false,
  displayId: 1
});

// Retrieve with type
const files = await get<string[]>("recent_files");
const state = await get<WindowState>("window_state");
```

## File Structure

```text
crates/ext_storage/
├── src/
│   └── lib.rs        # Extension implementation
├── ts/
│   └── init.ts       # TypeScript module shim
├── build.rs          # forge-weld build configuration
└── Cargo.toml
```

## Storage Location

The storage database is located at:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/<app>/storage.db` |
| Linux | `~/.local/share/<app>/storage.db` |
| Windows | `%APPDATA%\<app>\storage.db` |

## Rust Implementation

Operations are annotated with forge-weld macros for automatic TypeScript binding generation:

```rust
// src/lib.rs
use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use serde::{Deserialize, Serialize};

#[weld_struct]
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageOptions {
    pub store: Option<String>,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_storage_get(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
    #[serde] opts: Option<StorageOptions>,
) -> Result<Option<serde_json::Value>, StorageError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_storage", "runtime:storage")
        .ts_path("ts/init.ts")
        .ops(&["op_storage_get", "op_storage_set", "op_storage_remove", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_storage extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `rusqlite` | SQLite bindings |
| `serde_json` | JSON serialization |
| `dirs` | Platform directories |
| `tokio` | Async runtime |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_app](/docs/crates/ext-app) - Application paths
- [ext_database](/docs/crates/ext-database) - Full database access
- [Architecture](/docs/architecture) - Full system architecture
