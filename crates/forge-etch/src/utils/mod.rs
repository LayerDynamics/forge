//! Utilities for forge-etch
//!
//! This module provides utilities for:
//! - SWC/deno_ast TypeScript parsing
//! - Symbol resolution and tracking
//! - Dependency graph analysis

pub mod graph;
pub mod swc;
pub mod symbols;

pub use swc::{parse_typescript_file, ParsedModule, SourceInfo};
pub use symbols::{SymbolRef, SymbolTable};
