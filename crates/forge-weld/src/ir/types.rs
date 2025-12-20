//! Type system for Forge-Weld
//!
//! This module provides the type representations for mapping Rust types
//! to TypeScript types, with full generics support.
//!
//! # Type Mapping Overview
//!
//! Forge-Weld automatically generates TypeScript bindings from Rust ops.
//! The type system handles primitive types, generics, collections, and
//! custom structs/enums with high fidelity.
//!
//! ## Primitive Type Mapping
//!
//! | Rust Type | TypeScript Type | Notes |
//! |-----------|----------------|-------|
//! | `u8`, `u16`, `u32` | `number` | Safe integer range |
//! | `i8`, `i16`, `i32` | `number` | Safe integer range |
//! | `u64`, `i64` | `bigint` | Exceeds JS safe integer max |
//! | `f32`, `f64` | `number` | IEEE 754 double precision |
//! | `usize`, `isize` | `number` | Platform-dependent size |
//! | `bool` | `boolean` | Direct mapping |
//! | `String`, `&str` | `string` | Owned and borrowed strings |
//! | `char` | `string` | Single character |
//! | `()` | `void` | Unit type |
//!
//! ## Collection Type Mapping
//!
//! | Rust Type | TypeScript Type | Notes |
//! |-----------|----------------|-------|
//! | `Vec<T>` | `T[]` | Generic array |
//! | `Vec<u8>` | `Uint8Array` | Special case for binary data |
//! | `Option<T>` | `T \| null` | Nullable types |
//! | `Result<T, E>` | `Promise<T>` | Errors become rejections |
//! | `HashMap<K, V>` | `Record<K, V>` | Key-value map |
//! | `BTreeMap<K, V>` | `Record<K, V>` | Ordered map (order not preserved) |
//! | `HashSet<T>` | `Set<T>` | Unique values |
//! | `BTreeSet<T>` | `Set<T>` | Ordered set (order not preserved) |
//! | `(A, B, C)` | `[A, B, C]` | Tuple → fixed-length array |
//! | `[T; N]` | `T[]` | Fixed-size array → dynamic |
//!
//! ## Wrapper Type Unwrapping
//!
//! Smart pointers and interior mutability types are transparently unwrapped:
//!
//! | Rust Type | TypeScript Type | Notes |
//! |-----------|----------------|-------|
//! | `Box<T>` | `T` | Heap allocation wrapper |
//! | `Arc<T>` | `T` | Atomic reference counted |
//! | `Rc<T>` | `T` | Reference counted |
//! | `RefCell<T>` | `T` | Runtime borrow checking |
//! | `Mutex<T>` | `T` | Thread-safe interior mutability |
//! | `RwLock<T>` | `T` | Reader-writer lock |
//! | `&T`, `&mut T` | `T` | References dereferenced |
//!
//! ## Custom Types
//!
//! | Rust Type | TypeScript Type | Notes |
//! |-----------|----------------|-------|
//! | `#[weld_struct] struct Foo` | `interface Foo` | Struct → interface |
//! | `#[weld_enum] enum Bar` | `type Bar = ...` | Enum → union type |
//! | `serde_json::Value` | `unknown` | Dynamic JSON |
//! | `Rc<RefCell<OpState>>` | _filtered_ | Internal runtime state |
//!
//! ## Generic Type Preservation
//!
//! Generic type parameters are preserved across the boundary:
//!
//! ```rust,ignore
//! // Rust
//! struct Container<T> {
//!     value: T,
//! }
//!
//! // TypeScript
//! interface Container<T> {
//!     value: T;
//! }
//! ```
//!
//! Nested generics work correctly:
//! ```rust,ignore
//! Result<Vec<Option<String>>, Error>  // Rust
//! Promise<(string | null)[]>          // TypeScript
//! ```
//!
//! # Error Conversion
//!
//! `Result<T, E>` types are converted to async operations:
//!
//! ```rust,ignore
//! // Rust op signature
//! #[weld_op(async)]
//! #[op2(async)]
//! async fn op_read_file(path: String) -> Result<String, FsError>
//!
//! // TypeScript signature
//! export function readFile(path: string): Promise<string>
//! ```
//!
//! Errors are thrown as JavaScript exceptions:
//! ```typescript
//! try {
//!     const content = await readFile("/secret.txt");
//! } catch (error) {
//!     console.error(error.message);  // "Permission denied: fs.read for /secret.txt"
//! }
//! ```
//!
//! # Unsupported Types
//!
//! Some Rust types cannot be directly represented in TypeScript:
//!
//! - **Raw pointers** (`*const T`, `*mut T`): Unsafe, no TS equivalent
//! - **Function pointers** (`fn()`): Use closures or trait objects instead
//! - **Lifetime parameters** (`'a`): Erased at type generation
//! - **Associated types**: Requires manual type annotation
//!
//! These types can still be used internally but won't appear in generated
//! TypeScript bindings. Use [`WeldType::Unknown`] as a fallback.
//!
//! # Implementation Notes
//!
//! - Type conversion happens at build time via `build.rs`
//! - Uses proc macros `#[weld_op]`, `#[weld_struct]`, `#[weld_enum]`
//! - Types collected via `linkme` inventory system
//! - Generated types written to `sdk/runtime.*.ts`
//!
//! See [`WeldType`] for the IR type representation and [`WeldPrimitive`]
//! for primitive type definitions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Primitive types supported by deno_core ops
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeldPrimitive {
    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    Usize,
    // Signed integers
    I8,
    I16,
    I32,
    I64,
    Isize,
    // Floats
    F32,
    F64,
    // Other primitives
    Bool,
    String,
    Str, // &str (treated same as String in TS)
    Char,
    Unit, // ()
}

impl WeldPrimitive {
    /// Convert to TypeScript type string
    pub fn to_typescript(&self) -> &'static str {
        match self {
            // Small integers map to number
            WeldPrimitive::U8 | WeldPrimitive::U16 | WeldPrimitive::U32 => "number",
            WeldPrimitive::I8 | WeldPrimitive::I16 | WeldPrimitive::I32 => "number",
            // Large integers map to bigint (can exceed JS safe integer)
            WeldPrimitive::U64 | WeldPrimitive::I64 => "bigint",
            WeldPrimitive::Usize | WeldPrimitive::Isize => "number", // platform-dependent, use number
            // Floats
            WeldPrimitive::F32 | WeldPrimitive::F64 => "number",
            // Other primitives
            WeldPrimitive::Bool => "boolean",
            WeldPrimitive::String | WeldPrimitive::Str | WeldPrimitive::Char => "string",
            WeldPrimitive::Unit => "void",
        }
    }

    /// Parse from Rust type string
    pub fn from_rust_type(s: &str) -> Option<Self> {
        match s {
            "u8" => Some(WeldPrimitive::U8),
            "u16" => Some(WeldPrimitive::U16),
            "u32" => Some(WeldPrimitive::U32),
            "u64" => Some(WeldPrimitive::U64),
            "usize" => Some(WeldPrimitive::Usize),
            "i8" => Some(WeldPrimitive::I8),
            "i16" => Some(WeldPrimitive::I16),
            "i32" => Some(WeldPrimitive::I32),
            "i64" => Some(WeldPrimitive::I64),
            "isize" => Some(WeldPrimitive::Isize),
            "f32" => Some(WeldPrimitive::F32),
            "f64" => Some(WeldPrimitive::F64),
            "bool" => Some(WeldPrimitive::Bool),
            "String" | "&str" | "str" => Some(WeldPrimitive::String),
            "char" => Some(WeldPrimitive::Char),
            "()" => Some(WeldPrimitive::Unit),
            _ => None,
        }
    }
}

impl fmt::Display for WeldPrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_typescript())
    }
}

/// Complex/composite types with full generics support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WeldType {
    /// Primitive type
    Primitive(WeldPrimitive),

    /// Option<T> -> T | null
    Option(Box<WeldType>),

    /// Vec<T> -> T[] (except Vec<u8>)
    Vec(Box<WeldType>),

    /// Vec<u8> -> Uint8Array (special case)
    Bytes,

    /// Result<T, E> -> Promise<T> (errors thrown)
    Result {
        ok: Box<WeldType>,
        err: Box<WeldType>,
    },

    /// HashMap<K, V> -> Record<K, V>
    HashMap {
        key: Box<WeldType>,
        value: Box<WeldType>,
    },

    /// BTreeMap<K, V> -> Record<K, V>
    BTreeMap {
        key: Box<WeldType>,
        value: Box<WeldType>,
    },

    /// HashSet<T> -> Set<T>
    HashSet(Box<WeldType>),

    /// BTreeSet<T> -> Set<T>
    BTreeSet(Box<WeldType>),

    /// Tuple types (A, B, C) -> [A, B, C]
    Tuple(Vec<WeldType>),

    /// Array [T; N] -> T[] (fixed size arrays)
    Array { element: Box<WeldType>, size: usize },

    /// Generic type with parameters: Foo<T, U> -> Foo<T, U>
    Generic { base: String, params: Vec<WeldType> },

    /// Custom struct reference -> interface name
    Struct(String),

    /// Enum type reference
    Enum(String),

    /// serde_json::Value -> unknown
    JsonValue,

    /// Rc<RefCell<OpState>> -> (internal, filtered out)
    OpState,

    /// Box<T> -> T (unwrapped)
    Box(Box<WeldType>),

    /// Arc<T> -> T (unwrapped)
    Arc(Box<WeldType>),

    /// Rc<T> -> T (unwrapped)
    Rc(Box<WeldType>),

    /// RefCell<T> -> T (unwrapped)
    RefCell(Box<WeldType>),

    /// Mutex<T> -> T (unwrapped)
    Mutex(Box<WeldType>),

    /// RwLock<T> -> T (unwrapped)
    RwLock(Box<WeldType>),

    /// Reference &T or &mut T -> T (dereferenced)
    Reference { inner: Box<WeldType>, mutable: bool },

    /// Raw pointer *const T or *mut T -> T (for advanced use)
    Pointer { inner: Box<WeldType>, mutable: bool },

    /// Never type ! -> never
    Never,

    /// Unknown/Any type (fallback)
    #[default]
    Unknown,
}

impl WeldType {
    /// Convert to TypeScript type string
    pub fn to_typescript(&self) -> String {
        match self {
            WeldType::Primitive(p) => p.to_typescript().to_string(),

            WeldType::Option(inner) => format!("{} | null", inner.to_typescript()),

            WeldType::Vec(inner) => format!("{}[]", inner.to_typescript_with_parens()),

            WeldType::Bytes => "Uint8Array".to_string(),

            WeldType::Result { ok, .. } => {
                // Result<T, E> becomes Promise<T>, errors are thrown
                format!("Promise<{}>", ok.to_typescript())
            }

            WeldType::HashMap { key, value } | WeldType::BTreeMap { key, value } => {
                format!("Record<{}, {}>", key.to_typescript(), value.to_typescript())
            }

            WeldType::HashSet(inner) | WeldType::BTreeSet(inner) => {
                format!("Set<{}>", inner.to_typescript())
            }

            WeldType::Tuple(elements) => {
                let types: Vec<String> = elements.iter().map(|t| t.to_typescript()).collect();
                format!("[{}]", types.join(", "))
            }

            WeldType::Array { element, .. } => {
                format!("{}[]", element.to_typescript_with_parens())
            }

            WeldType::Generic { base, params } => {
                if params.is_empty() {
                    base.clone()
                } else {
                    let type_params: Vec<String> =
                        params.iter().map(|t| t.to_typescript()).collect();
                    format!("{}<{}>", base, type_params.join(", "))
                }
            }

            WeldType::Struct(name) | WeldType::Enum(name) => name.clone(),

            WeldType::JsonValue => "unknown".to_string(),

            WeldType::OpState => "/* OpState */".to_string(),

            // Wrapper types unwrap to their inner type
            WeldType::Box(inner)
            | WeldType::Arc(inner)
            | WeldType::Rc(inner)
            | WeldType::RefCell(inner)
            | WeldType::Mutex(inner)
            | WeldType::RwLock(inner)
            | WeldType::Reference { inner, .. }
            | WeldType::Pointer { inner, .. } => inner.to_typescript(),

            WeldType::Never => "never".to_string(),

            WeldType::Unknown => "unknown".to_string(),
        }
    }

    /// Convert to TypeScript with parentheses if needed (for arrays)
    fn to_typescript_with_parens(&self) -> String {
        match self {
            // Types that need parentheses when used as array element
            WeldType::Option(_) | WeldType::Result { .. } => {
                format!("({})", self.to_typescript())
            }
            _ => self.to_typescript(),
        }
    }

    /// Check if this type is an async Result (used for return types)
    pub fn is_async_result(&self) -> bool {
        matches!(self, WeldType::Result { .. })
    }

    /// Check if this is the OpState type (should be filtered from params)
    pub fn is_op_state(&self) -> bool {
        matches!(self, WeldType::OpState)
    }

    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, WeldType::Primitive(_))
    }

    /// Unwrap wrapper types (Box, Arc, Rc, etc.)
    pub fn unwrap_wrappers(&self) -> &WeldType {
        match self {
            WeldType::Box(inner)
            | WeldType::Arc(inner)
            | WeldType::Rc(inner)
            | WeldType::RefCell(inner)
            | WeldType::Mutex(inner)
            | WeldType::RwLock(inner)
            | WeldType::Reference { inner, .. }
            | WeldType::Pointer { inner, .. } => inner.unwrap_wrappers(),
            _ => self,
        }
    }

    /// Create a primitive type
    pub fn primitive(p: WeldPrimitive) -> Self {
        WeldType::Primitive(p)
    }

    /// Create a string type
    pub fn string() -> Self {
        WeldType::Primitive(WeldPrimitive::String)
    }

    /// Create a boolean type
    pub fn bool() -> Self {
        WeldType::Primitive(WeldPrimitive::Bool)
    }

    /// Create a void/unit type
    pub fn void() -> Self {
        WeldType::Primitive(WeldPrimitive::Unit)
    }

    /// Create an Option<T> type
    pub fn option(inner: WeldType) -> Self {
        WeldType::Option(Box::new(inner))
    }

    /// Create a Vec<T> type
    pub fn vec(inner: WeldType) -> Self {
        // Special case: Vec<u8> is Bytes
        if inner == WeldType::Primitive(WeldPrimitive::U8) {
            WeldType::Bytes
        } else {
            WeldType::Vec(Box::new(inner))
        }
    }

    /// Create a Result<T, E> type
    pub fn result(ok: WeldType, err: WeldType) -> Self {
        WeldType::Result {
            ok: Box::new(ok),
            err: Box::new(err),
        }
    }

    /// Create a HashMap<K, V> type
    pub fn hashmap(key: WeldType, value: WeldType) -> Self {
        WeldType::HashMap {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    /// Create a tuple type
    pub fn tuple(elements: Vec<WeldType>) -> Self {
        WeldType::Tuple(elements)
    }

    /// Create a struct reference
    pub fn struct_ref(name: impl Into<String>) -> Self {
        WeldType::Struct(name.into())
    }
}

impl fmt::Display for WeldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_typescript())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_typescript() {
        assert_eq!(WeldPrimitive::U32.to_typescript(), "number");
        assert_eq!(WeldPrimitive::U64.to_typescript(), "bigint");
        assert_eq!(WeldPrimitive::Bool.to_typescript(), "boolean");
        assert_eq!(WeldPrimitive::String.to_typescript(), "string");
        assert_eq!(WeldPrimitive::Unit.to_typescript(), "void");
    }

    #[test]
    fn test_complex_types() {
        // Option<string> -> string | null
        let opt = WeldType::option(WeldType::string());
        assert_eq!(opt.to_typescript(), "string | null");

        // Vec<number> -> number[]
        let vec = WeldType::vec(WeldType::Primitive(WeldPrimitive::U32));
        assert_eq!(vec.to_typescript(), "number[]");

        // Vec<u8> -> Uint8Array
        let bytes = WeldType::vec(WeldType::Primitive(WeldPrimitive::U8));
        assert_eq!(bytes.to_typescript(), "Uint8Array");

        // Result<string, Error> -> Promise<string>
        let result = WeldType::result(WeldType::string(), WeldType::struct_ref("Error"));
        assert_eq!(result.to_typescript(), "Promise<string>");

        // HashMap<string, number> -> Record<string, number>
        let map = WeldType::hashmap(WeldType::string(), WeldType::Primitive(WeldPrimitive::U32));
        assert_eq!(map.to_typescript(), "Record<string, number>");

        // (string, number, bool) -> [string, number, boolean]
        let tuple = WeldType::tuple(vec![
            WeldType::string(),
            WeldType::Primitive(WeldPrimitive::U32),
            WeldType::bool(),
        ]);
        assert_eq!(tuple.to_typescript(), "[string, number, boolean]");
    }

    #[test]
    fn test_nested_generics() {
        // Result<Vec<Option<string>>, Error> -> Promise<(string | null)[]>
        let nested = WeldType::result(
            WeldType::Vec(Box::new(WeldType::option(WeldType::string()))),
            WeldType::struct_ref("Error"),
        );
        assert_eq!(nested.to_typescript(), "Promise<(string | null)[]>");
    }
}
