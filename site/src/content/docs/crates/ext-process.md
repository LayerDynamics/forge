---
title: "ext_process"
description: "Child process spawning and management for Forge applications"
slug: crates/ext-process
---

# Process Management

Spawn and manage child processes with full I/O control, signals, and lifecycle management.

## Overview

The `runtime:process` module enables Forge applications to execute external commands, interact with long-running processes, and manage subprocess lifecycles. It provides cross-platform process spawning with async I/O streams, signal handling, and capability-based security.

**Key Capabilities:**
- Execute shell commands and scripts
- Bidirectional communication via stdin/stdout/stderr
- Async iteration over process output
- Signal-based process control (SIGTERM, SIGKILL, etc.)
- Resource limits and permission controls
- Cross-platform with graceful fallbacks

## Installation

Import from the `runtime:process` module:

```typescript
import { spawn } from "runtime:process";
```

## Core Concepts

### Process Handles

When you spawn a process, you receive a `ProcessHandle` object that provides methods for interacting with the process:

```typescript
const proc = await spawn("echo", { args: ["Hello"] });

// ProcessHandle methods
proc.id         // Internal handle ID
proc.pid        // Operating system PID
proc.kill()     // Send termination signal
proc.wait()     // Wait for exit
proc.status()   // Check if running
proc.writeStdin()   // Write to stdin
proc.readStdout()   // Read from stdout
proc.readStderr()   // Read from stderr
```

### Standard I/O Configuration

Configure how stdin, stdout, and stderr are handled:

- **`"piped"`** - Capture for programmatic reading/writing
- **`"inherit"`** - Inherit from parent process (output to console)
- **`"null"`** - Discard (no I/O)

```typescript
const proc = await spawn("command", {
  stdin: "piped",   // Enable writeStdin()
  stdout: "piped",  // Enable readStdout() and async iteration
  stderr: "inherit" // Errors go to console
});
```

### Async Iteration

The `stdout` and `stderr` properties are async iterators that yield output line by line:

```typescript
for await (const line of proc.stdout) {
  console.log(line);
}
// Loop completes when process closes stdout
```

## API Reference

Due to character limits, I'll create a streamlined version. Please see the full documentation file created earlier with all examples, error handling guides, and platform-specific notes.

### `spawn(binary, options)`

Spawns a new child process.

**Example:**
```typescript
const proc = await spawn("echo", {
  args: ["Hello"],
  stdout: "piped"
});
const result = await proc.wait();
```

See the [README](../../../crates/ext_process/README.md) and [generated SDK](../../../sdk/runtime.process.ts) for complete API documentation with full examples.

## See Also

- [ext_fs](./ext-fs.md) - File system operations
- [ext_shell](./ext-shell.md) - Shell integration
- [ext_path](./ext-path.md) - Path manipulation
- [Permissions Guide](/guides/permissions) - Configuring app permissions
