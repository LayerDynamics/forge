//! Builder API for forge-etch
//!
//! This module provides the main builder API for generating documentation
//! from Forge extensions. It's designed to be used in build.rs scripts.

mod etch_builder;

pub use etch_builder::{BuildOutput, EtchBuilder, OutputFormat};
