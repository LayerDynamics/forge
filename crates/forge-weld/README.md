# Forge Weld

This module provides the glue/bindings for the Forge framework. It connects Rust and Deno methods and generates the TypeScript type declarations output by the framework.

## How the Binding/Glue Works

Forge Weld bridges the gap between Rust `deno_core` ops and TypeScript by:

1. **Capturing type metadata** from Rust structs, enums, and op functions via proc macros
2. **Storing metadata at compile time** using `linkme` distributed slices
3. **Generating TypeScript declarations** (`.d.ts` files) from the collected metadata
4. **Transpiling TypeScript init modules** to JavaScript for the Deno runtime

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      forge-weld-macro                           │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────┐             │
│  │ #[weld_op]│  │#[weld_struct]│  │ #[weld_enum] │              │
│  └─────┬────┘  └──────┬───────┘  └───────┬──────┘              │
│        │              │                   │                     │
│        └──────────────┴───────────────────┘                     │
│                       │                                         │
│                       ▼                                         │
│              rust_type_to_weld_type()                           │
│              (Rust → WeldType parsing)                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                        forge-weld                               │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   IR (Intermediate Rep)                   │  │
│  │  ┌─────────────┐  ┌────────────┐  ┌─────────────┐        │  │
│  │  │  WeldType   │  │  OpSymbol  │  │ WeldStruct  │        │  │
│  │  │ WeldPrimitive│  │  OpParam   │  │ WeldEnum    │        │  │
│  │  └─────────────┘  └────────────┘  └─────────────┘        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           │                                     │
│  ┌────────────────────────┴─────────────────────────────────┐  │
│  │                   Inventory (linkme)                      │  │
│  │        WELD_OPS | WELD_STRUCTS | WELD_ENUMS              │  │
│  └────────────────────────┬─────────────────────────────────┘  │
│                           │                                     │
│  ┌────────────────────────┴─────────────────────────────────┐  │
│  │                      Codegen                              │  │
│  │  ┌──────────────┐  ┌───────────────────┐                 │  │
│  │  │ DtsGenerator │  │ ExtensionGenerator│                 │  │
│  │  │  (.d.ts)     │  │  (extension.rs)   │                 │  │
│  │  └──────────────┘  └───────────────────┘                 │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      Build                                │  │
│  │  ExtensionBuilder - for build.rs scripts                 │  │
│  │  transpile_ts() - TypeScript → JavaScript                │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │   Generated Output    │
              │  ┌─────────────────┐  │
              │  │ host.fs.d.ts    │  │
              │  │ host.ui.d.ts    │  │
              │  │ extension.rs    │  │
              │  │ init.js         │  │
              │  └─────────────────┘  │
              └───────────────────────┘
```

### Type System

The `WeldType` enum represents all Rust types that can be mapped to TypeScript:

| Rust Type | WeldType | TypeScript |
|-----------|----------|------------|
| `u8`, `u16`, `u32`, `i8`, `i16`, `i32` | `Primitive(U8/U16/U32/I8/I16/I32)` | `number` |
| `u64`, `i64` | `Primitive(U64/I64)` | `bigint` |
| `f32`, `f64` | `Primitive(F32/F64)` | `number` |
| `bool` | `Primitive(Bool)` | `boolean` |
| `String`, `&str` | `Primitive(String)` | `string` |
| `()` | `Primitive(Unit)` | `void` |
| `Option<T>` | `Option(T)` | `T \| null` |
| `Vec<T>` | `Vec(T)` | `T[]` |
| `Vec<u8>` | `Bytes` | `Uint8Array` |
| `Result<T, E>` | `Result { ok, err }` | `Promise<T>` |
| `HashMap<K, V>` | `HashMap { key, value }` | `Record<K, V>` |
| `HashSet<T>` | `HashSet(T)` | `Set<T>` |
| `(A, B, C)` | `Tuple([A, B, C])` | `[A, B, C]` |
| `serde_json::Value` | `JsonValue` | `unknown` |
| Custom struct | `Struct(name)` | interface |
| Custom enum | `Enum(name)` | type union |

### Proc Macros

#### `#[weld_op]` - Annotate deno_core ops

```rust
use forge_weld_macro::weld_op;

#[weld_op(async)]
#[op2(async)]
pub async fn op_fs_read_text(
    #[string] path: String,
) -> Result<String, FsError> {
    // implementation
}
```

This generates metadata capturing:
- Function name → TypeScript name (`op_fs_read_text` → `readText`)
- Parameter types (with snake_case → camelCase conversion)
- Return type (Result<T, E> → Promise<T>)
- Async flag

#### `#[weld_struct]` - Annotate structs

```rust
use forge_weld_macro::weld_struct;

#[weld_struct]
#[derive(Serialize, Deserialize)]
pub struct FileStat {
    pub is_file: bool,
    pub is_directory: bool,
    pub size: u64,
}
```

Generates TypeScript interface:

```typescript
export interface FileStat {
  isFile: boolean;
  isDirectory: boolean;
  size: bigint;
}
```

#### `#[weld_enum]` - Annotate enums

```rust
use forge_weld_macro::weld_enum;

#[weld_enum]
#[derive(Serialize, Deserialize)]
pub enum WatchEventKind {
    Create,
    Modify,
    Remove,
}
```

Generates TypeScript union type:

```typescript
export type WatchEventKind = "Create" | "Modify" | "Remove";
```

### Build Script Integration

Use `ExtensionBuilder` in your extension's `build.rs`:

```rust
use forge_weld::build::ExtensionBuilder;

fn main() {
    ExtensionBuilder::new("host_fs", "host:fs")
        .ts_path("ts/init.ts")
        .ops(&["op_fs_read_text", "op_fs_write_text"])
        .generate_sdk_types("../../sdk")
        .build()
        .expect("Failed to build extension");
}
```

This will:
1. Transpile `ts/init.ts` to JavaScript
2. Generate `extension.rs` with the module initialization
3. Generate `host.fs.d.ts` in the SDK directory

### Inventory System

Forge Weld uses `linkme` distributed slices to collect metadata at compile time:

```rust
// Automatically registered by proc macros
#[linkme::distributed_slice(WELD_OPS)]
static _WELD_OP: fn() -> OpSymbol = || metadata_fn();

// Collect at runtime
let ops: Vec<OpSymbol> = collect_ops();
let structs: Vec<WeldStruct> = collect_structs();
let enums: Vec<WeldEnum> = collect_enums();
```

This allows the build system to discover all annotated types without manually listing them.

## Modules

- **`ir`**: Intermediate representation types (`WeldType`, `OpSymbol`, `WeldStruct`, `WeldEnum`)
- **`codegen`**: Code generators (`DtsGenerator`, `ExtensionGenerator`, `TypeScriptGenerator`)
- **`build`**: Build script utilities (`ExtensionBuilder`, `transpile_ts`)
