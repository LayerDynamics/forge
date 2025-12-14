//! Code generation for Forge extensions
//!
//! This module provides generators for:
//! - TypeScript init.ts modules (init.ts)
//! - TypeScript declaration files (.d.ts)
//! - Rust extension.rs macro invocations

pub mod typescript;
pub mod dts;
pub mod extension;

pub use typescript::TypeScriptGenerator;
pub use dts::{DtsGenerator, DtsBuilder};
pub use extension::{ExtensionGenerator, generate_extension_file};
