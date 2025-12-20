# Forge Type Mapping Reference

Complete guide to how Rust types are mapped to TypeScript in Forge extensions.

## Quick Reference Table

| Rust Type | TypeScript Type | Notes |
|-----------|----------------|-------|
| `u8`, `u16`, `u32`, `i8`, `i16`, `i32` | `number` | Integer types within JS safe range |
| `u64`, `i64` | `bigint` | Large integers exceeding 2^53 |
| `f32`, `f64` | `number` | Floating point |
| `bool` | `boolean` | Boolean value |
| `String`, `&str` | `string` | Text |
| `char` | `string` | Single character |
| `()` | `void` | Unit/no value |
| `Vec<T>` | `T[]` | Array (except Vec<u8>) |
| `Vec<u8>` | `Uint8Array` | Binary data |
| `Option<T>` | `T \| null` | Nullable value |
| `Result<T, E>` | `Promise<T>` | Async with error throwing |
| `HashMap<K,V>`, `BTreeMap<K,V>` | `Record<K, V>` | Key-value map |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>` | Unique values |
| `(A, B, C)` | `[A, B, C]` | Tuple types |
| `Box<T>`, `Arc<T>`, `Rc<T>` | `T` | Unwrapped |
| `&T`, `&mut T` | `T` | Dereferenced |
| `struct Foo` with `#[weld_struct]` | `interface Foo` | Structure |
| `enum Bar` with `#[weld_enum]` | `type Bar = ...` | Tagged union |
| `serde_json::Value` | `unknown` | Dynamic JSON |

## Detailed Explanations

### Integer Types

JavaScript's `number` type uses IEEE 754 double-precision, which has a safe integer range of ±2^53 - 1.

**Safe Mappings (→ `number`):**
- `u8`, `u16`, `u32` (0 to 4,294,967,295)
- `i8`, `i16`, `i32` (-2,147,483,648 to 2,147,483,647)
- `usize`, `isize` (platform-dependent, typically 64-bit)

**BigInt Mappings (→ `bigint`):**
- `u64` (0 to 18,446,744,073,709,551,615)
- `i64` (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)

**Example:**
```rust
// Rust
#[weld_op]
#[op2]
fn op_get_stats() -> (u32, u64) {
    (1000u32, 1_000_000_000_000u64)
}

// TypeScript
declare function getStats(): [number, bigint];

// Usage
const [count, total] = await getStats();
console.log(typeof count);  // "number"
console.log(typeof total);  // "bigint"
```

### Option Types

`Option<T>` becomes `T | null` (not `T | undefined`).

```rust
// Rust
#[weld_struct]
struct User {
    name: String,
    email: Option<String>,
}

// TypeScript
interface User {
    name: string;
    email: string | null;
}
```

**Nested Options:**
```rust
Option<Option<String>>  →  string | null | null  (simplified to  string | null)
```

### Result Types

All `Result<T, E>` types become `Promise<T>`. Errors are **thrown** as exceptions.

```rust
// Rust
#[weld_op(async)]
#[op2(async)]
async fn op_read_config() -> Result<Config, ConfigError> {
    // ...
}

// TypeScript
declare function readConfig(): Promise<Config>;

// Error handling
try {
    const config = await readConfig();
} catch (err) {
    // ConfigError is thrown here
    console.error(err.message);
}
```

**Error Types:** The error type `E` in `Result<T, E>` is not exposed in TypeScript.
All errors are caught generically as JavaScript `Error` objects.

### Collections

**Vec<T> → T[]:**
```rust
Vec<String>       →  string[]
Vec<Vec<u32>>     →  number[][]
Vec<Option<T>>    →  (T | null)[]
```

**Special case - Vec<u8> → Uint8Array:**
```rust
// Rust
#[weld_op]
#[op2]
fn op_read_bytes() -> Vec<u8> { /* ... */ }

// TypeScript
declare function readBytes(): Uint8Array;

// Usage
const bytes = await readBytes();
console.log(bytes instanceof Uint8Array);  // true
```

**HashMap/BTreeMap → Record:**
```rust
HashMap<String, u32>     →  Record<string, number>
HashMap<u32, String>     →  Record<number, string>  // number keys!

// TypeScript
type Config = Record<string, number>;
const config: Config = { timeout: 5000, retries: 3 };
```

**HashSet/BTreeSet → Set:**
```rust
HashSet<String>  →  Set<string>

// Usage
const tags = new Set<string>(["rust", "typescript"]);
```

### Generics

Type parameters are preserved in the generated TypeScript:

```rust
// Rust
#[weld_struct]
struct Response<T> {
    data: T,
    status: u32,
}

// TypeScript
interface Response<T> {
    data: T;
    status: number;
}

// Usage
type UserResponse = Response<User>;
```

**Complex nested generics:**
```rust
Result<Vec<Option<HashMap<String, User>>>, Error>

// Becomes:
Promise<(Record<string, User> | null)[]>
```

### Tuples

Fixed-length tuples map to TypeScript tuple types:

```rust
(String, u32, bool)  →  [string, number, boolean]

// Usage
const data: [string, number, boolean] = ["hello", 42, true];
const [text, num, flag] = data;  // Destructuring works
```

**Unit tuple:**
```rust
()  →  void
```

### Wrapper Types

Smart pointers and mutability wrappers are **transparently unwrapped**:

```rust
Box<String>              →  string
Arc<Mutex<Config>>       →  Config
Rc<RefCell<Vec<u8>>>     →  Uint8Array
&String                  →  string
&mut Vec<T>              →  T[]
```

This unwrapping happens automatically during code generation. JavaScript
has no concept of ownership or borrowing, so these Rust-specific wrappers
are erased.

### Custom Structs

Structs annotated with `#[weld_struct]` become TypeScript interfaces:

```rust
// Rust
#[weld_struct]
struct Point {
    x: f64,
    y: f64,
}

#[weld_struct]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

// TypeScript
interface Point {
    x: number;
    y: number;
}

interface Rectangle {
    topLeft: Point;    // Field names converted to camelCase
    bottomRight: Point;
}
```

**Field name conversion:**
- `snake_case` (Rust) → `camelCase` (TypeScript)
- `top_left` → `topLeft`
- `user_id` → `userId`

### Custom Enums

Rust enums map to TypeScript discriminated unions:

```rust
// Rust
#[weld_enum]
enum Status {
    Pending,
    Running { progress: u32 },
    Completed { result: String },
    Failed { error: String },
}

// TypeScript
type Status =
    | { type: "Pending" }
    | { type: "Running"; progress: number }
    | { type: "Completed"; result: string }
    | { type: "Failed"; error: string };

// Usage
function handleStatus(status: Status) {
    switch (status.type) {
        case "Pending":
            console.log("Waiting...");
            break;
        case "Running":
            console.log(`Progress: ${status.progress}%`);
            break;
        case "Completed":
            console.log(`Result: ${status.result}`);
            break;
        case "Failed":
            console.error(`Error: ${status.error}`);
            break;
    }
}
```

## Edge Cases and Limitations

### Unsupported Types

These Rust types have no TypeScript equivalent and are mapped to `unknown`:

- **Function pointers:** `fn(i32) -> i32` → `unknown`
- **Raw pointers:** `*const T`, `*mut T` → `unknown`
- **Never type:** `!` → `never`
- **Associated types:** Requires manual annotation

### OpState Filtering

`Rc<RefCell<OpState>>` parameters are automatically **filtered out** of
TypeScript signatures:

```rust
// Rust
#[weld_op]
#[op2]
fn op_example(
    state: Rc<RefCell<OpState>>,  // Filtered
    #[string] name: String,
) -> String {
    // ...
}

// TypeScript (state parameter removed)
declare function example(name: string): string;
```

### Lifetime Parameters

Rust lifetime parameters are erased during type generation:

```rust
struct Borrowed<'a> {
    data: &'a str,
}

// TypeScript (no lifetime)
interface Borrowed {
    data: string;
}
```

## Best Practices

1. **Use explicit types:** Avoid complex type inference; annotate function signatures
2. **Prefer owned types:** `String` over `&str` for public APIs
3. **Document error variants:** TypeScript loses error type information
4. **Avoid pointer types:** Use owned or reference types
5. **Test generated types:** Check `sdk/runtime.*.ts` output

## See Also

- [forge-weld API docs](https://docs.rs/forge-weld)
- [Writing Extensions](./extensions.md)
- [Type System Implementation](../crates/forge-weld/src/ir/types.rs)
