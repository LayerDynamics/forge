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

The IR module is organized into submodules for representing ops, structs, enums, and modules:

```text
ir/
├── mod.rs       # Module exports and re-exports
├── types.rs     # WeldType, WeldPrimitive definitions
├── symbol.rs    # OpSymbol, OpParam, StructField, EnumVariant
├── inventory.rs # WELD_OPS, WELD_STRUCTS, WELD_ENUMS distributed slices
└── module.rs    # WeldModule builder pattern
```

**Core types:**

```rust
// Op symbol with metadata
struct OpSymbol {
    rust_name: String,
    ts_name: String,
    is_async: bool,
    params: Vec<OpParam>,
    return_type: WeldType,
    doc: Option<String>,
    op2_attrs: Op2Attrs,
    module: Option<String>,
}

// Op parameter
struct OpParam {
    rust_name: String,
    ts_name: String,
    ty: WeldType,
    attr: ParamAttr,
    optional: bool,
    doc: Option<String>,
}

// Struct definition
struct WeldStruct {
    rust_name: String,
    ts_name: String,
    fields: Vec<StructField>,
    doc: Option<String>,
    type_params: Vec<String>,
}

// Enum definition
struct WeldEnum {
    rust_name: String,
    ts_name: String,
    variants: Vec<EnumVariant>,
    doc: Option<String>,
}

// Module containing ops and types
struct WeldModule {
    name: String,
    specifier: String,
    esm_entry_point: String,
    ops: Vec<OpSymbol>,
    structs: Vec<WeldStruct>,
    enums: Vec<WeldEnum>,
}
```

### Codegen

The codegen module contains generators for TypeScript and Rust code:

```text
codegen/
├── mod.rs        # Module exports
├── typescript.rs # TypeScript type conversion (WeldType → TS)
├── dts.rs        # .d.ts file generator
└── extension.rs  # deno_core::extension! macro generator
```

**TypeScript generation:**

```rust
// Generate TypeScript declarations
let generator = TypeScriptGenerator::new();
let dts = generator.generate_module(&module);

// Generate .d.ts file
let dts_builder = DtsBuilder::new("host:fs");
dts_builder.generate_to_file("sdk/generated/host.fs.d.ts")?;
```

**Extension macro generation:**

```rust
// Generate deno_core::extension! invocation
let gen = ExtensionGenerator::new(&module);
let output = gen.generate(js_source);
// Produces:
// deno_core::extension!(
//     host_fs,
//     ops = [op_fs_read_text, op_fs_write_text, ...],
//     esm_entry_point = "ext:host_fs/init.js",
//     esm = ["ext:host_fs/init.js" = { source = "..." }]
// );
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

Represents Rust types for TypeScript conversion:

```rust
enum WeldType {
    // Primitives
    Primitive(WeldPrimitive),

    // Containers
    Option(Box<WeldType>),
    Vec(Box<WeldType>),
    Result { ok: Box<WeldType>, err: Box<WeldType> },
    HashMap { key: Box<WeldType>, value: Box<WeldType> },
    BTreeMap { key: Box<WeldType>, value: Box<WeldType> },
    HashSet(Box<WeldType>),
    BTreeSet(Box<WeldType>),

    // Smart pointers (transparent in TypeScript)
    Box(Box<WeldType>),
    Arc(Box<WeldType>),
    Rc(Box<WeldType>),
    RefCell(Box<WeldType>),
    Mutex(Box<WeldType>),
    RwLock(Box<WeldType>),

    // References
    Reference { inner: Box<WeldType>, mutable: bool },
    Pointer { inner: Box<WeldType>, mutable: bool },

    // Compound types
    Tuple(Vec<WeldType>),
    Struct(String),       // Named struct reference
    Enum(String),         // Named enum reference

    // Special
    JsonValue,            // serde_json::Value → unknown
    OpState,              // Internal Deno type (filtered out)
    Never,                // ! type → never
}

enum WeldPrimitive {
    U8, U16, U32, U64, Usize,
    I8, I16, I32, I64, Isize,
    F32, F64,
    Bool, String, Str, Char, Unit,
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
│   ├── lib.rs              # Main entry, re-exports
│   ├── ir/
│   │   ├── mod.rs          # IR module exports
│   │   ├── types.rs        # WeldType, WeldPrimitive
│   │   ├── symbol.rs       # OpSymbol, OpParam, StructField
│   │   ├── inventory.rs    # Distributed slices for compile-time collection
│   │   └── module.rs       # WeldModule builder
│   ├── codegen/
│   │   ├── mod.rs          # Codegen module exports
│   │   ├── typescript.rs   # WeldType → TypeScript conversion
│   │   ├── dts.rs          # .d.ts file generation
│   │   └── extension.rs    # deno_core::extension! macro generation
│   └── build/
│       ├── mod.rs          # Build utilities exports
│       ├── extension.rs    # ExtensionBuilder
│       └── transpile.rs    # TypeScript → JavaScript via deno_ast
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
