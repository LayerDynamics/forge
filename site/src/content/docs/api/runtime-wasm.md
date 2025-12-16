---
title: "runtime:wasm"
description: WebAssembly module loading, instantiation, and execution with WASI support.
slug: api/runtime-wasm
---

The `runtime:wasm` module provides WebAssembly support including module compilation, instantiation, function calls, memory access, and WASI integration.

> **Implementation**: TypeScript types are auto-generated from Rust via [forge-weld](/docs/crates/forge-weld). See [ext_wasm](/docs/crates/ext-wasm) for implementation details.

## Capabilities

WebAssembly operations must be declared in `manifest.app.toml`:

```toml
[capabilities.wasm]
load = true        # Allow loading/compiling WASM modules
execute = true     # Allow instantiation and function calls
```

---

## Compiling Modules

### compile(bytes)

Compile WASM bytes into a module:

```typescript
import { compile, instantiate } from "runtime:wasm";
import { readBytes } from "runtime:fs";

const bytes = await readBytes("./module.wasm");
const moduleId = await compile(bytes);
const instance = await instantiate(moduleId);
```

### compileFile(path)

Compile WASM directly from a file path:

```typescript
import { compileFile, instantiate } from "runtime:wasm";

const moduleId = await compileFile("./math.wasm");
const instance = await instantiate(moduleId);
const [result] = await instance.call("add", 2, 3);
console.log(result); // 5
```

### dropModule(moduleId)

Free a compiled module's resources:

```typescript
import { compileFile, dropModule } from "runtime:wasm";

const moduleId = await compileFile("./module.wasm");
// Use the module...
await dropModule(moduleId);
```

---

## Instantiating Modules

### instantiate(moduleId, wasiConfig?)

Create a runnable instance from a compiled module:

```typescript
import { compileFile, instantiate } from "runtime:wasm";

// Basic instantiation
const moduleId = await compileFile("./pure.wasm");
const instance = await instantiate(moduleId);
```

With WASI configuration for modules that need system access:

```typescript
const wasiModuleId = await compileFile("./app.wasm");
const wasiInstance = await instantiate(wasiModuleId, {
  preopens: { "/data": "./app-data" },
  env: { "HOME": "/data" },
  args: ["app", "--verbose"],
  inheritStdout: true,
});
await wasiInstance.call("_start");
```

### WasiConfig Options

| Option | Type | Description |
|--------|------|-------------|
| `preopens` | `Record<string, string>` | Guest path to host path mappings |
| `env` | `Record<string, string>` | Environment variables |
| `args` | `string[]` | Command-line arguments |
| `inheritStdin` | `boolean` | Inherit stdin from host |
| `inheritStdout` | `boolean` | Inherit stdout from host |
| `inheritStderr` | `boolean` | Inherit stderr from host |

---

## Calling Functions

### instance.call(name, ...args)

Call an exported WASM function:

```typescript
import { compileFile, instantiate } from "runtime:wasm";

const moduleId = await compileFile("./calculator.wasm");
const instance = await instantiate(moduleId);

// Simple call with auto-typed arguments
const [sum] = await instance.call("add", 10, 20);
console.log(sum); // 30

// Call with multiple return values
const [quotient, remainder] = await instance.call("divmod", 17, 5);
console.log(quotient, remainder); // 3 2
```

### Explicit Type Annotations

Use the `types` helper for explicit value types:

```typescript
import { compileFile, instantiate, types } from "runtime:wasm";

const instance = await instantiate(await compileFile("./math.wasm"));

// Explicit i64 for large integers
const [result] = await instance.call(
  "large_add",
  types.i64(9007199254740992n),
  types.i64(1n)
);

// Explicit float types
const [area] = await instance.call(
  "circle_area",
  types.f64(3.14159)
);
```

### Type Helpers

| Helper | Creates | Example |
|--------|---------|---------|
| `types.i32(n)` | 32-bit integer | `types.i32(42)` |
| `types.i64(n)` | 64-bit integer | `types.i64(9007199254740992n)` |
| `types.f32(n)` | 32-bit float | `types.f32(3.14)` |
| `types.f64(n)` | 64-bit float | `types.f64(3.141592653589793)` |

---

## Inspecting Exports

### instance.getExports()

List all exports from an instance:

```typescript
import { compileFile, instantiate } from "runtime:wasm";

const instance = await instantiate(await compileFile("./module.wasm"));
const exports = await instance.getExports();

for (const exp of exports) {
  console.log(`${exp.kind}: ${exp.name}`);
  if (exp.kind === "function") {
    console.log(`  params: ${exp.params?.join(", ")}`);
    console.log(`  results: ${exp.results?.join(", ")}`);
  }
}
```

### ExportInfo Structure

```typescript
interface ExportInfo {
  name: string;
  kind: "function" | "memory" | "table" | "global";
  params?: WasmValueType[];   // For functions
  results?: WasmValueType[];  // For functions
}
```

---

## Memory Access

### instance.memory.read(offset, length)

Read bytes from WASM linear memory:

```typescript
const instance = await instantiate(await compileFile("./strings.wasm"));

// Get a string from WASM memory
const [ptr, len] = await instance.call("get_greeting");
const bytes = await instance.memory.read(ptr, len);
const greeting = new TextDecoder().decode(bytes);
console.log(greeting); // "Hello, World!"
```

### instance.memory.write(offset, data)

Write bytes to WASM linear memory:

```typescript
const instance = await instantiate(await compileFile("./strings.wasm"));

// Allocate memory in WASM
const [ptr] = await instance.call("allocate", 13);

// Write string to WASM memory
const data = new TextEncoder().encode("Hello, World!");
await instance.memory.write(ptr, data);

// Call function that uses the string
await instance.call("print_string", ptr, data.length);
```

### instance.memory.size()

Get current memory size in pages (1 page = 64KB):

```typescript
const pages = await instance.memory.size();
console.log(`Memory: ${pages * 64} KB`);
```

### instance.memory.grow(pages)

Grow memory by additional pages:

```typescript
const prevSize = await instance.memory.grow(1);
console.log(`Grew from ${prevSize} to ${prevSize + 1} pages`);
```

---

## Instance Lifecycle

### instance.drop()

Release instance resources when done:

```typescript
import { compileFile, instantiate, dropModule } from "runtime:wasm";

const moduleId = await compileFile("./module.wasm");
const instance = await instantiate(moduleId);

try {
  await instance.call("process");
} finally {
  await instance.drop();
  await dropModule(moduleId);
}
```

---

## Complete Example

A complete example showing compilation, instantiation, function calls, and memory access:

```typescript
import { compileFile, instantiate, dropModule, types } from "runtime:wasm";

async function runWasmApp() {
  // Compile the module
  const moduleId = await compileFile("./image_processor.wasm");

  // Instantiate with WASI for file access
  const instance = await instantiate(moduleId, {
    preopens: { "/images": "./data/images" },
    inheritStdout: true,
  });

  try {
    // Check available exports
    const exports = await instance.getExports();
    console.log("Exports:", exports.map(e => e.name));

    // Allocate buffer for input
    const imageSize = 1024 * 1024; // 1MB
    const [inputPtr] = await instance.call("allocate", imageSize);

    // Load image data into WASM memory
    const imageData = new Uint8Array(imageSize); // ... load from somewhere
    await instance.memory.write(inputPtr, imageData);

    // Process the image
    const [outputPtr, outputLen] = await instance.call(
      "process_image",
      inputPtr,
      imageSize,
      types.f32(0.5) // brightness adjustment
    );

    // Read result
    const result = await instance.memory.read(outputPtr, outputLen);
    console.log(`Processed ${result.length} bytes`);

    // Free allocations
    await instance.call("deallocate", inputPtr);
    await instance.call("deallocate", outputPtr);

  } finally {
    // Clean up
    await instance.drop();
    await dropModule(moduleId);
  }
}

runWasmApp();
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 5000 | CompileError | Failed to compile WASM module |
| 5001 | InstantiateError | Failed to instantiate module |
| 5002 | CallError | Function call failed |
| 5003 | ExportNotFound | Export not found in module |
| 5004 | InvalidModuleHandle | Module ID not found |
| 5005 | InvalidInstanceHandle | Instance ID not found |
| 5006 | MemoryError | Memory access error |
| 5007 | TypeError | Type mismatch in function call |
| 5008 | IoError | File loading error |
| 5009 | PermissionDenied | Capability not granted |
| 5010 | WasiError | WASI configuration error |
| 5011 | FuelExhausted | Execution limit exceeded |

---

## Type Reference

### WasmValueType

```typescript
type WasmValueType = "i32" | "i64" | "f32" | "f64";
```

### WasmValue

```typescript
interface WasmValue {
  type: WasmValueType;
  value: number;
}
```

### WasmInstance

```typescript
interface WasmInstance {
  readonly id: string;
  readonly moduleId: string;
  call(name: string, ...args: (number | bigint | WasmValue)[]): Promise<number[]>;
  getExports(): Promise<ExportInfo[]>;
  memory: WasmMemory;
  drop(): Promise<void>;
}
```

### WasmMemory

```typescript
interface WasmMemory {
  read(offset: number, length: number): Promise<Uint8Array>;
  write(offset: number, data: Uint8Array): Promise<void>;
  size(): Promise<number>;
  grow(pages: number): Promise<number>;
}
```
