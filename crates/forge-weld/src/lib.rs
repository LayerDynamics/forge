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
//!     ExtensionBuilder::new("host_fs", "host:fs")
//!         .ts_path("ts/init.ts")
//!         .ops(&["op_fs_read_text", "op_fs_write_text"])
//!         .generate_sdk_types("../../sdk")
//!         .build()
//!         .expect("Failed to build extension");
//! }
//! ```

pub mod ir;
pub mod codegen;
pub mod build;

// Re-export commonly used types
pub use ir::{
    WeldType, WeldPrimitive, OpSymbol, WeldStruct, StructField, WeldModule,
    OpParam, ParamAttr, Op2Attrs, WeldEnum, EnumVariant, ModuleBuilder,
    ModuleValidationError, SymbolRegistry, collect_ops, collect_structs, collect_enums,
    WELD_OPS, WELD_STRUCTS, WELD_ENUMS,
};
pub use codegen::{TypeScriptGenerator, DtsGenerator, ExtensionGenerator, DtsBuilder};
pub use build::{transpile_ts, transpile_file, ExtensionBuilder, ExtensionBuilderError, TranspileError};

// Re-export linkme for inventory
pub use linkme;
