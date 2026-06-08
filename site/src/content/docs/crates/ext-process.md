---
title: "ext_process"
description: "Child process spawning and management for Forge applications"
slug: docs/crates/ext-process
---

# ext_process

Child process spawning and management extension for Forge applications. Provides APIs for executing external commands, interactive process communication, and lifecycle management.

> **Module**: `runtime:process` - TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld).

## Overview

The `ext_process` crate enables Forge applications to:

- **Spawn Processes**: Execute external commands and scripts
- **Bidirectional I/O**: Read stdout/stderr, write to stdin
- **Async Iteration**: Stream output line-by-line with async generators
- **Lifecycle Control**: Kill, wait, and monitor process status
- **Signal Handling**: Send termination signals (SIGTERM, SIGKILL, etc.)
- **Security**: Capability-based permission controls

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     ext_process Extension                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────────┐    ┌──────────────┐  │
│  │   spawn()    │───▶│  ProcessHandle   │───▶│   Child      │  │
│  │   op_spawn   │    │   (Rust struct)  │    │  (tokio)     │  │
│  └──────────────┘    └──────────────────┘    └──────────────┘  │
│                              │                       │          │
│                              ▼                       ▼          │
│                      ┌──────────────┐       ┌──────────────┐   │
│                      │  ProcessMap  │       │ I/O Streams  │   │
│                      │ (Arc<Mutex>) │       │ stdin/out/err│   │
│                      └──────────────┘       └──────────────┘   │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  Operations: spawn, kill, wait, status, writeStdin,             │
│              readStdout, readStderr                              │
└─────────────────────────────────────────────────────────────────┘
```

## Permissions

Process spawning requires explicit permissions in `manifest.app.toml`:

```toml
[permissions.process]
# Specific binaries allowed
spawn = ["echo", "ls", "node", "/usr/bin/python3"]

# Allow any binary (use with caution!)
# spawn = ["*"]
```

In development mode (`forge dev`), all permissions are granted automatically.

## Import

```typescript
import {
  spawn,
  kill,
  wait,
  status,
  writeStdin,
  readStdout,
  readStderr,
  type SpawnOptions,
  type ProcessHandle,
  type WaitResult,
  type ProcessStatus,
  type ReadOutput
} from "runtime:process";
```

## API Reference

### spawn(binary, options?)

Spawns a new child process and returns a handle for interacting with it.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `binary` | `string` | Path to executable or command name in PATH |
| `options` | `SpawnOptions` | Optional configuration for the process |

**SpawnOptions:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `args` | `string[]` | `[]` | Command-line arguments |
| `cwd` | `string` | Parent's cwd | Working directory |
| `env` | `Record<string, string>` | Inherited | Environment variables |
| `stdin` | `"piped" \| "inherit" \| "null"` | `"null"` | Standard input handling |
| `stdout` | `"piped" \| "inherit" \| "null"` | `"piped"` | Standard output handling |
| `stderr` | `"piped" \| "inherit" \| "null"` | `"piped"` | Standard error handling |

**Returns:** `Promise<ProcessHandle>` - Handle to the spawned process

**Throws:**
- Error (4001) - Permission denied to spawn the binary
- Error (4002) - Binary not found
- Error (4003) - Failed to spawn process
- Error (4009) - Too many processes already spawned

**Example:**

```typescript
import { spawn } from "runtime:process";

// Simple command execution
const proc = await spawn("echo", { args: ["Hello, World!"] });
for await (const line of proc.stdout) {
  console.log(line); // "Hello, World!"
}
await proc.wait();

// With custom working directory and environment
const build = await spawn("npm", {
  args: ["run", "build"],
  cwd: "/path/to/project",
  env: {
    "NODE_ENV": "production",
    "CI": "true"
  }
});
```

---

### ProcessHandle Interface

The handle returned by `spawn()` provides methods for process interaction.

#### ProcessHandle.id

**Type:** `string` (readonly)

Internal process handle identifier used for low-level operations.

#### ProcessHandle.pid

**Type:** `number` (readonly)

Operating system process ID. Can be used with external tools:

```typescript
const proc = await spawn("server");
console.log(`Server started with PID: ${proc.pid}`);
// Can now use `kill ${proc.pid}` from terminal if needed
```

#### ProcessHandle.kill(signal?)

Terminates the process with an optional signal.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `signal` | `string` | Signal name (default: platform-specific) |

**Returns:** `Promise<boolean>` - True if signal was sent successfully

**Common Signals:**

| Signal | Description |
|--------|-------------|
| `SIGTERM` | Graceful termination request |
| `SIGKILL` | Immediate termination (cannot be caught) |
| `SIGINT` | Interrupt (like Ctrl+C) |
| `SIGHUP` | Hangup signal |

**Example:**

```typescript
const proc = await spawn("long-running-server");

// Give process chance to clean up
await proc.kill("SIGTERM");

// Wait a bit, then force kill if still running
setTimeout(async () => {
  const stat = await proc.status();
  if (stat.running) {
    await proc.kill("SIGKILL");
  }
}, 5000);
```

#### ProcessHandle.wait()

Waits for the process to complete and returns exit information.

**Returns:** `Promise<WaitResult>`

**WaitResult:**

| Property | Type | Description |
|----------|------|-------------|
| `success` | `boolean` | True if exit code was 0 |
| `code` | `number \| null` | Exit code, or null if killed by signal |
| `signal` | `string \| null` | Signal name if terminated, null otherwise |

**Example:**

```typescript
const proc = await spawn("npm", { args: ["test"] });
const result = await proc.wait();

if (result.success) {
  console.log("Tests passed!");
} else if (result.signal) {
  console.log(`Tests killed by ${result.signal}`);
} else {
  console.error(`Tests failed with code ${result.code}`);
}
```

#### ProcessHandle.status()

Checks current status without blocking.

**Returns:** `Promise<ProcessStatus>`

**ProcessStatus:**

| Property | Type | Description |
|----------|------|-------------|
| `running` | `boolean` | Whether the process is still running |
| `exitCode` | `number \| undefined` | Exit code if exited |
| `signal` | `string \| undefined` | Signal if terminated |

**Example:**

```typescript
const proc = await spawn("background-task");

// Poll status every second
const interval = setInterval(async () => {
  const stat = await proc.status();
  if (!stat.running) {
    console.log("Task completed with code:", stat.exitCode);
    clearInterval(interval);
  }
}, 1000);
```

#### ProcessHandle.writeStdin(data)

Writes data to the process's standard input.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `data` | `string` | Data to write to stdin |

**Returns:** `Promise<void>`

**Throws:**
- Error (4007) - Stdin is closed
- Error (4008) - Stdin was not configured as "piped"

**Example:**

```typescript
const proc = await spawn("cat", { stdin: "piped", stdout: "piped" });

// Write multiple lines
await proc.writeStdin("Line 1\n");
await proc.writeStdin("Line 2\n");
await proc.writeStdin("Line 3\n");

// Read output
for await (const line of proc.stdout) {
  console.log("Received:", line);
}
```

#### ProcessHandle.readStdout()

Reads available data from standard output.

**Returns:** `Promise<ReadOutput>`

**ReadOutput:**

| Property | Type | Description |
|----------|------|-------------|
| `data` | `string \| null` | Data read, or null if none available |
| `eof` | `boolean` | True if stream has ended |

**Example:**

```typescript
const proc = await spawn("ls", { stdout: "piped" });

// Manual read loop
while (true) {
  const output = await proc.readStdout();
  if (output.data) {
    console.log(output.data);
  }
  if (output.eof) break;
}
```

#### ProcessHandle.readStderr()

Reads available data from standard error.

**Returns:** `Promise<ReadOutput>`

**Example:**

```typescript
const proc = await spawn("command-that-might-fail", { stderr: "piped" });

const result = await proc.wait();
if (!result.success) {
  const errors = await proc.readStderr();
  console.error("Error output:", errors.data);
}
```

#### ProcessHandle.stdout (async iterator)

Async iterator for reading stdout line by line.

**Example:**

```typescript
const proc = await spawn("tail", { args: ["-f", "logfile.txt"], stdout: "piped" });

for await (const line of proc.stdout) {
  console.log("[LOG]", line);

  if (line.includes("SHUTDOWN")) {
    await proc.kill();
    break;
  }
}
```

#### ProcessHandle.stderr (async iterator)

Async iterator for reading stderr line by line.

**Example:**

```typescript
const proc = await spawn("build-script", { stderr: "piped" });

// Process errors in real-time
for await (const line of proc.stderr) {
  if (line.includes("WARNING")) {
    console.warn("Build warning:", line);
  } else if (line.includes("ERROR")) {
    console.error("Build error:", line);
  }
}
```

---

### Low-Level Functions

These functions operate on handle IDs directly. Prefer using `ProcessHandle` methods when possible.

#### kill(handle, signal?)

```typescript
import { spawn, kill } from "runtime:process";

const proc = await spawn("server");
await kill(proc.id, "SIGTERM");
```

#### wait(handle)

```typescript
import { spawn, wait } from "runtime:process";

const proc = await spawn("task");
const result = await wait(proc.id);
```

#### status(handle)

```typescript
import { spawn, status } from "runtime:process";

const proc = await spawn("daemon");
const stat = await status(proc.id);
```

#### writeStdin(handle, data)

```typescript
import { spawn, writeStdin } from "runtime:process";

const proc = await spawn("input-reader", { stdin: "piped" });
await writeStdin(proc.id, "data\n");
```

#### readStdout(handle) / readStderr(handle)

```typescript
import { spawn, readStdout, readStderr } from "runtime:process";

const proc = await spawn("command", { stdout: "piped", stderr: "piped" });
const out = await readStdout(proc.id);
const err = await readStderr(proc.id);
```

## Type Definitions

```typescript
/**
 * Configuration options for spawning a process.
 */
interface SpawnOptions {
  /** Command-line arguments */
  args?: string[];
  /** Working directory */
  cwd?: string;
  /** Environment variables */
  env?: Record<string, string>;
  /** How to handle stdin: "piped" | "inherit" | "null" */
  stdin?: "piped" | "inherit" | "null";
  /** How to handle stdout: "piped" | "inherit" | "null" */
  stdout?: "piped" | "inherit" | "null";
  /** How to handle stderr: "piped" | "inherit" | "null" */
  stderr?: "piped" | "inherit" | "null";
}

/**
 * Result of waiting for process completion.
 */
interface WaitResult {
  /** Whether exit code was 0 */
  success: boolean;
  /** Exit code, or null if killed by signal */
  code: number | null;
  /** Signal name if terminated by signal */
  signal: string | null;
}

/**
 * Current process status.
 */
interface ProcessStatus {
  /** Whether the process is running */
  running: boolean;
  /** Exit code if exited */
  exitCode?: number;
  /** Signal if terminated */
  signal?: string;
}

/**
 * Output from reading stdout/stderr.
 */
interface ReadOutput {
  /** Data read, or null if none available */
  data: string | null;
  /** Whether the stream has ended */
  eof: boolean;
}

/**
 * Handle to a spawned process.
 */
interface ProcessHandle {
  readonly id: string;
  readonly pid: number;

  kill(signal?: string): Promise<boolean>;
  wait(): Promise<WaitResult>;
  status(): Promise<ProcessStatus>;
  writeStdin(data: string): Promise<void>;
  readStdout(): Promise<ReadOutput>;
  readStderr(): Promise<ReadOutput>;

  /** Async iterator for stdout */
  stdout: AsyncIterable<string>;
  /** Async iterator for stderr */
  stderr: AsyncIterable<string>;
}
```

## Lifecycle Hooks

Process operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:process";

// Log all spawned processes
onBefore("spawn", (args) => {
  console.log("Spawning process...");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:process";

onAfter("spawn", (result) => {
  console.log("Process spawned with PID:", result.pid);
});

onAfter("wait", (result) => {
  console.log("Process exited:", result.success ? "success" : "failure");
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:process";

onError("spawn", (error) => {
  console.error("Failed to spawn:", error.message);
});
```

### removeAllHooks(opName?)

```typescript
import { removeAllHooks } from "runtime:process";

// Remove hooks for specific operation
removeAllHooks("spawn");

// Remove all hooks
removeAllHooks();
```

**Available operation names:** `"spawn"`, `"kill"`, `"wait"`, `"status"`, `"writeStdin"`, `"readStdout"`, `"readStderr"`

## Handler System

Register custom handlers for process-related operations.

### registerHandler(name, handler)

```typescript
import { registerHandler, invokeHandler } from "runtime:process";

registerHandler("runBuild", async (projectPath: string) => {
  const proc = await spawn("npm", {
    args: ["run", "build"],
    cwd: projectPath
  });
  return await proc.wait();
});

// Later...
const result = await invokeHandler("runBuild", "/path/to/project");
```

### invokeHandler(name, ...args)

Invoke a registered handler by name.

### listHandlers()

```typescript
import { listHandlers } from "runtime:process";

const handlers = listHandlers();
console.log("Available handlers:", handlers);
```

### hasHandler(name) / removeHandler(name)

```typescript
import { hasHandler, removeHandler } from "runtime:process";

if (hasHandler("runBuild")) {
  removeHandler("runBuild");
}
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 4000 | IoError | I/O error during process operations |
| 4001 | PermissionDenied | Permission denied to spawn process |
| 4002 | NotFound | Process binary not found |
| 4003 | SpawnFailed | Failed to spawn process |
| 4004 | AlreadyExited | Process already exited |
| 4005 | Timeout | Operation timeout |
| 4006 | InvalidHandle | Invalid process handle |
| 4007 | StdinClosed | Process stdin is closed |
| 4008 | OutputNotCaptured | stdout/stderr not piped |
| 4009 | TooManyProcesses | Too many processes spawned |

## Complete Example

### Build Script Runner

```typescript
import { spawn } from "runtime:process";

/**
 * Run a build script and stream output in real-time.
 */
async function runBuild(projectPath: string): Promise<boolean> {
  console.log(`Starting build in ${projectPath}...`);

  const proc = await spawn("npm", {
    args: ["run", "build"],
    cwd: projectPath,
    env: {
      ...process.env,
      "NODE_ENV": "production",
      "CI": "true"
    },
    stdout: "piped",
    stderr: "piped"
  });

  console.log(`Build process started (PID: ${proc.pid})`);

  // Stream stdout
  const stdoutPromise = (async () => {
    for await (const line of proc.stdout) {
      console.log("[BUILD]", line);
    }
  })();

  // Stream stderr
  const stderrPromise = (async () => {
    for await (const line of proc.stderr) {
      console.error("[ERROR]", line);
    }
  })();

  // Wait for both streams and process to complete
  await Promise.all([stdoutPromise, stderrPromise]);
  const result = await proc.wait();

  if (result.success) {
    console.log("✅ Build completed successfully!");
    return true;
  } else {
    console.error(`❌ Build failed with code ${result.code}`);
    return false;
  }
}

// Usage
const success = await runBuild("/path/to/my-app");
```

### Interactive REPL

```typescript
import { spawn } from "runtime:process";

/**
 * Interactive Python session.
 */
async function pythonRepl() {
  const proc = await spawn("python3", {
    args: ["-i"],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped"
  });

  console.log("Python REPL started. Type expressions to evaluate.");

  // Read stdout in background
  (async () => {
    for await (const line of proc.stdout) {
      console.log(">>>", line);
    }
  })();

  // Read stderr (Python uses stderr for prompts)
  (async () => {
    for await (const line of proc.stderr) {
      if (!line.startsWith(">>>")) {
        console.error(line);
      }
    }
  })();

  // Execute some Python commands
  await proc.writeStdin("x = 10\n");
  await proc.writeStdin("y = 20\n");
  await proc.writeStdin("print(f'Sum: {x + y}')\n");
  await proc.writeStdin("exit()\n");

  const result = await proc.wait();
  console.log("Python exited with code:", result.code);
}
```

### Process Pool

```typescript
import { spawn, type ProcessHandle, type WaitResult } from "runtime:process";

/**
 * Process pool for parallel task execution.
 */
class ProcessPool {
  private maxConcurrent: number;
  private running: Map<string, ProcessHandle> = new Map();
  private queue: Array<() => Promise<void>> = [];

  constructor(maxConcurrent: number = 4) {
    this.maxConcurrent = maxConcurrent;
  }

  async run(binary: string, args: string[]): Promise<WaitResult> {
    return new Promise((resolve, reject) => {
      const task = async () => {
        try {
          const proc = await spawn(binary, { args });
          this.running.set(proc.id, proc);

          const result = await proc.wait();
          this.running.delete(proc.id);

          this.processQueue();
          resolve(result);
        } catch (error) {
          this.processQueue();
          reject(error);
        }
      };

      if (this.running.size < this.maxConcurrent) {
        task();
      } else {
        this.queue.push(task);
      }
    });
  }

  private processQueue() {
    while (this.queue.length > 0 && this.running.size < this.maxConcurrent) {
      const task = this.queue.shift()!;
      task();
    }
  }

  async killAll(): Promise<void> {
    const kills = Array.from(this.running.values()).map(proc =>
      proc.kill("SIGTERM")
    );
    await Promise.all(kills);
  }

  get activeCount(): number {
    return this.running.size;
  }
}

// Usage
const pool = new ProcessPool(4);

const tasks = [
  pool.run("process-file", ["file1.txt"]),
  pool.run("process-file", ["file2.txt"]),
  pool.run("process-file", ["file3.txt"]),
  pool.run("process-file", ["file4.txt"]),
  pool.run("process-file", ["file5.txt"]),
];

const results = await Promise.all(tasks);
console.log("All tasks completed:", results.every(r => r.success));
```

### Process with Timeout

```typescript
import { spawn } from "runtime:process";

/**
 * Run a command with a timeout.
 */
async function runWithTimeout(
  binary: string,
  args: string[],
  timeoutMs: number
): Promise<{ success: boolean; timedOut: boolean; output: string }> {
  const proc = await spawn(binary, {
    args,
    stdout: "piped",
    stderr: "piped"
  });

  let output = "";
  let timedOut = false;

  // Collect output
  const outputPromise = (async () => {
    for await (const line of proc.stdout) {
      output += line + "\n";
    }
  })();

  // Set up timeout
  const timeoutPromise = new Promise<void>((resolve) => {
    setTimeout(async () => {
      const status = await proc.status();
      if (status.running) {
        timedOut = true;
        await proc.kill("SIGKILL");
      }
      resolve();
    }, timeoutMs);
  });

  await Promise.race([outputPromise, timeoutPromise]);
  const result = await proc.wait();

  return {
    success: result.success && !timedOut,
    timedOut,
    output: output.trim()
  };
}

// Usage
const result = await runWithTimeout("slow-command", [], 5000);
if (result.timedOut) {
  console.error("Command timed out after 5 seconds");
} else if (result.success) {
  console.log("Output:", result.output);
}
```

## Best Practices

### Always Clean Up Processes

```typescript
const proc = await spawn("server");

try {
  // ... do work with the server
} finally {
  // Ensure process is terminated
  const status = await proc.status();
  if (status.running) {
    await proc.kill("SIGTERM");
    await proc.wait();
  }
}
```

### Handle Both stdout and stderr

```typescript
const proc = await spawn("command", {
  stdout: "piped",
  stderr: "piped"
});

// Process both streams concurrently
await Promise.all([
  (async () => {
    for await (const line of proc.stdout) {
      handleOutput(line);
    }
  })(),
  (async () => {
    for await (const line of proc.stderr) {
      handleError(line);
    }
  })()
]);
```

### Check Exit Codes

```typescript
const result = await proc.wait();

// Don't just check success
if (result.code !== 0) {
  if (result.signal) {
    console.error(`Process killed by ${result.signal}`);
  } else {
    console.error(`Process failed with code ${result.code}`);
  }
}
```

### Use Appropriate stdio Modes

```typescript
// For commands you want to see output: inherit
await spawn("npm", { args: ["install"], stdout: "inherit", stderr: "inherit" });

// For commands you want to process output: piped
const proc = await spawn("ls", { stdout: "piped" });
for await (const line of proc.stdout) { /* ... */ }

// For background commands: null
await spawn("daemon", { stdin: "null", stdout: "null", stderr: "null" });
```

## Platform Considerations

### Signal Names

| Signal | Unix | Windows |
|--------|------|---------|
| `SIGTERM` | ✅ Supported | ✅ Emulated |
| `SIGKILL` | ✅ Supported | ✅ Emulated |
| `SIGINT` | ✅ Supported | ✅ Supported |
| `SIGHUP` | ✅ Supported | ❌ Not supported |
| `SIGUSR1/2` | ✅ Supported | ❌ Not supported |

### Path Resolution

- Unix: Searches `$PATH` environment variable
- Windows: Searches `%PATH%` and adds `.exe`, `.cmd`, `.bat` extensions

```typescript
// Cross-platform command
const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const proc = await spawn(npm, { args: ["install"] });
```

## See Also

- [runtime-process.md](/docs/api/runtime-process) - API documentation
- [ext_shell](./ext-shell.md) - Shell integration utilities
- [ext_fs](./ext-fs.md) - File system operations
- [ext_signals](./ext-signals.md) - Signal handling
