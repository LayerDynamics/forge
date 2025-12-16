# runtime:wasm API Reference

The `runtime:wasm` module provides WebAssembly runtime capabilities with WASI support and capability-based security.

## Capabilities

WASM operations must be declared in `manifest.app.toml`:

```toml
[capabilities.wasm]
load = ["./wasm/*.wasm", "~/.myapp/plugins/*.wasm"]
preopens = ["./data", "~/.myapp/storage"]
```

- `load` - Paths allowed for loading WASM modules
- `preopens` - Directories that can be exposed to WASI modules

---

## Module Compilation

### compile(bytes)

Compile WASM binary to a module:

```typescript
import { compile } from "runtime:wasm";

const response = await fetch("./module.wasm");
const bytes = new Uint8Array(await response.arrayBuffer());
const moduleId = await compile(bytes);
```

### compileFile(path)

Compile WASM from a file path:

```typescript
import { compileFile } from "runtime:wasm";

const moduleId = await compileFile("./calculator.wasm");
```

### dropModule(moduleId)

Release a compiled module:

```typescript
import { dropModule } from "runtime:wasm";

await dropModule(moduleId);
```

---

## Instance Creation

### instantiate(moduleId, wasiConfig?)

Create an instance from a compiled module:

```typescript
import { compileFile, instantiate } from "runtime:wasm";

const moduleId = await compileFile("./module.wasm");
const instance = await instantiate(moduleId);
```

With WASI configuration:

```typescript
const instance = await instantiate(moduleId, {
  preopens: {
    "/data": "./app-data",     // Guest path -> Host path
    "/tmp": "/tmp/myapp"
  },
  env: {
    "APP_MODE": "production",
    "DEBUG": "0"
  },
  args: ["--config", "app.toml"],
  inheritStdout: true,
  inheritStderr: true
});
```

#### WasiConfig Options

| Option | Type | Description |
|--------|------|-------------|
| `preopens` | `Record<string, string>` | Guest path to host path mappings |
| `env` | `Record<string, string>` | Environment variables |
| `args` | `string[]` | Command-line arguments |
| `inheritStdin` | `boolean` | Inherit stdin from host |
| `inheritStdout` | `boolean` | Inherit stdout from host |
| `inheritStderr` | `boolean` | Inherit stderr from host |

---

## Function Calls

### instance.call(name, ...args)

Call an exported function:

```typescript
// Simple call with auto type inference
const [result] = await instance.call("add", 5, 3);
console.log(result); // 8

// With explicit types
import { types } from "runtime:wasm";

const [result] = await instance.call("compute",
  types.i32(100),
  types.f64(3.14)
);
```

### instance.getExports()

Get list of module exports:

```typescript
const exports = await instance.getExports();
// [
//   { name: "add", kind: "function", params: ["i32", "i32"], results: ["i32"] },
//   { name: "memory", kind: "memory" },
//   { name: "factorial", kind: "function", params: ["i64"], results: ["i64"] }
// ]
```

### instance.drop()

Release the instance:

```typescript
await instance.drop();
```

---

## Memory Operations

Access linear memory through `instance.memory`:

### instance.memory.read(offset, length)

Read bytes from memory:

```typescript
const data = await instance.memory.read(0, 256);
// Returns: Uint8Array

// Read a null-terminated string
function readString(instance, ptr) {
  const bytes = await instance.memory.read(ptr, 1024);
  const end = bytes.indexOf(0);
  return new TextDecoder().decode(bytes.subarray(0, end));
}
```

### instance.memory.write(offset, data)

Write bytes to memory:

```typescript
const encoder = new TextEncoder();
await instance.memory.write(0, encoder.encode("Hello\0"));
```

### instance.memory.size()

Get memory size in pages (64KB each):

```typescript
const pages = await instance.memory.size();
console.log(`Memory: ${pages * 64}KB`);
```

### instance.memory.grow(pages)

Grow memory by specified pages:

```typescript
const previousSize = await instance.memory.grow(1);
console.log(`Grew from ${previousSize} to ${previousSize + 1} pages`);
```

---

## Type Helpers

The `types` object provides explicit type constructors:

```typescript
import { types } from "runtime:wasm";

types.i32(42)       // 32-bit signed integer
types.i64(9007199254740993n)  // 64-bit signed integer
types.f32(3.14)     // 32-bit float
types.f64(2.718281828)  // 64-bit float
```

---

## Low-Level API

For advanced use cases, low-level functions are available:

```typescript
import { call, getExports, memory } from "runtime:wasm";

// Direct function call with explicit types
const results = await call(instanceId, "compute", [
  { type: "i32", value: 10 },
  { type: "f64", value: 3.14 }
]);

// Direct memory access
const data = await memory.read(instanceId, 0, 100);
await memory.write(instanceId, 0, new Uint8Array([1, 2, 3]));
const size = await memory.size(instanceId);
await memory.grow(instanceId, 2);
```

---

## Complete Example

```typescript
import { compileFile, instantiate, types } from "runtime:wasm";

// Load and instantiate a WASM module
const moduleId = await compileFile("./calculator.wasm");
const calc = await instantiate(moduleId);

// Check available functions
const exports = await calc.getExports();
console.log("Available functions:", exports.filter(e => e.kind === "function"));

// Call functions
const [sum] = await calc.call("add", 10, 20);
const [product] = await calc.call("multiply", types.i32(5), types.i32(7));

console.log(`10 + 20 = ${sum}`);
console.log(`5 * 7 = ${product}`);

// Clean up
await calc.drop();
await dropModule(moduleId);
```

---

## WASI Example

```typescript
import { compileFile, instantiate } from "runtime:wasm";

// Compile a WASI-compatible module
const moduleId = await compileFile("./cli-tool.wasm");

// Instantiate with WASI configuration
const instance = await instantiate(moduleId, {
  preopens: {
    "/": "./sandbox"  // Mount sandbox directory as root
  },
  env: {
    "HOME": "/home/user",
    "PATH": "/bin"
  },
  args: ["tool", "--verbose", "input.txt"],
  inheritStdout: true,
  inheritStderr: true
});

// Call the WASI _start entry point
await instance.call("_start");

await instance.drop();
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 5000 | CompileError | Failed to compile WASM module |
| 5001 | InstantiateError | Failed to instantiate module |
| 5002 | CallError | Function call failed |
| 5003 | ExportNotFound | Export not found in module |
| 5004 | InvalidModuleHandle | Invalid module ID |
| 5005 | InvalidInstanceHandle | Invalid instance ID |
| 5006 | MemoryError | Memory access error |
| 5007 | TypeError | Type mismatch in function call |
| 5008 | IoError | File loading error |
| 5009 | PermissionDenied | Capability check failed |
| 5010 | WasiError | WASI configuration error |
| 5011 | FuelExhausted | Execution limit reached |
