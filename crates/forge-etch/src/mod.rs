//! forge-etch: Documentation generator for Forge framework
//!
//! This crate generates documentation from Forge extensions by:
//! - Parsing TypeScript source files (ts/init.ts) using deno_ast/SWC
//! - Extracting metadata from forge-weld IR (ops, structs, enums)
//! - Merging documentation sources with TypeScript JSDoc taking precedence
//! - Generating Astro-compatible markdown for the documentation site
//! - Generating standalone HTML documentation
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐
//! │ ts/init.ts      │    │ forge-weld IR    │
//! │ (SWC parse)     │    │ (ops/structs)    │
//! └────────┬────────┘    └────────┬─────────┘
//!          │                      │
//!          └──────────┬───────────┘
//!                     ▼
//!              ┌──────────────┐
//!              │   EtchNode   │
//!              └──────┬───────┘
//!                     │
//!          ┌──────────┴──────────┐
//!          ▼                     ▼
//!    ┌──────────┐         ┌──────────┐
//!    │ Astro MD │         │   HTML   │
//!    └──────────┘         └──────────┘
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use forge_etch::EtchBuilder;
//!
//! // In build.rs
//! EtchBuilder::new("host_fs", "runtime:fs")
//!     .rust_source("src/lib.rs")
//!     .ts_source("ts/init.ts")
//!     .generate_astro(true)
//!     .generate_html(true)
//!     .build()
//!     .expect("Failed to generate docs");
//! ```

// Core types
pub mod js_doc;
pub mod node;
pub mod params;
pub mod types;

// TypeScript construct types
pub mod class;
pub mod decorators;
pub mod r#enum;
pub mod function;
pub mod interface;
pub mod ts_type_params;
pub mod ts_types;
pub mod type_alias;
pub mod variable;

// Parsing and utilities
pub mod deno;
pub mod diagnostics;
pub mod parser;
pub mod printer;
pub mod test;
pub mod visibility;

// Submodules
pub mod astro;
pub mod builder;
pub mod docgen;
pub mod embed;
pub mod html;
pub mod utils;

// Re-exports for convenience
pub use class::ClassDef;
pub use diagnostics::{EtchError, EtchResult};
pub use function::{FunctionDef, OpDef};
pub use interface::InterfaceDef;
pub use js_doc::{EtchDoc, JsDocTag};
pub use node::{EtchNode, EtchNodeDef, EtchNodeKind, Location};
pub use params::ParamDef;
pub use r#enum::EnumDef;
pub use type_alias::TypeAliasDef;
pub use types::{EtchLiteral, EtchPrimitive, EtchType, EtchTypeKind};
pub use variable::VariableDef;

// Terminal output
pub use printer::EtchPrinter;

// Deno utilities
pub use deno::{deno_version, from_file_url, is_deno_runtime, to_file_url, DenoConfig};
pub use deno::{generate_deno_imports, jsr_import, jsr_import_latest, ModuleImport};

// Asset embedding
pub use embed::{all_assets, get_asset, list_assets, COPY_BUTTON_JS, DEFAULT_CSS, SEARCH_JS};
pub use embed::{embed_in_html, generate_standalone_html, EmbedConfig, EmbeddedAsset};

// Builder API
pub use builder::{BuildOutput, EtchBuilder};

// Documentation generation
pub use docgen::extension::ExtensionStats;
pub use docgen::rust::get_type_exports;
pub use docgen::typescript::{extract_exports, is_declaration_file, is_typescript_file};
pub use docgen::{EtchConfig, Etcher, ExtensionDoc};

// Output generators
pub use astro::check_config::{
    check_config, validate_config, validate_output_dir, AstroConfig, SidebarItem, StarlightConfig,
    ValidationResult,
};
pub use astro::compat::{
    detect_version, frontmatter_for_version, supports_feature, AstroVersion, CompatConfig,
    FrontmatterStyle,
};
pub use astro::slug::{anchor_slug, file_slug, slug as astro_slug, slugify_path, unique_slug};
pub use astro::AstroGenerator;
pub use html::types::{HtmlConfig, PageContext};
pub use html::HtmlGenerator;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const NAME: &str = env!("CARGO_PKG_NAME");

// Test utilities - available for downstream crate testing
// Note: The test module provides mock builders and fixtures for testing forge-etch functionality
