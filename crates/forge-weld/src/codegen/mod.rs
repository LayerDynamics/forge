//! Code generation for Forge extensions
//!
//! This module provides generators for:
//! - TypeScript init.ts modules (init.ts)
//! - TypeScript declaration files (.d.ts)
//! - Rust extension.rs macro invocations
//! - Extensibility APIs (hooks, handlers)
//! - Preload scripts for WebView renderers
//! - TypeScript SDK classes (class-based SDKs)
//! - JSON Schema and OpenAPI specifications

pub mod dts;
pub mod extensibility;
pub mod extension;
pub mod preload;
pub mod schema;
pub mod sdk_class;
pub mod typescript;

pub use dts::{DtsBuilder, DtsGenerator};
pub use extensibility::ExtensibilityGenerator;
pub use extension::{generate_extension_file, ExtensionGenerator};
pub use preload::PreloadGenerator;
pub use schema::{GeneratedSchema, SchemaGenerator};
pub use sdk_class::SdkClassGenerator;
pub use typescript::TypeScriptGenerator;
