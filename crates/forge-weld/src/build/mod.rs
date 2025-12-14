//! Build utilities for Forge extensions
//!
//! This module provides utilities for use in build.rs scripts:
//! - TypeScript transpilation via deno_ast
//! - ExtensionBuilder for simplified extension crate setup

pub mod extension;
pub mod transpile;

pub use extension::{ExtensionBuilder, ExtensionBuilderError};
pub use transpile::{transpile_file, transpile_ts, TranspileError};
