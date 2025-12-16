---
title: "ext_timers"
description: Timer functionality extension providing the runtime:timers module.
slug: crates/ext-timers
---

The `ext_timers` crate provides timer and delay functionality for Forge applications through the `runtime:timers` module.

## Overview

ext_timers handles:

- **One-shot timers** - Fire once after delay
- **Interval timers** - Fire repeatedly
- **Sleep/delay** - Pause execution
- **Timer cancellation** - Cancel pending timers
- **High-resolution timing** - Millisecond precision

## Module: `runtime:timers`

```typescript
import {
  setTimeout,
  setInterval,
  clearTimeout,
  clearInterval,
  sleep
} from "runtime:timers";
```

## Key Types

### Error Types

```rust
enum TimersErrorCode {
    Generic = 9200,
    InvalidDelay = 9201,
    TimerNotFound = 9202,
    CancelFailed = 9203,
}

struct TimersError {
    code: TimersErrorCode,
    message: String,
}
```

### Timer Types

```rust
struct TimerHandle {
    id: u32,
}

struct TimerState {
    timers: HashMap<u32, TimerInfo>,
    next_id: u32,
}

struct TimerInfo {
    id: u32,
    delay_ms: u64,
    repeat: bool,
    callback: JsCallback,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_timers_set_timeout` | `setTimeout(callback, delay)` | Create one-shot timer |
| `op_timers_set_interval` | `setInterval(callback, delay)` | Create repeating timer |
| `op_timers_clear_timeout` | `clearTimeout(handle)` | Cancel one-shot timer |
| `op_timers_clear_interval` | `clearInterval(handle)` | Cancel repeating timer |
| `op_timers_sleep` | `sleep(ms)` | Async sleep |
| `op_timers_exists` | `exists(handle)` | Check if timer exists |

## Usage Examples

### One-Shot Timers

```typescript
import { setTimeout, clearTimeout } from "runtime:timers";

// Fire after 1 second
const timer = setTimeout(() => {
  console.log("Timer fired!");
}, 1000);

// Cancel if needed
clearTimeout(timer);
```

### Interval Timers

```typescript
import { setInterval, clearInterval } from "runtime:timers";

let count = 0;
const interval = setInterval(() => {
  count++;
  console.log(`Tick ${count}`);

  if (count >= 5) {
    clearInterval(interval);
  }
}, 1000);
```

### Async Sleep

```typescript
import { sleep } from "runtime:timers";

async function delayedOperation() {
  console.log("Starting...");
  await sleep(2000);  // Wait 2 seconds
  console.log("Done!");
}
```

### Debounce Pattern

```typescript
import { setTimeout, clearTimeout } from "runtime:timers";

let debounceTimer: number | null = null;

function debounce(fn: () => void, delay: number) {
  return () => {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(fn, delay);
  };
}

const debouncedSearch = debounce(() => {
  performSearch();
}, 300);
```

### Timeout Pattern

```typescript
import { sleep } from "runtime:timers";

async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number
): Promise<T> {
  const timeout = sleep(timeoutMs).then(() => {
    throw new Error("Operation timed out");
  });

  return Promise.race([promise, timeout]);
}

// Usage
const result = await withTimeout(fetchData(), 5000);
```

## File Structure

```text
crates/ext_timers/
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
pub struct TimerHandle {
    pub id: u32,
}

#[weld_op(async)]
#[op2(async)]
pub async fn op_timers_sleep(
    #[bigint] delay_ms: u64,
) -> Result<(), TimersError> {
    // implementation
}

#[weld_op]
#[op2]
#[serde]
pub fn op_timers_set_timeout(
    state: Rc<RefCell<OpState>>,
    #[bigint] delay_ms: u64,
) -> Result<TimerHandle, TimersError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_timers", "runtime:timers")
        .ts_path("ts/init.ts")
        .ops(&["op_timers_set_timeout", "op_timers_set_interval", "op_timers_sleep", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_timers extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async timers |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]` macros |
| `linkme` | Compile-time symbol collection |

## Notes

- Timer precision depends on system capabilities
- Very short delays (< 1ms) may not be accurate
- Timers are cancelled when the runtime exits

## Related

- [Architecture](/docs/architecture) - Full system architecture
