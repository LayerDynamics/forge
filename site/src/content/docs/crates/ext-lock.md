---
title: "ext_lock"
description: Named async lock extension providing the runtime:lock module.
slug: crates/ext-lock
---

The `ext_lock` crate provides named asynchronous locks for Forge applications through the `runtime:lock` module.

## Overview

ext_lock handles:

- **Named locks** - Acquire/release locks by name
- **Async acquisition** - Non-blocking lock operations
- **Try acquire** - Immediate lock attempt without waiting
- **Lock timeouts** - Optional timeout for acquisition

## Module: `runtime:lock`

```typescript
import {
  acquire,
  release,
  tryAcquire,
  isLocked
} from "runtime:lock";
```

## Key Types

### Error Types

```rust
enum LockErrorCode {
    Generic = 8400,
    AcquireFailed = 8401,
    ReleaseFailed = 8402,
    Timeout = 8403,
    NotHeld = 8404,
    InvalidName = 8405,
}

struct LockError {
    code: LockErrorCode,
    message: String,
}
```

### Lock Types

```rust
struct LockHandle {
    id: u32,
    name: String,
}

struct LockState {
    locks: HashMap<String, LockInfo>,
    next_id: u32,
}

struct LockInfo {
    id: u32,
    holder: Option<u32>,
    waiters: VecDeque<Waker>,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_lock_acquire` | `acquire(name, timeout?)` | Acquire a named lock |
| `op_lock_release` | `release(handle)` | Release a held lock |
| `op_lock_try_acquire` | `tryAcquire(name)` | Try to acquire without waiting |
| `op_lock_is_locked` | `isLocked(name)` | Check if lock is held |

## Usage Examples

### Basic Locking

```typescript
import { acquire, release } from "runtime:lock";

const lock = await acquire("my-resource");
try {
  // Critical section
  await doSomething();
} finally {
  await release(lock);
}
```

### Try Acquire

```typescript
import { tryAcquire, release } from "runtime:lock";

const lock = await tryAcquire("my-resource");
if (lock) {
  try {
    await doSomething();
  } finally {
    await release(lock);
  }
} else {
  console.log("Resource is busy");
}
```

### With Timeout

```typescript
import { acquire, release } from "runtime:lock";

try {
  const lock = await acquire("my-resource", { timeout: 5000 });
  // ... use resource ...
  await release(lock);
} catch (e) {
  if (e.code === 8403) {
    console.log("Lock acquisition timed out");
  }
}
```

## File Structure

```text
crates/ext_lock/
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
pub struct LockHandle {
    pub id: u32,
    pub name: String,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_lock_acquire(
    state: Rc<RefCell<OpState>>,
    #[string] name: String,
    #[serde] opts: Option<LockOptions>,
) -> Result<LockHandle, LockError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_lock", "runtime:lock")
        .ts_path("ts/init.ts")
        .ops(&["op_lock_acquire", "op_lock_release", "op_lock_try_acquire", "op_lock_is_locked"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_lock extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async primitives |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [forge-runtime](/docs/crates/forge-runtime) - Main runtime
- [Architecture](/docs/architecture) - Full system architecture
