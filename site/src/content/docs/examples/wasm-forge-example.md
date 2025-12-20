---
title: "wasm-forge-example"
description: WebAssembly integration with compile, instantiate, and memory operations
slug: examples/wasm-forge-example
---

A demonstration of WebAssembly integration in Forge applications.

## Overview

This example shows:
- Compiling WASM modules with `runtime:wasm`
- Instantiating and calling WASM functions
- Memory read/write operations
- Module lifecycle management

## Features

- Compile WASM from bytes
- Call exported functions with parameters
- Direct memory manipulation
- Multiple module support (add, multiply, memory ops)

## Running

```bash
forge dev examples/wasm-forge-example
```

## Capabilities

```toml
[capabilities.channels]
allowed = ["*"]

[capabilities.wasm]
allowed = true
```

## Key Patterns

### Compiling WASM

```typescript
import { compile, instantiate } from "runtime:wasm";

// Compile from bytes
const moduleId = await compile(wasmBytes);

// Instantiate the module
const instance = await instantiate(moduleId);
```

### Calling Functions

```typescript
// Get exports
const exports = await instance.getExports();
// Returns: [{ name: "add", kind: "function" }, ...]

// Call a function
const result = await instance.call("add", 7, 5);
console.log(result[0]); // 12
```

### Memory Operations

```typescript
// Write to WASM memory
const data = new Uint8Array([1, 2, 3, 4]);
await instance.memory.write(0, data);

// Read from WASM memory
const bytes = await instance.memory.read(0, 4);

// Get memory size (in 64KB pages)
const pages = await instance.memory.size();
```

### Module Cleanup

```typescript
// Release resources when done
await instance.drop();
```

## Example WASM Modules

### Simple Add Function

```wat
(module
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add))
```

### Memory Operations

```wat
(module
  (memory (export "memory") 1)
  (func (export "get_value") (param i32) (result i32)
    local.get 0
    i32.load)
  (func (export "set_value") (param i32 i32)
    local.get 0
    local.get 1
    i32.store))
```

## Architecture

```text
Deno (runtime:wasm)          WebAssembly Runtime
       |                            |
       |-- compile(bytes) -------> | Parse & validate
       | <-- moduleId ------------ |
       |                            |
       |-- instantiate(id) ------> | Create instance
       | <-- instance ------------ |
       |                            |
       |-- call("add", 7, 5) ----> | Execute
       | <-- [12] ---------------- |
       |                            |
       |-- memory.write() -------> | Direct memory access
       |                            |
```

## Extending

Load external WASM files:

```typescript
import { readBytes } from "runtime:fs";

const wasmBytes = await readBytes("./my-module.wasm");
const moduleId = await compile(wasmBytes);
const instance = await instantiate(moduleId);
```
