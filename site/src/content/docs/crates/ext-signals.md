---
title: "ext_signals"
description: OS signal handling extension providing the runtime:signals module.
slug: crates/ext-signals
---

The `ext_signals` crate provides OS signal subscription and handling for Forge applications through the `runtime:signals` module.

## Overview

ext_signals handles:

- **Signal subscription** - Listen for OS signals
- **Graceful shutdown** - Handle SIGTERM/SIGINT
- **Custom handlers** - Run code when signals received
- **Signal masking** - Temporarily block signals

## Module: `runtime:signals`

```typescript
import {
  subscribe,
  unsubscribe,
  once
} from "runtime:signals";
```

## Key Types

### Error Types

```rust
enum SignalsErrorCode {
    Generic = 8900,
    SubscribeFailed = 8901,
    UnsubscribeFailed = 8902,
    InvalidSignal = 8903,
    NotSupported = 8904,
}

struct SignalsError {
    code: SignalsErrorCode,
    message: String,
}
```

### Signal Types

```rust
enum Signal {
    SIGINT,      // Ctrl+C
    SIGTERM,     // Termination request
    SIGHUP,      // Hangup (Unix)
    SIGQUIT,     // Quit (Unix)
    SIGUSR1,     // User-defined 1 (Unix)
    SIGUSR2,     // User-defined 2 (Unix)
    SIGBREAK,    // Ctrl+Break (Windows)
}

struct SignalSubscription {
    id: u32,
    signal: Signal,
}

struct SignalState {
    subscriptions: HashMap<u32, SignalHandler>,
    next_id: u32,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_signals_subscribe` | `subscribe(signal, callback)` | Subscribe to signal |
| `op_signals_unsubscribe` | `unsubscribe(subscription)` | Remove subscription |
| `op_signals_once` | `once(signal)` | Wait for signal once |

## Usage Examples

### Graceful Shutdown

```typescript
import { subscribe } from "runtime:signals";

const sub = await subscribe("SIGINT", async () => {
  console.log("Shutting down gracefully...");
  await cleanup();
  process.exit(0);
});
```

### One-Time Signal Wait

```typescript
import { once } from "runtime:signals";

console.log("Waiting for SIGTERM...");
await once("SIGTERM");
console.log("Received SIGTERM, exiting");
```

### Multiple Signal Handlers

```typescript
import { subscribe, unsubscribe } from "runtime:signals";

// Handle both SIGINT and SIGTERM
const intSub = await subscribe("SIGINT", handleShutdown);
const termSub = await subscribe("SIGTERM", handleShutdown);

async function handleShutdown() {
  // Cleanup and exit
  await unsubscribe(intSub);
  await unsubscribe(termSub);
  process.exit(0);
}
```

### Reload Configuration

```typescript
import { subscribe } from "runtime:signals";

// Reload config on SIGHUP (Unix)
await subscribe("SIGHUP", async () => {
  console.log("Reloading configuration...");
  await reloadConfig();
});
```

## Signal Availability

| Signal | macOS | Linux | Windows |
|--------|-------|-------|---------|
| SIGINT | ✓ | ✓ | ✓ |
| SIGTERM | ✓ | ✓ | ✓ |
| SIGHUP | ✓ | ✓ | ✗ |
| SIGQUIT | ✓ | ✓ | ✗ |
| SIGUSR1 | ✓ | ✓ | ✗ |
| SIGUSR2 | ✓ | ✓ | ✗ |
| SIGBREAK | ✗ | ✗ | ✓ |

## File Structure

```text
crates/ext_signals/
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
pub enum Signal {
    SIGINT,
    SIGTERM,
    SIGHUP,
    SIGQUIT,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct SignalSubscription {
    pub id: u32,
    pub signal: Signal,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_signals_subscribe(
    state: Rc<RefCell<OpState>>,
    #[serde] signal: Signal,
) -> Result<SignalSubscription, SignalsError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_signals", "runtime:signals")
        .ts_path("ts/init.ts")
        .ops(&["op_signals_subscribe", "op_signals_unsubscribe", "op_signals_once"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_signals extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Signal handling |
| `signal-hook` | Unix signal registration |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_app](/docs/crates/ext-app) - Application lifecycle
- [ext_process](/docs/crates/ext-process) - Process management
- [Architecture](/docs/architecture) - Full system architecture
