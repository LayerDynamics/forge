---
title: "forge-weld"
description: Code generation and binding utilities for Rust↔TypeScript integration.
slug: crates/forge-weld
---

The `forge-weld` crate provides the "glue" between Rust deno_core ops and TypeScript. It generates TypeScript type definitions, init modules, and handles the build process for Forge extensions.

## Overview

forge-weld handles:

- **Type generation** - Generate `.d.ts` files from Rust ops and structs
- **SDK modules** - Generate runtime `runtime.*.ts` wrappers for `Deno.core.ops`
- **Module building** - Transpile TypeScript init modules to JavaScript
- **Extension scaffolding** - Configure extension builds via `ExtensionBuilder`
- **Symbol registry** - Collect ops and types at compile time via `linkme`

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                      forge-weld                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────┐    ┌────────────┐    ┌────────────────────┐ │
│  │     IR     │───►│   Codegen  │───►│  TypeScript (.d.ts)│ │
│  │ (types,ops)│    │            │    │  JavaScript (init) │ │
│  └────────────┘    └────────────┘    └────────────────────┘ │
│                                                             │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                   ExtensionBuilder                     │ │
│  │    (build.rs helper for extension crates)              │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Usage

### Step 1: Annotate Rust Code

In your extension's `src/lib.rs`, use the forge-weld-macro attributes:

```rust
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use deno_core::op2;

// Annotate structs that should appear in TypeScript
#[weld_struct]
#[derive(Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

// Annotate enums for TypeScript union types
#[weld_enum]
#[derive(Serialize)]
pub enum PathType {
    File,
    Directory,
    Symlink,
}

// Annotate ops - #[weld_op] must come BEFORE #[op2]
#[weld_op(async)]
#[op2(async)]
#[serde]
pub async fn op_fs_read_text(#[string] path: String) -> Result<String, FsError> {
    // implementation
}
```

### Step 2: Configure build.rs

```rust
use forge_weld::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("runtime_fs", "runtime:fs")
        .ts_path("ts/init.ts")
        .ops(&["op_fs_read_text", "op_fs_write_text", "op_fs_stat"])
        .generate_sdk_module("sdk")   // Generates sdk/runtime.fs.ts
        .use_inventory_types()         // Reads #[weld_*] annotations
        .build()
        .expect("Failed to build extension");
}
```

### Step 3: Include Generated Code

In your `src/lib.rs`, include the generated extension registration:

```rust
// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn fs_extension() -> Extension {
    runtime_fs::ext()  // Name matches ExtensionBuilder::new() first arg
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
// Generate TypeScript init/runtime module
let generator = TypeScriptGenerator::new(&module);
let ts_source = generator.generate();

// Generate .d.ts file
let dts = DtsGenerator::new(&module).generate();
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
//     esm_entry_point = "ext:runtime_fs/init.js",
//     esm = ["ext:runtime_fs/init.js" = { source = "..." }]
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
ExtensionBuilder::new("runtime_fs", "runtime:fs")
    .ts_path("ts/init.ts")           // TypeScript shim source
    .ops(&["op_fs_read_text"])       // Op names to register
    .generate_sdk_module("sdk")      // Generate sdk/runtime.fs.ts
    .use_inventory_types()           // Use #[weld_*] macro annotations
    .build()?;
```

**Key methods:**

| Method | Purpose |
|--------|---------|
| `new(name, specifier)` | Create builder with extension name and module specifier |
| `ts_path(path)` | Path to TypeScript shim (ts/init.ts) |
| `ops(&[...])` | List of op function names to register |
| `generate_sdk_module(dir)` | Generate TypeScript function-based SDK to directory |
| `generate_sdk_class(path)` | Generate TypeScript class-based SDK to directory |
| `generate_schema()` | Enable JSON Schema generation with defaults |
| `generate_schema_with(config)` | Enable schema generation with custom SchemaConfig |
| `schema_output_dir(dir)` | Set schema output directory (default: sdk/schemas) |
| `schema_formats(formats)` | Set schema formats to generate (JsonSchema, OpenAPI, TypeScriptSdk) |
| `use_inventory_types()` | Read types from `#[weld_*]` macro annotations |
| `build()` | Execute the build, generating all output files |

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

## Schema Generation

forge-weld can automatically generate JSON Schema and OpenAPI specifications from your extension's types and operations.

### Configuration

```rust
use forge_weld::{ExtensionBuilder, SchemaConfig, SchemaFormat};

ExtensionBuilder::new("runtime_fs", "runtime:fs")
    .ts_path("ts/init.ts")
    .ops(&["op_fs_read_text", "op_fs_write_text"])
    .generate_sdk_module("sdk")         // Function-based SDK
    .generate_sdk_class("sdk/classes")  // Class-based SDK (optional)
    .generate_schema()                   // JSON Schema with defaults
    .schema_formats(&[
        SchemaFormat::JsonSchema,        // JSON Schema 2020-12
        SchemaFormat::OpenApi,           // OpenAPI 3.1.0
    ])
    .use_inventory_types()
    .build()?;
```

### SchemaConfig

Controls schema generation behavior:

```rust
pub struct SchemaConfig {
    /// Output directory (default: "sdk/schemas")
    pub output_dir: PathBuf,

    /// Formats to generate
    pub formats: Vec<SchemaFormat>,

    /// Include example values (default: true)
    pub include_examples: bool,

    /// Add version suffix to filenames (default: false)
    pub versioned: bool,

    /// Base URL for schema $id fields (default: None)
    pub schema_base_url: Option<String>,

    /// Fail build on schema errors (default: true)
    pub fail_on_error: bool,
}
```

### SchemaFormat

```rust
pub enum SchemaFormat {
    JsonSchema,      // JSON Schema Draft 2020-12
    OpenApi,         // OpenAPI 3.1.0
    TypeScriptSdk,   // Class-based TypeScript SDK
}
```

### Generated Output

Running the build generates:

```
sdk/
├── runtime.fs.ts                    # Function-based SDK
├── classes/
│   └── FsClient.ts                  # Class-based SDK (if enabled)
└── schemas/
    ├── runtime.fs.schema.json       # JSON Schema 2020-12
    └── runtime.fs.openapi.json      # OpenAPI 3.1.0
```

### Class-Based SDK

The class-based SDK provides an object-oriented alternative to the function-based SDK:

```typescript
// Function-based SDK (default)
import { readTextFile, writeTextFile } from "runtime:fs";
await readTextFile("./config.json");

// Class-based SDK (opt-in)
import { fs } from "runtime:fs/classes";
await fs.readTextFile("./config.json");

// Or instantiate custom client
import { FsClient } from "runtime:fs/classes";
const fsClient = new FsClient();
```

### Usage with External Tools

Generated schemas can be used with:
- **JSON Schema validators** - Runtime validation
- **OpenAPI tools** - API documentation, code generation
- **TypeScript** - Enhanced type checking and IDE support
- **Testing tools** - Schema-based test generation

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
