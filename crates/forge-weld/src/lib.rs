//! Forge-Weld: Code generation and binding utilities for Forge framework
//!
//! This module provides the glue between Rust deno_core ops and TypeScript.
//! It generates TypeScript type definitions, init.ts modules, and extension.rs
//! macro invocations from Rust op definitions.
//!
//! # Architecture
//!
//! - `ir`: Intermediate representation for types, symbols, and modules
//! - `codegen`: Code generation for TypeScript and Rust
//! - `build`: Build script utilities for extension crates
//!
//! # Usage
//!
//! In your extension's `build.rs`:
//!
//! ```rust,ignore
//! use forge_weld::build::ExtensionBuilder;
//!
//! fn main() {
//!     ExtensionBuilder::new("host_fs", "runtime:fs")
//!         .ts_path("ts/init.ts")
//!         .ops(&["op_fs_read_text", "op_fs_write_text"])
//!         .generate_sdk_types("../../sdk")
//!         .build()
//!         .expect("Failed to build extension");
//! }
//! ```

pub mod build;
pub mod codegen;
pub mod ir;

// Re-export commonly used types
pub use build::{
    transpile_file, transpile_ts, ExtensionBuilder, ExtensionBuilderError, TranspileError,
};
pub use codegen::{DtsBuilder, DtsGenerator, ExtensionGenerator, TypeScriptGenerator};
pub use ir::{
    collect_enums, collect_ops, collect_structs, EnumVariant, ModuleBuilder, ModuleValidationError,
    Op2Attrs, OpParam, OpSymbol, ParamAttr, StructField, SymbolRegistry, WeldEnum, WeldModule,
    WeldPrimitive, WeldStruct, WeldType, WELD_ENUMS, WELD_OPS, WELD_STRUCTS,
};

// Re-export linkme for inventory
pub use linkme;
