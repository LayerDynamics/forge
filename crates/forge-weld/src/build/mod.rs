//! Build utilities for Forge extensions
//!
//! This module provides utilities for use in build.rs scripts:
//! - TypeScript transpilation via deno_ast
//! - ExtensionBuilder for simplified extension crate setup
//! - PreloadBuilder for generating preload scripts

pub mod extension;
pub mod preload;
pub mod transpile;

pub use extension::{
    host_extension, DocConfig, DocFormat, ExtensionBuilder, ExtensionBuilderError,
};
pub use preload::{
    generate_preload, generate_preload_to_file, PreloadBuilder, PreloadBuilderError,
};
pub use transpile::{transpile_file, transpile_ts, TranspileError};
