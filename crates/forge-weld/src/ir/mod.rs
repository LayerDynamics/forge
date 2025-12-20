//! Intermediate Representation (IR) for Forge extensions
//!
//! This module provides the type system and metadata structures for
//! representing Rust deno_core ops and their TypeScript equivalents.

pub mod extensibility;
pub mod inventory;
pub mod module;
pub mod symbol;
pub mod types;

pub use extensibility::*;
pub use inventory::*;
pub use module::*;
pub use symbol::*;
pub use types::*;
