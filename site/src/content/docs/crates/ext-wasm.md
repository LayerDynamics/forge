---
title: "ext_wasm"
description: WebAssembly runtime extension providing the runtime:wasm module.
slug: crates/ext-wasm
---

The `ext_wasm` crate provides comprehensive WebAssembly support for Forge applications through the `runtime:wasm` module, enabling you to load and execute WASM modules with full WASI support.

## Overview

ext_wasm integrates the [Wasmtime](https://wasmtime.dev/) WebAssembly runtime into Forge, providing:

- **Module compilation** - Compile WASM bytecode from bytes or files with AOT compilation
- **Instance management** - Create multiple independent instances from a single compiled module
- **Function calls** - Invoke exported WASM functions with automatic type conversion
- **Linear memory access** - Direct read/write access to WASM memory
- **WASI support** - Full WebAssembly System Interface with file system access
- **Capability-based security** - Controlled file system access via directory preopens

## Quick Start

```typescript
import * as wasm from "runtime:wasm";

// Compile WASM module
const wasmBytes = await Deno.readFile("module.wasm");
const moduleId = await wasm.compile(wasmBytes);

// Create instance
const instance = await wasm.instantiate(moduleId);

// Call exported function
const [result] = await instance.call("add", 10, 32);
console.log("Result:", result); // 42

// Cleanup
await instance.drop();
await wasm.dropModule(moduleId);
```

## Module: `runtime:wasm`

### Module Compilation

Compile WASM modules from bytes or files. Compiled modules are cached and can be reused for multiple instances.

```typescript
import { compile, compileFile } from "runtime:wasm";

// Compile from bytes
const wasmBytes = await Deno.readFile("module.wasm");
const moduleId = await compile(wasmBytes);

// Or compile directly from file
const moduleId2 = await compileFile("./module.wasm");
```

**Performance Note:** Compilation is expensive (~10-100ms). Cache the module ID and reuse it for multiple instances.

### Instance Creation

Create instances from compiled modules. Each instance has independent state and memory.

```typescript
import { instantiate } from "runtime:wasm";

// Basic instantiation
const instance = await instantiate(moduleId);

// With WASI configuration
const instance2 = await instantiate(moduleId, {
  preopens: { "/data": "./app-data" },
  env: { "LOG_LEVEL": "debug" },
  args: ["--verbose"],
  inheritStdout: true
});
```

### Function Calls

Call exported WASM functions with automatic type conversion.

```typescript
// Automatic type conversion
const [sum] = await instance.call("add", 10, 32);

// Multiple return values
const [quotient, remainder] = await instance.call("divmod", 42, 5);

// Explicit type control
import { types } from "runtime:wasm";
const [result] = await instance.call("process",
  types.i32(42),
  types.f64(3.14159)
);
```

### Memory Access

Read and write directly to WebAssembly linear memory.

```typescript
// Write string to memory
const text = "Hello, WASM!";
const bytes = new TextEncoder().encode(text);
await instance.memory.write(0, bytes);

// WASM processes the data
await instance.call("process_string", 0, bytes.length);

// Read result
const resultBytes = await instance.memory.read(1024, 256);
const result = new TextDecoder().decode(resultBytes);

// Check memory size (in 64KB pages)
const pages = await instance.memory.size();
console.log(`Memory: ${pages * 64}KB`);

// Grow memory if needed
if (pages < 16) {
  await instance.memory.grow(16 - pages);
}
```

## WASI Configuration

WASI (WebAssembly System Interface) provides system-level access to WASM modules.

### Directory Preopens

Map guest virtual paths to host directories for capability-based security.

```typescript
const instance = await instantiate(moduleId, {
  preopens: {
    "/data": "./app-data",      // Guest /data -> Host ./app-data
    "/config": "./config",       // Guest /config -> Host ./config
    "/tmp": "./temp-storage"     // Guest /tmp -> Host ./temp-storage
  }
});

// WASM module can now access these directories
// but CANNOT access other file system locations
```

**Security:** Only grant access to the minimum required directories. Never grant root access (`"/": "/"`).

### Environment Variables

Provide environment variables to WASM modules.

```typescript
const instance = await instantiate(moduleId, {
  env: {
    "DATABASE_URL": "sqlite:///data/app.db",
    "API_KEY": "secret-key-here",
    "LOG_LEVEL": "debug"
  }
});
```

### Command-Line Arguments

Pass arguments to WASM modules.

```typescript
const instance = await instantiate(moduleId, {
  args: ["--verbose", "--port", "3000", "--workers", "4"]
});
```

### Standard I/O

Inherit stdin/stdout/stderr from the host process.

```typescript
const instance = await instantiate(moduleId, {
  inheritStdin: true,   // WASM can read from host stdin
  inheritStdout: true,  // WASM output goes to host stdout
  inheritStderr: true   // WASM errors go to host stderr
});

// Useful for WASM CLI tools that need interactive I/O
```

## Multiple Instances

Create multiple independent instances from a single compiled module.

```typescript
import { compile, instantiate } from "runtime:wasm";

// Compile once
const moduleId = await compile(wasmBytes);

// Create worker instances
const workers = await Promise.all([
  instantiate(moduleId, { env: { "WORKER_ID": "1" } }),
  instantiate(moduleId, { env: { "WORKER_ID": "2" } }),
  instantiate(moduleId, { env: { "WORKER_ID": "3" } })
]);

// Process in parallel
await Promise.all(workers.map(worker =>
  worker.call("process_batch", batchId)
));

// Cleanup
await Promise.all(workers.map(w => w.drop()));
await dropModule(moduleId);
```

Each instance has:
- Independent linear memory
- Separate WASI state (file descriptors, environment)
- Isolated execution state

## Type System

WebAssembly supports four numeric value types.

### Value Types

| Type | Description | Range | JavaScript |
|------|-------------|-------|------------|
| `i32` | 32-bit integer | -2^31 to 2^31-1 | `number` |
| `i64` | 64-bit integer | -2^63 to 2^63-1 | `bigint` or `number` |
| `f32` | 32-bit float | IEEE 754 single | `number` |
| `f64` | 64-bit float | IEEE 754 double | `number` |

### Automatic Type Conversion

Arguments are automatically converted based on type and value range:

```typescript
// Integer in i32 range -> i32
await instance.call("process", 42);

// Integer outside i32 range -> i64
await instance.call("process", 9007199254740991);

// Float -> f64
await instance.call("process", 3.14159);

// BigInt -> i64
await instance.call("process", 9007199254740991n);
```

### Explicit Type Control

Use the `types` helper for precise type control:

```typescript
import { types } from "runtime:wasm";

// Force i32 even for small values
const [result] = await instance.call("add_i32",
  types.i32(10),
  types.i32(32)
);

// Force i64 for large values
const [largeResult] = await instance.call("add_i64",
  types.i64(9007199254740991n),
  types.i64(1n)
);

// Specify float precision
const [floatResult] = await instance.call("compute",
  types.f32(3.14159),  // Single precision
  types.f64(2.71828)   // Double precision
);
```

## Export Introspection

Discover available exports before calling functions.

```typescript
const exports = await instance.getExports();

// List all functions
const functions = exports.filter(e => e.kind === "func");
console.log("Available functions:", functions.map(f => f.name));

// Find memory export
const memory = exports.find(e => e.kind === "memory");
if (memory) {
  console.log("Memory export:", memory.name);
}

// Check if function exists
const hasAdd = exports.some(e =>
  e.kind === "func" && e.name === "add"
);
if (hasAdd) {
  const [result] = await instance.call("add", 10, 32);
}
```

Export kinds:
- `"func"` - Exported function
- `"memory"` - Exported linear memory
- `"table"` - Exported table
- `"global"` - Exported global variable

## Error Handling

All operations return structured errors with machine-readable codes.

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 5000 | CompileError | Failed to compile WASM module |
| 5001 | InstantiateError | Failed to instantiate module |
| 5002 | CallError | Function call failed |
| 5003 | ExportNotFound | Export not found in module |
| 5004 | InvalidModuleHandle | Invalid module handle |
| 5005 | InvalidInstanceHandle | Invalid instance handle |
| 5006 | MemoryError | Memory access out of bounds |
| 5007 | TypeError | Type mismatch in function call |
| 5008 | IoError | IO error (file loading) |
| 5009 | PermissionDenied | Permission denied by capability system |
| 5010 | WasiError | WASI configuration error |
| 5011 | FuelExhausted | Fuel exhaustion (execution limit) |

### Error Handling Examples

```typescript
// Compilation errors
try {
  const moduleId = await compile(invalidBytes);
} catch (error) {
  // Error 5000: Invalid WASM bytecode
  console.error("Compilation failed:", error.message);
}

// Function call errors
try {
  await instance.call("nonexistent");
} catch (error) {
  // Error 5003: Export not found
  console.error("Function not found:", error.message);
}

// Memory access errors
try {
  await instance.memory.read(999999999, 1024);
} catch (error) {
  // Error 5006: Out of bounds
  console.error("Memory access failed:", error.message);
}

// WASI permission errors
try {
  const instance = await instantiate(moduleId, {
    preopens: { "/sensitive": "/etc" }  // May be denied
  });
} catch (error) {
  // Error 5009 or 5010: Permission denied
  console.error("WASI configuration failed:", error.message);
}
```

## Performance Tips

### Compile Once, Instantiate Many

```typescript
// ❌ BAD: Recompile for each instance
for (let i = 0; i < 10; i++) {
  const moduleId = await compile(wasmBytes);  // Wasteful!
  const instance = await instantiate(moduleId);
}

// ✅ GOOD: Compile once, reuse
const moduleId = await compile(wasmBytes);
const instances = await Promise.all(
  Array(10).fill(null).map(() => instantiate(moduleId))
);
```

### Batch Memory Operations

```typescript
// ❌ BAD: Multiple small reads
for (let i = 0; i < 1000; i++) {
  const byte = await instance.memory.read(i, 1);
  process(byte);
}

// ✅ GOOD: Single large read
const allBytes = await instance.memory.read(0, 1000);
for (let i = 0; i < 1000; i++) {
  process(allBytes[i]);
}
```

### Minimize Cross-Boundary Calls

```typescript
// ❌ BAD: Many small function calls
for (let i = 0; i < 1000; i++) {
  await instance.call("process_item", i);
}

// ✅ GOOD: Batch processing in WASM
await instance.call("process_batch", 0, 1000);
```

## Common Patterns

### Data Processing Pipeline

```typescript
import { compile, instantiate } from "runtime:wasm";

// Compile image processing module
const moduleId = await compile(await Deno.readFile("image_process.wasm"));
const instance = await instantiate(moduleId);

// Load image into memory
const imageData = await Deno.readFile("input.png");
await instance.memory.write(0, imageData);

// Process in WASM
await instance.call("apply_filter", 0, imageData.length, 1024);

// Read result
const processed = await instance.memory.read(1024, imageData.length);
await Deno.writeFile("output.png", processed);

await instance.drop();
```

### Plugin System

```typescript
import { compileFile, instantiate } from "runtime:wasm";

// Load user plugins
const pluginFiles = await Array.fromAsync(Deno.readDir("./plugins"));
const plugins = [];

for (const file of pluginFiles.filter(f => f.name.endsWith(".wasm"))) {
  const moduleId = await compileFile(`./plugins/${file.name}`);
  const instance = await instantiate(moduleId, {
    preopens: { "/data": "./plugin-data" }
  });
  plugins.push({ name: file.name, instance });
}

// Execute plugins
for (const plugin of plugins) {
  const [result] = await plugin.instance.call("execute", taskId);
  console.log(`Plugin ${plugin.name} result:`, result);
}
```

### Worker Pool

```typescript
import { compile, instantiate } from "runtime:wasm";

class WasmWorkerPool {
  constructor(moduleId, size) {
    this.workers = [];
    this.available = [];

    // Create worker instances
    for (let i = 0; i < size; i++) {
      const instance = await instantiate(moduleId);
      this.workers.push(instance);
      this.available.push(instance);
    }
  }

  async execute(funcName, ...args) {
    // Wait for available worker
    while (this.available.length === 0) {
      await new Promise(r => setTimeout(r, 10));
    }

    const worker = this.available.pop();
    try {
      return await worker.call(funcName, ...args);
    } finally {
      this.available.push(worker);
    }
  }

  async cleanup() {
    await Promise.all(this.workers.map(w => w.drop()));
  }
}

// Usage
const moduleId = await compile(wasmBytes);
const pool = new WasmWorkerPool(moduleId, 4);

// Execute tasks in parallel
const results = await Promise.all([
  pool.execute("process", 1),
  pool.execute("process", 2),
  pool.execute("process", 3)
]);

await pool.cleanup();
```

## Common Pitfalls

### 1. Resource Cleanup Order

```typescript
// ❌ ERROR: Drop module before instances
await dropModule(moduleId);
await instance.drop();  // Instance now invalid!

// ✅ CORRECT: Drop instances before module
await instance.drop();
await dropModule(moduleId);
```

### 2. Large Integer Precision

```typescript
// ❌ JavaScript loses precision for large i64
const [result] = await instance.call("process_i64", 9007199254740992);

// ✅ Use BigInt for values outside safe integer range
const [result] = await instance.call("process_i64",
  types.i64(9007199254740992n)
);
```

### 3. Memory Growth Assumptions

```typescript
// ❌ Assuming growth always succeeds
await instance.memory.grow(1000);  // May fail!

// ✅ Handle growth failure
try {
  const oldSize = await instance.memory.grow(pagesNeeded);
  console.log(`Grew from ${oldSize} pages`);
} catch (error) {
  console.error("Failed to grow memory:", error);
  // Use existing memory or fail gracefully
}
```

### 4. Unsafe Preopen Paths

```typescript
// ❌ DANGEROUS: Granting root access
await instantiate(moduleId, {
  preopens: { "/": "/" }  // Full file system access!
});

// ✅ SAFE: Minimal required access
await instantiate(moduleId, {
  preopens: {
    "/data": "./app-data",
    "/tmp": "./temp-storage"
  }
});
```

## Implementation Details

### Architecture

```text
TypeScript Application
  ↓ compile(bytes)
WasmState (Rust)
  ├─ Wasmtime Engine (shared)
  ├─ Compiled Modules (HashMap)
  └─ Instances (HashMap)
      ├─ Store (per-instance)
      ├─ WASI Context
      └─ Linear Memory
```

### State Management

- `WasmState`: Thread-safe state wrapped in `Arc<Mutex<>>`
- `Engine`: Shared Wasmtime engine for all modules
- `Module`: Compiled WASM bytecode (cached)
- `Instance`: Runtime instance with independent state
- `Store`: Per-instance execution context
- `WasiP1Ctx`: WASI preview1 context with preopens

### Wasmtime Integration

Forge uses Wasmtime 27.0 for WebAssembly execution:
- AOT compilation to native machine code
- WASI preview1 support via `wasmtime-wasi`
- Capability-based file system access
- Memory bounds checking
- Type validation

## Testing

Run the ext_wasm test suite:

```bash
# All tests
cargo test -p ext_wasm

# With output
cargo test -p ext_wasm -- --nocapture

# Specific test
cargo test -p ext_wasm test_compile_and_instantiate

# With debug logging
RUST_LOG=ext_wasm=debug cargo test -p ext_wasm -- --nocapture
```

Test coverage:
- Module compilation from bytes and files
- Instance creation with and without WASI
- Function calls with all value types (i32, i64, f32, f64)
- Multiple return values
- Linear memory operations (read, write, size, grow)
- Export introspection
- Error handling for all error codes
- WASI file system access via preopens
- Multiple instances from single module
- Resource cleanup (drop instance, drop module)

## Related Documentation

- [Wasmtime Documentation](https://docs.wasmtime.dev/) - Wasmtime runtime details
- [WASI Specification](https://github.com/WebAssembly/WASI) - WASI standard
- [WebAssembly Specification](https://webassembly.github.io/spec/) - Core WASM spec
- [forge-weld](/docs/crates/forge-weld) - Code generation library
- [ext_fs](/docs/crates/ext-fs) - File system operations
- [ext_process](/docs/crates/ext-process) - Process spawning

## API Reference

For complete API documentation with all types and methods, see:
- [runtime:wasm API Reference](/docs/api/runtime-wasm) - Generated TypeScript API docs

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `wasmtime` | 27.0 | WebAssembly runtime and JIT compiler |
| `wasmtime-wasi` | 27.0 | WASI preview1 implementation |
| `deno_core` | 0.373 | Op definitions and runtime integration |
| `tokio` | 1.x | Async runtime for mutex synchronization |
| `serde` | 1.x | Serialization framework |
| `forge-weld-macro` | 0.1 | TypeScript binding generation macros |
| `forge-weld` | 0.1 | Build-time code generation |

## Platform Support

| Platform | Support | Notes |
|----------|---------|-------|
| macOS (x64) | ✅ Full | Native Wasmtime support |
| macOS (ARM) | ✅ Full | M1/M2/M3 optimized |
| Linux (x64) | ✅ Full | Native Wasmtime support |
| Linux (ARM) | ✅ Full | ARMv8 support |
| Windows (x64) | ✅ Full | Native Wasmtime support |
| Windows (ARM) | ⚠️ Limited | May have issues with some modules |
