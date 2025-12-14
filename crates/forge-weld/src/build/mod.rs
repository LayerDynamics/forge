//! Build utilities for Forge extensions
//!
//! This module provides utilities for use in build.rs scripts:
//! - TypeScript transpilation via deno_ast
//! - ExtensionBuilder for simplified extension crate setup

pub mod transpile;
pub mod extension;

pub use transpile::{transpile_ts, transpile_file, TranspileError};
pub use extension::{ExtensionBuilder, ExtensionBuilderError};
