# ext_wasm

WebAssembly (WASM) extension for Forge runtime.

## Overview

`ext_wasm` provides comprehensive WebAssembly support for Forge applications using the [Wasmtime](https://wasmtime.dev/) runtime. It enables loading and executing WASM modules with full WASI (WebAssembly System Interface) support, linear memory access, and capability-based security.

**Runtime Module:** `runtime:wasm`

## Features

### Module Management
- Compile WASM modules from bytes or files
- Cache compiled modules for reuse across multiple instances
- Drop modules to free resources
- Automatic validation and error reporting

### Instance Management
- Create multiple independent instances from a single compiled module
- Each instance has its own linear memory and state
- Instance-level resource management
- Automatic cleanup on drop

### Function Calls
- Invoke exported WASM functions with type-safe arguments
- Automatic type conversion between JavaScript and WASM types (i32, i64, f32, f64)
- Support for multiple return values
- Introspect available exports

### Linear Memory Access
- Read and write memory at arbitrary offsets
- Page-based memory sizing (64KB pages)
- Dynamic memory growth
- Efficient Uint8Array integration
- Bounds-checked access

### WASI Support
- Full WASI preview1 implementation
- File system access with directory preopens (capability-based security)
- Environment variables
- Command-line arguments
- Standard I/O inheritance (stdin, stdout, stderr)

## Usage

### Basic Module Compilation and Execution

```typescript
import * as wasm from "runtime:wasm";

// Load and compile WASM module
const wasmBytes = await Deno.readFile("module.wasm");
const moduleId = await wasm.compile(wasmBytes);

// Create instance
const instance = await wasm.instantiate(moduleId);

// Call exported function
const [result] = await instance.call("add", 10, 32);
console.log("10 + 32 =", result); // 42

// Cleanup
await instance.drop();
await wasm.dropModule(moduleId);
```

### WASI Configuration

```typescript
import { compile, instantiate } from "runtime:wasm";

const moduleId = await compile(wasmBytes);

// Configure WASI environment
const instance = await instantiate(moduleId, {
  // Map guest paths to host directories (capability-based security)
  preopens: {
    "/data": "./app-data",
    "/config": "./config"
  },
  // Provide environment variables
  env: {
    "DATABASE_URL": "sqlite:///data/app.db",
    "LOG_LEVEL": "debug"
  },
  // Command-line arguments
  args: ["--verbose", "--port", "3000"],
  // Inherit standard I/O from host
  inheritStdout: true,
  inheritStderr: true
});

// WASM module can now access /data and /config directories
await instance.call("main");
```

### Linear Memory Access

```typescript
import { compile, instantiate } from "runtime:wasm";

const instance = await instantiate(moduleId);

// Write string to memory
const text = "Hello, WASM!";
const bytes = new TextEncoder().encode(text);
await instance.memory.write(0, bytes);

// WASM module processes the data
await instance.call("process_string", 0, bytes.length);

// Read result from memory
const resultBytes = await instance.memory.read(1024, 256);
const result = new TextDecoder().decode(resultBytes);
console.log("Result:", result);

// Check memory size (in 64KB pages)
const pages = await instance.memory.size();
console.log(`Memory: ${pages} pages (${pages * 64}KB)`);

// Grow memory if needed
if (pages < 16) {  // Need at least 1MB
  const oldSize = await instance.memory.grow(16 - pages);
  console.log(`Grew memory from ${oldSize} to ${16} pages`);
}
```

### Multiple Instances

```typescript
import { compile, instantiate, dropModule } from "runtime:wasm";

// Compile module once
const moduleId = await compile(wasmBytes);

// Create multiple instances (each with independent state/memory)
const instances = await Promise.all([
  instantiate(moduleId, { env: { "WORKER_ID": "1" } }),
  instantiate(moduleId, { env: { "WORKER_ID": "2" } }),
  instantiate(moduleId, { env: { "WORKER_ID": "3" } })
]);

// Process in parallel
await Promise.all(instances.map(inst => inst.call("process_data")));

// Cleanup all instances
await Promise.all(instances.map(inst => inst.drop()));

// Drop the module
await dropModule(moduleId);
```

### Explicit Type Control

```typescript
import { types } from "runtime:wasm";

// Force specific WASM types
const [result] = await instance.call("multiply",
  types.i32(7),     // Explicitly 32-bit integer
  types.i32(6)
);

// 64-bit integers (use BigInt for large values)
const [largeResult] = await instance.call("add_i64",
  types.i64(9007199254740991n),  // Max safe integer + 1
  types.i64(1n)
);

// Floating point precision control
const [precise] = await instance.call("compute",
  types.f64(3.141592653589793),  // Double precision
  types.f32(2.71828)              // Single precision
);
```

### Export Introspection

```typescript
const instance = await instantiate(moduleId);

// List all exports
const exports = await instance.getExports();
console.log("Available exports:", exports);

// Filter by kind
const functions = exports.filter(e => e.kind === "func");
console.log("Functions:", functions.map(f => f.name));

const memory = exports.find(e => e.kind === "memory");
if (memory) {
  console.log("Memory export:", memory.name);
}

// Check if specific function exists
const hasProcess = exports.some(e =>
  e.kind === "func" && e.name === "process"
);
```

## Architecture

### Module Compilation Flow

```text
Deno.readFile("module.wasm")
  ↓
compile(bytes)
  ↓ Wasmtime AOT compilation
Compiled Module (cached in WasmState)
  ↓
instantiate(moduleId, config?)
  ↓ Create Store + WASI context
Instance (ready for function calls)
  ↓
call("function_name", args...)
  ↓ Cross-boundary invocation
WASM function execution
  ↓
Return values (converted to JavaScript)
```

### State Management

```text
WasmState (Arc<Mutex<>>)
├─ engine: wasmtime::Engine (shared)
├─ modules: HashMap<String, WasmModule>
│   └─ module: wasmtime::Module (compiled bytecode)
├─ instances: HashMap<String, WasmInstance>
│   ├─ store: wasmtime::Store<WasmStoreData>
│   ├─ instance: wasmtime::Instance
│   ├─ memory: Option<wasmtime::Memory>
│   └─ wasi_ctx: WasiP1Ctx
└─ next_module_id / next_instance_id (counters)
```

### WASI Integration

```text
WasiConfig (TypeScript)
  ↓ Convert to Rust
WasiCtxBuilder
  ├─ preopens → DirPerms + FilePerms
  ├─ env → Environment variables
  ├─ args → Command-line arguments
  └─ inherit_std* → Stdio inheritance
  ↓
WasiP1Ctx (injected into Store)
  ↓
WASM module imports WASI functions
  ↓
File system access, I/O operations
```

## Error Handling

All operations return structured errors with machine-readable error codes (5000-5011).

### Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 5000 | CompileError | Failed to compile WASM module |
| 5001 | InstantiateError | Failed to instantiate module |
| 5002 | CallError | Function call failed |
| 5003 | ExportNotFound | Export not found in module |
| 5004 | InvalidModuleHandle | Invalid module handle |
| 5005 | InvalidInstanceHandle | Invalid instance handle |
| 5006 | MemoryError | Memory access error |
| 5007 | TypeError | Type mismatch in function call |
| 5008 | IoError | IO error (file loading) |
| 5009 | PermissionDenied | Permission denied by capability system |
| 5010 | WasiError | WASI configuration error |
| 5011 | FuelExhausted | Fuel exhaustion (execution limit) |

### Error Handling Patterns

```typescript
import { compile, instantiate } from "runtime:wasm";

// Compilation errors
try {
  const moduleId = await compile(invalidBytes);
} catch (error) {
  // Error 5000: Invalid WASM bytecode
  console.error("Compilation failed:", error);
}

// Instantiation errors
try {
  const instance = await instantiate("invalid-id");
} catch (error) {
  // Error 5004: Invalid module handle
  console.error("Instantiation failed:", error);
}

// Function call errors
try {
  await instance.call("nonexistent");
} catch (error) {
  // Error 5003: Export not found
  console.error("Call failed:", error);
}

// Memory access errors
try {
  await instance.memory.read(999999999, 1024);
} catch (error) {
  // Error 5006: Out of bounds
  console.error("Memory access failed:", error);
}

// WASI permission errors
try {
  const instance = await instantiate(moduleId, {
    preopens: { "/root": "/" }  // Attempting to grant root access
  });
} catch (error) {
  // Error 5009 or 5010: Permission denied or WASI error
  console.error("WASI configuration failed:", error);
}
```

## Implementation Details

### Module Compilation

Modules are compiled using Wasmtime's AOT (Ahead-of-Time) compiler:
1. Parse WASM bytecode
2. Validate module structure
3. Compile to native machine code
4. Store in `WasmState.modules` HashMap
5. Return opaque module ID

Compiled modules are cached and can be reused for multiple instances.

### Instance Creation

Instance creation involves:
1. Retrieve compiled module from cache
2. Create new Wasmtime `Store` with `WasmStoreData`
3. Build WASI context using `WasiCtxBuilder` (if config provided)
4. Link WASI imports using `preview1::add_to_linker`
5. Instantiate module in the store
6. Extract memory export (if present)
7. Store instance in `WasmState.instances` HashMap
8. Return opaque instance ID

Each instance has independent:
- Linear memory (separate allocation)
- WASI state (file descriptors, environment)
- Execution state (stack, globals)

### Function Calls

Cross-boundary function calls:
1. Look up instance by ID
2. Find export by name and verify it's a function
3. Convert JavaScript arguments to WASM values
   - Numbers → i32/i64/f32/f64 based on type/range
   - BigInt → i64
   - WasmValue → specified type
4. Invoke function via Wasmtime
5. Convert WASM return values to JavaScript
6. Return array of unwrapped values

Type conversion is automatic but can be overridden using `types` helpers.

### Linear Memory Access

Memory operations:
- **Read**: Copy bytes from WASM memory to JavaScript Uint8Array
- **Write**: Copy bytes from JavaScript Uint8Array to WASM memory
- **Size**: Return current memory size in 64KB pages
- **Grow**: Attempt to grow memory by specified number of pages

Memory is bounds-checked - out-of-bounds access throws Error 5006.

### WASI Preopens

Directory preopens use capability-based security:
1. Guest path (`/data`) mapped to host path (`./app-data`)
2. Host path converted to absolute, canonical form
3. Directory opened with `DirPerms` and `FilePerms`
4. WASM module can only access files within preopen directories

This prevents WASM modules from accessing arbitrary file system locations.

## Performance Considerations

- **Compilation**: Expensive (~10-100ms for typical modules). Cache and reuse compiled modules.
- **Instance Creation**: Relatively cheap (~100μs). Safe to create multiple instances.
- **Memory Access**: Requires data copying between WASM and JavaScript. Minimize round trips for large data.
- **Function Calls**: Cross-boundary overhead (~1μs per call). Batch operations when possible.
- **Memory Growth**: Expensive operation. Allocate sufficient initial memory in WASM module.

### Optimization Tips

```typescript
// ❌ BAD: Recompile module for each instance
for (let i = 0; i < 10; i++) {
  const moduleId = await compile(wasmBytes);  // Wasteful!
  const instance = await instantiate(moduleId);
}

// ✅ GOOD: Compile once, instantiate multiple times
const moduleId = await compile(wasmBytes);
const instances = await Promise.all(
  Array(10).fill(null).map(() => instantiate(moduleId))
);

// ❌ BAD: Multiple small memory reads
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

## Thread Safety

- `WasmState` is wrapped in `Arc<Mutex<>>` for safe concurrent access from multiple threads
- Wasmtime `Engine` is `Send + Sync` and shared across all modules
- `Store` and `Instance` are not thread-safe and protected by the state mutex
- All ops acquire the mutex lock, perform operation, then release
- Wasmtime handles internal synchronization for compiled modules

## Platform Support

| Platform | Support | Notes |
|----------|---------|-------|
| macOS (x64) | ✅ Full | Native support |
| macOS (ARM) | ✅ Full | Native support (M1/M2/M3) |
| Linux (x64) | ✅ Full | Native support |
| Linux (ARM) | ✅ Full | Native support |
| Windows (x64) | ✅ Full | Native support |
| Windows (ARM) | ⚠️ Limited | May have issues with some WASM modules |

Wasmtime provides native compilation for all major platforms.

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `wasmtime` | 27.0 | WebAssembly runtime and JIT compiler |
| `wasmtime-wasi` | 27.0 | WASI preview1 implementation |
| `deno_core` | 0.373 | Op definitions and runtime integration |
| `tokio` | 1.x | Async runtime for mutex synchronization |
| `serde` | 1.x | Serialization framework |
| `forge-weld-macro` | 0.1 | TypeScript binding generation |
| `forge-weld` | 0.1 | Build-time code generation |
| `linkme` | 0.3 | Compile-time symbol collection |

## Testing

```bash
# Run all tests
cargo test -p ext_wasm

# Run with output
cargo test -p ext_wasm -- --nocapture

# Run specific test
cargo test -p ext_wasm test_compile_and_instantiate

# With debug logging
RUST_LOG=ext_wasm=debug cargo test -p ext_wasm -- --nocapture
```

Tests cover:
- Module compilation from bytes and files
- Instance creation with and without WASI
- Function calls with various argument types (i32, i64, f32, f64)
- Multiple return values
- Linear memory access (read, write, size, grow)
- Export introspection
- Error handling for invalid operations
- WASI file system access via preopens
- Multiple instances from single module
- Resource cleanup (drop instance, drop module)

## Common Pitfalls

### 1. Not Dropping Instances Before Module

```typescript
// ❌ ERROR: Instances still reference the module
await dropModule(moduleId);
await instance.drop();  // Instance already invalid!

// ✅ CORRECT: Drop instances first
await instance.drop();
await dropModule(moduleId);
```

### 2. Incorrect Type Conversion

```typescript
// ❌ JavaScript number precision loss for large i64
const [result] = await instance.call("process_i64", 9007199254740992);

// ✅ Use BigInt for values outside safe integer range
const [result] = await instance.call("process_i64",
  types.i64(9007199254740992n)
);
```

### 3. Memory Growth Assumptions

```typescript
// ❌ Assuming memory.grow() always succeeds
await instance.memory.grow(1000);  // May fail if exceeds max

// ✅ Handle growth failure
try {
  const oldSize = await instance.memory.grow(pagesNeeded);
  console.log(`Grew from ${oldSize} pages`);
} catch (error) {
  console.error("Failed to grow memory:", error);
  // Use existing memory or fail gracefully
}
```

### 4. Preopen Path Security

```typescript
// ❌ DANGEROUS: Granting access to sensitive directories
await instantiate(moduleId, {
  preopens: { "/": "/" }  // Root access!
});

// ✅ SAFE: Grant minimal required access
await instantiate(moduleId, {
  preopens: {
    "/data": "./app-data",      // Application data only
    "/tmp": "./temp-storage"     // Temporary files
  }
});
```

## See Also

- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WASI Specification](https://github.com/WebAssembly/WASI)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WebAssembly.org](https://webassembly.org/)
- [ext_fs](../ext_fs/) - File system operations extension
- [ext_process](../ext_process/) - Process spawning extension
- [Forge Documentation](../../site/) - Full framework documentation

## License

Part of the Forge project. See the repository root for license information.
