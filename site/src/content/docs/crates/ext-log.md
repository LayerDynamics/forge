---
title: "ext_log"
description: Structured logging extension providing the runtime:log module.
slug: crates/ext-log
---

The `ext_log` crate provides structured logging functionality for Forge applications through the `runtime:log` module.

## Overview

ext_log handles:

- **Log levels** - Trace, debug, info, warn, error
- **Structured data** - Key-value pairs with messages
- **Log targets** - Console, file, custom handlers
- **Filtering** - Level-based and target-based filtering

## Module: `runtime:log`

```typescript
import {
  trace,
  debug,
  info,
  warn,
  error,
  setLevel
} from "runtime:log";
```

## Key Types

### Error Types

```rust
enum LogErrorCode {
    Generic = 8500,
    InvalidLevel = 8501,
    WriteFailed = 8502,
    ConfigFailed = 8503,
}

struct LogError {
    code: LogErrorCode,
    message: String,
}
```

### Log Types

```rust
enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

struct LogRecord {
    level: LogLevel,
    message: String,
    target: Option<String>,
    fields: HashMap<String, Value>,
    timestamp: DateTime<Utc>,
}

struct LogConfig {
    level: LogLevel,
    targets: Vec<LogTarget>,
}

enum LogTarget {
    Console,
    File(PathBuf),
    Custom(String),
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_log_emit` | `trace/debug/info/warn/error(msg, fields?)` | Emit log record |
| `op_log_set_level` | `setLevel(level)` | Set minimum log level |
| `op_log_get_level` | `getLevel()` | Get current log level |

## Usage Examples

### Basic Logging

```typescript
import { info, warn, error } from "runtime:log";

info("Application started");
warn("Configuration file not found, using defaults");
error("Failed to connect to database", { host: "localhost", port: 5432 });
```

### Structured Logging

```typescript
import { info, debug } from "runtime:log";

info("User logged in", {
  userId: "123",
  email: "user@example.com",
  loginMethod: "oauth"
});

debug("Request processed", {
  method: "GET",
  path: "/api/users",
  duration: 45,
  status: 200
});
```

### Setting Log Level

```typescript
import { setLevel, debug } from "runtime:log";

// Only show warnings and errors
setLevel("warn");

debug("This won't be shown");  // Filtered out
```

## File Structure

```text
crates/ext_log/
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
use forge_weld_macro::{weld_op, weld_enum};
use serde::{Deserialize, Serialize};

#[weld_enum]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

#[weld_op]
#[op2]
pub fn op_log_emit(
    state: Rc<RefCell<OpState>>,
    #[serde] level: LogLevel,
    #[string] message: String,
    #[serde] fields: Option<serde_json::Value>,
) -> Result<(), LogError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_log", "runtime:log")
        .ts_path("ts/init.ts")
        .ops(&["op_log_emit", "op_log_set_level", "op_log_get_level"])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_log extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tracing` | Logging infrastructure |
| `tracing-subscriber` | Log formatting |
| `chrono` | Timestamps |
| `serde` | Serialization |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [ext_trace](/docs/crates/ext-trace) - Application tracing
- [Architecture](/docs/architecture) - Full system architecture
