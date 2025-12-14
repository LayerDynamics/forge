//! Code generation for Forge extensions
//!
//! This module provides generators for:
//! - TypeScript init.ts modules (init.ts)
//! - TypeScript declaration files (.d.ts)
//! - Rust extension.rs macro invocations

pub mod dts;
pub mod extension;
pub mod typescript;

pub use dts::{DtsBuilder, DtsGenerator};
pub use extension::{generate_extension_file, ExtensionGenerator};
pub use typescript::TypeScriptGenerator;
