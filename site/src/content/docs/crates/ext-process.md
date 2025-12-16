---
title: "ext_process"
description: Process management extension providing the runtime:process module.
slug: crates/ext-process
---

The `ext_process` crate provides child process spawning and management for Forge applications through the `runtime:process` module.

## Overview

ext_process handles:

- **Process spawning** - Launch child processes
- **Stdio configuration** - Pipe, inherit, or null
- **Process I/O** - Read stdout/stderr, write stdin
- **Process lifecycle** - Wait, kill, status
- **Capability-based security** - Command allowlisting

## Module: `runtime:process`

```typescript
import { spawn } from "runtime:process";

const proc = await spawn("ls", { args: ["-la"] });
for await (const line of proc.stdout) {
  console.log(line);
}
await proc.wait();
```

## Key Types

### Error Types

```rust
enum ProcessErrorCode {
    Io = 4000,
    PermissionDenied = 4001,
    NotFound = 4002,
    SpawnFailed = 4003,
    ProcessNotFound = 4004,
    AlreadyExited = 4005,
}

struct ProcessError {
    code: ProcessErrorCode,
    message: String,
}
```

### Spawn Types

```rust
struct SpawnOpts {
    args: Option<Vec<String>>,
    cwd: Option<String>,
    env: Option<HashMap<String, String>>,
    stdin: Option<StdioConfig>,
    stdout: Option<StdioConfig>,
    stderr: Option<StdioConfig>,
}

enum StdioConfig {
    Inherit,
    Piped,
    Null,
}

struct SpawnResult {
    id: u32,
    stdin: Option<ProcessStdin>,
    stdout: Option<ProcessStdout>,
    stderr: Option<ProcessStderr>,
}
```

### Process State Types

```rust
struct ProcessStatus {
    id: u32,
    running: bool,
    exit_code: Option<i32>,
}

struct ProcessOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct ProcessHandle {
    id: u32,
}

struct ProcessState {
    processes: HashMap<u32, ChildProcess>,
    next_id: u32,
}
```

### Capability Types

```rust
struct ProcessCapabilities {
    allowed_commands: Vec<String>,
    denied_commands: Vec<String>,
    allow_env_access: bool,
}

trait ProcessCapabilityChecker {
    fn check_spawn(&self, command: &str) -> bool;
    fn check_env_access(&self) -> bool;
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_process_spawn` | `spawn(cmd, opts?)` | Spawn a child process |
| `op_process_write_stdin` | `proc.stdin.write(data)` | Write to process stdin |
| `op_process_read_stdout` | `proc.stdout[Symbol.asyncIterator]` | Read from stdout |
| `op_process_read_stderr` | `proc.stderr[Symbol.asyncIterator]` | Read from stderr |
| `op_process_wait` | `proc.wait()` | Wait for process to exit |
| `op_process_kill` | `proc.kill(signal?)` | Kill process |
| `op_process_status` | `proc.status()` | Get process status |

## File Structure

```text
crates/ext_process/
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
pub enum StdioConfig {
    Inherit,
    Piped,
    Null,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct SpawnResult {
    pub id: u32,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_process_spawn(
    state: Rc<RefCell<OpState>>,
    #[string] command: String,
    #[serde] opts: Option<SpawnOpts>,
) -> Result<SpawnResult, ProcessError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_process", "runtime:process")
        .ts_path("ts/init.ts")
        .ops(&["op_process_spawn", "op_process_wait", "op_process_kill", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_process extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async process spawning |
| `nix` | Unix signals (Unix only) |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [runtime:process API](/docs/api/runtime-process) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Code generation library
