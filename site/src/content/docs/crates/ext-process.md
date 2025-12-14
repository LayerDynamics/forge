---
title: "ext_process"
description: Process management extension providing the host:process module.
---

The `ext_process` crate provides child process spawning and management for Forge applications through the `host:process` module.

## Overview

ext_process handles:

- **Process spawning** - Launch child processes
- **Stdio configuration** - Pipe, inherit, or null
- **Process I/O** - Read stdout/stderr, write stdin
- **Process lifecycle** - Wait, kill, status
- **Capability-based security** - Command allowlisting

## Module: `host:process`

```typescript
import { spawn } from "host:process";

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

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `tokio` | Async process spawning |
| `nix` | Unix signals (Unix only) |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |

## Related

- [host:process API](/docs/api/host-process) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Build system
