---
title: "forge-weld-macro"
description: Procedural macros for annotating Rust ops and structs with TypeScript metadata.
slug: crates/forge-weld-macro
---

The `forge-weld-macro` crate provides procedural macros for annotating Rust code with metadata used for TypeScript code generation.

## Overview

The macros in this crate:

1. **Leave original code unchanged** - No modifications to your Rust implementations
2. **Generate companion metadata** - Create functions that return type information
3. **Register with inventory** - Add metadata to forge-weld's compile-time registry

## Macros

### `#[weld_op]`

Annotate deno_core ops for TypeScript binding generation:

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

**Attributes:**

| Attribute | Description |
|-----------|-------------|
| `#[weld_op]` | Sync op |
| `#[weld_op(async)]` | Async op |
| `#[weld_op(ts_name = "customName")]` | Custom TypeScript function name |

**Generated TypeScript:**

```typescript
export function readTextFile(path: string): Promise<string>;
```

### `#[weld_struct]`

Annotate structs for TypeScript interface generation:

```rust
use forge_weld_macro::weld_struct;

#[weld_struct]
#[derive(Serialize, Deserialize)]
pub struct FileStat {
    pub is_file: bool,
    pub is_directory: bool,
    pub size: u64,
    pub modified: Option<u64>,
}
```

**Attributes:**

| Attribute | Description |
|-----------|-------------|
| `#[weld_struct]` | Basic struct |
| `#[weld_struct(ts_name = "CustomName")]` | Custom TypeScript interface name |

**Generated TypeScript:**

```typescript
export interface FileStat {
    is_file: boolean;
    is_directory: boolean;
    size: number;
    modified?: number;
}
```

### `#[weld_enum]`

Annotate enums for TypeScript union type generation:

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

**Generated TypeScript:**

```typescript
export type WatchEventKind = "Create" | "Modify" | "Remove";
```

## How It Works

When you use `#[weld_op]`:

1. The macro parses the function signature
2. Generates a companion function returning `OpSymbol`
3. Registers the symbol in `WELD_OPS` distributed slice

```rust
// Your code
#[weld_op(async)]
#[op2(async)]
pub async fn op_fs_read_text(path: String) -> Result<String, FsError> { ... }

// Generated (simplified) - uses snake_case for Rust naming conventions
#[doc(hidden)]
fn __op_fs_read_text_weld_metadata() -> OpSymbol {
    OpSymbol {
        rust_name: "op_fs_read_text".to_string(),
        ts_name: "readText".to_string(),  // Auto-converted to camelCase
        is_async: true,
        params: vec![OpParam {
            rust_name: "path".to_string(),
            ts_name: "path".to_string(),
            ty: WeldType::Primitive(WeldPrimitive::String),
            // ...
        }],
        return_type: WeldType::Result {
            ok: Box::new(WeldType::Primitive(WeldPrimitive::String)),
            err: Box::new(WeldType::Struct("FsError".to_string())),
        },
        // ...
    }
}

forge_weld::register_op!(__op_fs_read_text_weld_metadata());
```

Similarly for `#[weld_struct]`:

```rust
// Your code
#[weld_struct]
#[derive(Serialize)]
pub struct FileInfo { pub path: String, pub size: u64 }

// Generated - snake_case function name from PascalCase struct
#[doc(hidden)]
fn __file_info_weld_metadata() -> WeldStruct {
    WeldStruct {
        rust_name: "FileInfo".to_string(),
        ts_name: "FileInfo".to_string(),
        fields: vec![/* ... */],
        // ...
    }
}

forge_weld::register_struct!(__file_info_weld_metadata());
```

## Usage Example

Complete example for a filesystem extension:

```rust
use forge_weld_macro::{weld_op, weld_struct, weld_enum};
use serde::{Deserialize, Serialize};

// Define types
#[weld_struct]
#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
}

#[weld_enum]
#[derive(Serialize, Deserialize)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

// Define ops
#[weld_op(async)]
#[op2(async)]
pub async fn op_fs_stat(
    #[string] path: String,
) -> Result<FileInfo, FsError> {
    // implementation
}

#[weld_op]
#[op2]
pub fn op_fs_exists(
    #[string] path: String,
) -> bool {
    // implementation
}
```

## Type Parser

The `type_parser.rs` module provides strict Rust→TypeScript type conversion. It panics on unsupported types to ensure no `unknown` types slip through to generated TypeScript.

**Supported types:**
- **Primitives**: `u8`-`u64`, `i8`-`i64`, `f32`, `f64`, `bool`, `String`, `char`, `()`
- **Containers**: `Option<T>`, `Vec<T>`, `Result<T, E>`, `HashMap<K, V>`, `BTreeMap<K, V>`
- **Sets**: `HashSet<T>`, `BTreeSet<T>`
- **Smart pointers**: `Box<T>`, `Arc<T>`, `Rc<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`
- **References**: `&T`, `&mut T`
- **Tuples**: `(A, B, C)`
- **Arrays/Slices**: `[T; N]`, `[T]`
- **Special**: `serde_json::Value` → `JsonValue`, `OpState`
- **Custom types**: Treated as struct references

**Unsupported types (will panic):**
- Bare function types (`fn(A) -> B`)
- `impl Trait` types
- Trait objects (`dyn Trait`)
- Inferred types (`_`)
- Macro types

## File Structure

```text
crates/forge-weld-macro/
├── src/
│   ├── lib.rs         # Macro entry points
│   ├── weld_op.rs     # #[weld_op] implementation
│   ├── weld_struct.rs # #[weld_struct] and #[weld_enum] implementation
│   └── type_parser.rs # Rust→WeldType conversion with strict validation
└── Cargo.toml
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `syn` | Rust syntax parsing |
| `quote` | Token generation |
| `proc-macro2` | Proc macro utilities |

## Related

- [forge-weld](/docs/crates/forge-weld) - Code generation library that uses these macros
- [Architecture](/docs/architecture) - Build system overview
