---
title: "ext_wasm"
description: WebAssembly runtime extension providing the runtime:wasm module.
slug: crates/ext-wasm
---

The `ext_wasm` crate provides WebAssembly module loading, instantiation, and execution for Forge applications through the `runtime:wasm` module.

## Overview

ext_wasm handles:

- **Module compilation** - Compile WASM from bytes or files
- **Module instantiation** - Create instances with imports
- **Function calls** - Call exported WASM functions
- **WASI support** - System interface for WASM modules
- **Memory access** - Read/write WASM memory
- **Capability-based security** - Module loading permissions

## Module: `runtime:wasm`

```typescript
import {
  compileFile,
  compile,
  instantiate
} from "runtime:wasm";

const module = await compileFile("./module.wasm");
const instance = await instantiate(module, {});
const result = instance.call("add", [1, 2]);
```

## Key Types

### Error Types

```rust
enum WasmErrorCode {
    CompileError = 5000,
    InstantiateError = 5001,
    RuntimeError = 5002,
    MemoryError = 5003,
    FunctionNotFound = 5004,
    TypeMismatch = 5005,
    FuelExhausted = 5006,
}

struct WasmError {
    code: WasmErrorCode,
    message: String,
}
```

### Module Types

```rust
struct WasmModule {
    id: u32,
    exports: Vec<ExportInfo>,
}

struct WasmInstance {
    id: u32,
    module_id: u32,
    exports: Vec<ExportInfo>,
}

struct ExportInfo {
    name: String,
    kind: ExportKind,
}

enum ExportKind {
    Function,
    Memory,
    Global,
    Table,
}
```

### Value Types

```rust
enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}
```

### WASI Configuration

```rust
struct WasiConfig {
    args: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    preopens: Option<HashMap<String, String>>,
    stdin: Option<StdioConfig>,
    stdout: Option<StdioConfig>,
    stderr: Option<StdioConfig>,
}
```

### State Types

```rust
struct WasmState {
    engine: Engine,
    modules: HashMap<u32, Module>,
    instances: HashMap<u32, Instance>,
    next_module_id: u32,
    next_instance_id: u32,
}

struct WasmCapabilities {
    allow_file_compile: bool,
    allow_wasi: bool,
    max_memory_bytes: Option<u64>,
    max_fuel: Option<u64>,
}
```

## Operations

| Op | TypeScript | Description |
|----|------------|-------------|
| `op_wasm_compile` | `compile(bytes)` | Compile WASM from bytes |
| `op_wasm_compile_file` | `compileFile(path)` | Compile WASM from file |
| `op_wasm_instantiate` | `instantiate(module, imports)` | Create instance |
| `op_wasm_call` | `instance.call(name, args)` | Call exported function |
| `op_wasm_get_export` | `instance.getExport(name)` | Get export info |
| `op_wasm_read_memory` | `instance.readMemory(offset, len)` | Read memory |
| `op_wasm_write_memory` | `instance.writeMemory(offset, data)` | Write memory |
| `op_wasm_drop_module` | (internal) | Free module |
| `op_wasm_drop_instance` | (internal) | Free instance |

## Fuel-Based Limits

WASM execution can be limited using fuel:

```typescript
const instance = await instantiate(module, {}, {
  fuel: 1000000  // Limit execution steps
});
```

When fuel is exhausted, a `FuelExhausted` error is thrown.

## File Structure

```text
crates/ext_wasm/
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
pub enum ExportKind {
    Function,
    Memory,
    Global,
    Table,
}

#[weld_struct]
#[derive(Debug, Serialize)]
pub struct WasmModule {
    pub id: u32,
    pub exports: Vec<ExportInfo>,
}

#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_wasm_compile(
    state: Rc<RefCell<OpState>>,
    #[buffer] bytes: &[u8],
) -> Result<WasmModule, WasmError> {
    // implementation
}
```

## Build Configuration

```rust
// build.rs
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_wasm", "runtime:wasm")
        .ts_path("ts/init.ts")
        .ops(&["op_wasm_compile", "op_wasm_instantiate", "op_wasm_call", /* ... */])
        .generate_sdk_module("sdk")
        .use_inventory_types()
        .build()
        .expect("Failed to build runtime_wasm extension");
}
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_core` | Op definitions |
| `wasmtime` | WebAssembly runtime |
| `wasmtime-wasi` | WASI implementation |
| `tokio` | Async runtime |
| `serde` | Serialization |
| `tracing` | Logging |
| `forge-weld` | Build-time code generation |
| `forge-weld-macro` | `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]` macros |
| `linkme` | Compile-time symbol collection |

## Related

- [runtime:wasm API](/docs/api/runtime-wasm) - TypeScript API documentation
- [forge-weld](/docs/crates/forge-weld) - Code generation library
