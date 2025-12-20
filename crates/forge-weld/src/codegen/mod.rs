//! Code generation for Forge extensions
//!
//! This module provides generators for:
//! - TypeScript init.ts modules (init.ts)
//! - TypeScript declaration files (.d.ts)
//! - Rust extension.rs macro invocations
//! - Extensibility APIs (hooks, handlers)
//! - Preload scripts for WebView renderers

pub mod dts;
pub mod extensibility;
pub mod extension;
pub mod preload;
pub mod typescript;

pub use dts::{DtsBuilder, DtsGenerator};
pub use extensibility::ExtensibilityGenerator;
pub use extension::{generate_extension_file, ExtensionGenerator};
pub use preload::PreloadGenerator;
pub use typescript::TypeScriptGenerator;
