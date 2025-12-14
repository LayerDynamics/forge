//! Intermediate Representation (IR) for Forge extensions
//!
//! This module provides the type system and metadata structures for
//! representing Rust deno_core ops and their TypeScript equivalents.

pub mod types;
pub mod symbol;
pub mod module;
pub mod inventory;

pub use types::*;
pub use symbol::*;
pub use module::*;
pub use inventory::*;
