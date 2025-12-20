# ext_process

Child process spawning and management for Forge applications.

## Overview

`ext_process` is a Forge extension that provides cross-platform child process spawning with comprehensive I/O management, lifecycle control, and capability-based security. It enables Forge applications to:

- Execute external commands and scripts
- Interact with long-running processes through bidirectional communication
- Manage process lifecycles with signals and status monitoring
- Stream output asynchronously with async iterators
- Control resource usage with configurable process limits

The extension wraps Rust's Tokio process primitives and exposes them to TypeScript via the `runtime:process` module.

## TypeScript Usage

### Basic Command Execution

```typescript
import { spawn } from "runtime:process";

const proc = await spawn("echo", { args: ["Hello, World!"] });

// Read output line by line
for await (const line of proc.stdout) {
  console.log(line); // "Hello, World!"
}

// Wait for completion
const result = await proc.wait();
console.log(`Exit code: ${result.code}`); // 0
```

### Interactive Process Communication

```typescript
const python = await spawn("python3", {
  args: ["-i"],
  stdin: "piped",   // Enable writing to stdin
  stdout: "piped",  // Enable reading from stdout
  stderr: "piped",  // Enable reading from stderr
  cwd: "/path/to/project"
});

// Write commands to Python REPL
await python.writeStdin("import math\n");
await python.writeStdin("print(math.pi)\n");

// Read response
const output = await python.readStdout();
console.log(output.data); // "3.141592653589793"

// Clean up
await python.kill("SIGTERM");
```

### Custom Environment and Working Directory

```typescript
const build = await spawn("npm", {
  args: ["run", "build"],
  cwd: "./frontend",
  env: {
    "NODE_ENV": "production",
    "API_URL": "https://api.example.com"
  }
});

// Stream build output
for await (const line of proc.stdout) {
  console.log(`[build] ${line}`);
}

const result = await build.wait();
if (!result.success) {
  throw new Error(`Build failed with code ${result.code}`);
}
```

### Long-Running Process Management

```typescript
const server = await spawn("./server", {
  stdout: "piped",
  stderr: "piped"
});

console.log(`Server started with PID: ${server.pid}`);

// Monitor health
const healthCheck = setInterval(async () => {
  const status = await server.status();
  if (!status.running) {
    console.error(`Server exited with code ${status.exitCode}`);
    clearInterval(healthCheck);
  }
}, 5000);

// Graceful shutdown
addEventListener("beforeunload", async () => {
  console.log("Shutting down server...");
  await server.kill("SIGTERM");
  await server.wait();
  clearInterval(healthCheck);
});
```

## Permissions

### Manifest Configuration

Process spawning requires explicit permissions in `manifest.app.toml`:

```toml
[permissions.process]
# Allow specific binaries
spawn = ["node", "python3", "/usr/bin/ffmpeg", "./scripts/build.sh"]

# Or allow all (not recommended for production)
spawn = ["*"]
```

### Development vs Production

- **Development mode** (`forge dev`): All processes allowed by default
- **Production mode** (bundled apps): Only processes in manifest allowlist can be spawned
- Permission violations throw error **4001** (`PermissionDenied`)

## Error Codes

All errors include structured error codes for programmatic handling:

| Code | Error Type | Meaning | How to Fix |
|------|------------|---------|------------|
| **4000** | `Io` | Generic I/O error during process operation | Check system logs, verify binary exists, retry operation |
| **4001** | `PermissionDenied` | Binary not in manifest spawn allowlist | Add binary to `[permissions.process].spawn` array in manifest.app.toml |
| **4002** | `NotFound` | Binary not found in PATH | Install binary or use absolute path |
| **4003** | `FailedToSpawn` | Process spawn failed | Check binary has execute permissions, verify arguments are valid |
| **4004** | `ProcessExited` | Operation attempted on exited process | Check exit code, restart process if needed |
| **4005** | `Timeout` | Operation timed out | Increase timeout, check process is responding |
| **4006** | `InvalidHandle` | Process handle no longer valid | Verify process still exists, check for race conditions |
| **4007** | `StdinClosed` | Cannot write to closed stdin | Check process still running, ensure stdin was configured as "piped" |
| **4008** | `OutputNotCaptured` | stdout/stderr not configured for reading | Set `stdout: "piped"` or `stderr: "piped"` in spawn options |
| **4009** | `TooManyProcesses` | Concurrent process limit reached | Wait for processes to exit, or increase max_processes limit |

### Error Handling Example

```typescript
try {
  const proc = await spawn("nonexistent-binary");
} catch (error) {
  if (error.message.includes("[4002]")) {
    console.error("Binary not found. Please install it first.");
  } else if (error.message.includes("[4001]")) {
    console.error("Permission denied. Update manifest.app.toml");
  } else {
    console.error("Unexpected error:", error);
  }
}
```

## Common Patterns

### Pattern 1: Run and Capture Output

```typescript
async function runCommand(binary: string, args: string[]): Promise<string> {
  const proc = await spawn(binary, { args, stdout: "piped" });

  let output = "";
  for await (const line of proc.stdout) {
    output += line + "\n";
  }

  const result = await proc.wait();
  if (!result.success) {
    throw new Error(`Command failed with code ${result.code}`);
  }

  return output.trim();
}

const gitBranch = await runCommand("git", ["branch", "--show-current"]);
console.log(`Current branch: ${gitBranch}`);
```

### Pattern 2: Stream Processing with Error Handling

```typescript
const proc = await spawn("npm", {
  args: ["install"],
  stdout: "piped",
  stderr: "piped"
});

// Process stdout and stderr concurrently
const [stdoutDone, stderrDone] = await Promise.all([
  (async () => {
    for await (const line of proc.stdout) {
      console.log(`[npm] ${line}`);
    }
  })(),
  (async () => {
    for await (const line of proc.stderr) {
      console.error(`[npm:error] ${line}`);
    }
  })()
]);

const result = await proc.wait();
```

### Pattern 3: Process Pool with Limits

```typescript
class ProcessPool {
  private active = new Set<ProcessHandle>();

  async run(binary: string, args: string[], maxConcurrent = 5): Promise<void> {
    // Wait if at limit
    while (this.active.size >= maxConcurrent) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    const proc = await spawn(binary, { args });
    this.active.add(proc);

    proc.wait().finally(() => {
      this.active.delete(proc);
    });
  }

  async waitAll(): Promise<void> {
    await Promise.all(
      Array.from(this.active).map(p => p.wait())
    );
  }
}
```

## Platform Notes

### macOS & Linux (Unix)

- **Signals**: Full support for SIGTERM, SIGKILL, SIGINT, SIGHUP, SIGUSR1, SIGUSR2
- **Exit Status**: Includes both exit codes and signal information
- **File Descriptors**: Child processes inherit parent's FDs (configure stdio carefully)
- **PATH Resolution**: Uses shell PATH resolution

### Windows

- **Signals**: Limited support - `kill()` uses `TerminateProcess` API
- **Signal Parameter**: Ignored in `kill()` operations
- **Exit Codes**: Uses Windows process exit conventions
- **PATH Resolution**: Uses Windows PATH resolution with `.exe` extension handling

### Cross-Platform Best Practices

1. **Signal Handling**: Use `SIGTERM` for graceful shutdown, but have fallback
   ```typescript
   await proc.kill("SIGTERM");
   await setTimeout(() => proc.kill("SIGKILL"), 5000); // Fallback
   ```

2. **Path Separators**: Use `path` module for cross-platform paths
   ```typescript
   import { join } from "runtime:path";
   const scriptPath = join("scripts", "build.sh");
   ```

3. **Binary Naming**: Account for `.exe` extension on Windows
   ```typescript
   const binary = Deno.build.os === "windows" ? "node.exe" : "node";
   ```

## Testing

### Running Tests

```bash
# Run all extension tests
cargo test -p ext_process

# Run with debug logging
RUST_LOG=ext_process=debug cargo test -p ext_process -- --nocapture

# Run specific test
cargo test -p ext_process test_process_spawn
```

### Testing Considerations

1. **Permissions**: Tests use `PermissiveProcessChecker` (allow all)
2. **Binaries**: Tests assume `echo` is available on all platforms
3. **Concurrency**: Process limit tests verify state management without spawning
4. **Platform**: Some tests are conditionally compiled for Unix/Windows

### Writing New Tests

```rust
#[tokio::test]
async fn test_custom_scenario() {
    let mut state = OpState::new();
    init_process_state(&mut state, None, Some(10));

    // Your test logic here
}
```

## Architecture

### State Management

The extension maintains process state in Deno's `OpState`:

```rust
pub struct ProcessState {
    pub processes: HashMap<String, ProcessHandle>,  // Active processes
    pub next_id: u64,                                // ID generator
    pub max_processes: usize,                        // Resource limit
}
```

### Process Handle Structure

Each spawned process gets a unique handle:

```rust
pub struct ProcessHandle {
    pub child: Arc<Mutex<Child>>,                    // Tokio child process
    pub pid: u32,                                    // OS process ID
    pub binary: String,                              // Executable name
    pub stdout: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    pub stderr: Option<Arc<Mutex<BufReader<ChildStderr>>>>,
    pub stdin: Option<Arc<Mutex<ChildStdin>>>,
    pub exited: bool,                                // Exit state cache
    pub exit_code: Option<i32>,                      // Cached exit code
}
```

### I/O Streaming

- **Stdout/Stderr**: Wrapped in `Arc<Mutex<BufReader<>>>` for concurrent line-buffered reads
- **Stdin**: Wrapped in `Arc<Mutex<>>` with explicit flushing after writes
- **Async Iteration**: TypeScript async iterators read until EOF

### Security Model

```rust
pub trait ProcessCapabilityChecker: Send + Sync {
    fn check_spawn(&self, binary: &str) -> Result<(), String>;
    fn check_env(&self, key: &str) -> Result<(), String>;
}
```

Runtime initializes with custom checker:

```rust
init_process_state(
    &mut op_state,
    Some(Arc::new(ManifestProcessChecker::new(manifest))),
    Some(20) // max concurrent
);
```

## Build System Integration

### Code Generation

This extension uses `forge-weld` for TypeScript binding generation:

```rust
// In build.rs
ExtensionBuilder::new("runtime_process", "runtime:process")
    .ts_path("ts/init.ts")
    .ops(&[
        "op_process_spawn",
        "op_process_kill",
        "op_process_wait",
        "op_process_status",
        "op_process_write_stdin",
        "op_process_read_stdout",
        "op_process_read_stderr",
    ])
    .generate_sdk_module("sdk")
    .use_inventory_types()
    .build()
```

### Build Artifacts

- **Generated SDK**: `sdk/runtime.process.ts` (auto-generated from `ts/init.ts`)
- **Extension Module**: `OUT_DIR/extension.rs` (included at compile time)

### Rebuilding

```bash
# Trigger regeneration of TypeScript SDK
cargo build -p ext_process

# Generate Rust docs
cargo doc -p ext_process --no-deps --open
```

## See Also

- **TypeScript SDK**: [`sdk/runtime.process.ts`](../../sdk/runtime.process.ts) - Generated SDK with full JSDoc
- **TypeScript Source**: [`ts/init.ts`](./ts/init.ts) - Source for SDK generation
- **Astro Docs**: `site/src/content/docs/crates/ext-process.md` - User-facing documentation
- **API Docs**: https://docs.rs/ext_process - Published Rust documentation
- **Examples**: [`examples/`](../../examples/) - Sample Forge applications using process spawning

## Contributing

When adding new operations:

1. Add `#[weld_op]` macro **before** `#[op2]` (required for code generation)
2. Update `build.rs` to include new op in `.ops(&[...])` list
3. Add TypeScript wrapper in `ts/init.ts` with comprehensive JSDoc
4. Add tests in `tests` module
5. Rebuild to regenerate SDK: `cargo build -p ext_process`
6. Update this README and Astro docs

## License

See [LICENSE](../../LICENSE) in the repository root.
