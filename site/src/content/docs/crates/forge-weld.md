---
title: "forge-weld"
description: Code generation and binding utilities for Rust↔TypeScript integration.
---

The `forge-weld` crate provides the "glue" between Rust deno_core ops and TypeScript. It generates TypeScript type definitions, init modules, and handles the build process for Forge extensions.

## Overview

forge-weld handles:

- **Type generation** - Generate `.d.ts` files from Rust ops and structs
- **Module building** - Transpile TypeScript init modules to JavaScript
- **Extension scaffolding** - Configure extension builds via `ExtensionBuilder`
- **Symbol registry** - Collect ops and types at compile time via `linkme`

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                      forge-weld                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────┐    ┌────────────┐    ┌────────────────────┐ │
│  │     IR     │───►│   Codegen  │───►│  TypeScript (.d.ts)│ │
│  │ (types,ops)│    │            │    │  JavaScript (init) │ │
│  └────────────┘    └────────────┘    └────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                   ExtensionBuilder                      │ │
│  │    (build.rs helper for extension crates)              │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Usage

In an extension's `build.rs`:

```rust
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_fs", "host:fs")
        .ts_path("ts/init.ts")
        .ops(&["op_fs_read_text", "op_fs_write_text"])
        .generate_sdk_types("sdk")
        .dts_generator(generate_host_fs_types)
        .build()
        .expect("Failed to build extension");
}
```

## Modules

### IR (Intermediate Representation)

Types for representing ops, structs, and modules:

```rust
// Op symbol with metadata
struct OpSymbol {
    name: &'static str,
    is_async: bool,
    params: Vec<OpParam>,
    return_type: WeldType,
}

// Struct definition
struct WeldStruct {
    name: &'static str,
    ts_name: Option<&'static str>,
    fields: Vec<StructField>,
}

// Module containing ops and types
struct WeldModule {
    name: String,
    specifier: String,
    ops: Vec<OpSymbol>,
    structs: Vec<WeldStruct>,
}
```

### Codegen

Code generators for TypeScript and Rust:

```rust
// Generate TypeScript declarations
let generator = TypeScriptGenerator::new();
let dts = generator.generate_module(&module);

// Generate .d.ts file
let dts_builder = DtsBuilder::new("host:fs");
dts_builder.generate_to_file("sdk/generated/host.fs.d.ts")?;
```

### Build

Build script utilities:

```rust
// Transpile TypeScript to JavaScript
let js = transpile_ts(ts_source)?;

// Transpile a file
let js = transpile_file(Path::new("ts/init.ts"))?;
```

## Key Types

### WeldType

Represents TypeScript types:

```rust
enum WeldType {
    Primitive(WeldPrimitive),
    Array(Box<WeldType>),
    Optional(Box<WeldType>),
    Promise(Box<WeldType>),
    Object(Vec<(String, WeldType)>),
    Reference(String),
    Union(Vec<WeldType>),
}

enum WeldPrimitive {
    String,
    Number,
    Boolean,
    Void,
    Unknown,
    BigInt,
}
```

### ExtensionBuilder

Fluent API for building extensions:

```rust
ExtensionBuilder::new("host_fs", "host:fs")
    .ts_path("ts/init.ts")           // TypeScript source
    .ops(&["op_fs_read_text"])       // Op names
    .generate_sdk_types("sdk")       // SDK output dir
    .dts_generator(fn)               // Custom .d.ts generator
    .build()?;
```

## Symbol Registry

forge-weld uses `linkme` for compile-time symbol collection:

```rust
// Distributed slices for collecting metadata
#[distributed_slice]
pub static WELD_OPS: [fn() -> OpSymbol];

#[distributed_slice]
pub static WELD_STRUCTS: [fn() -> WeldStruct];

#[distributed_slice]
pub static WELD_ENUMS: [fn() -> WeldEnum];
```

## File Structure

```text
crates/forge-weld/
├── src/
│   ├── lib.rs      # Main entry, re-exports
│   ├── ir.rs       # Intermediate representation types
│   ├── codegen.rs  # Code generators
│   └── build.rs    # ExtensionBuilder and utilities
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `deno_ast` | TypeScript parsing and transpilation |
| `syn`, `quote`, `proc-macro2` | Rust AST manipulation |
| `linkme` | Compile-time symbol collection |
| `serde`, `serde_json` | Type serialization |
| `thiserror` | Error types |

## Related

- [forge-weld-macro](/docs/crates/forge-weld-macro) - Procedural macros (`#[weld_op]`, `#[weld_struct]`)
- [Architecture](/docs/architecture) - Build system documentation
