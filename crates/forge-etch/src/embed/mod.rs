//! Asset embedding module for forge-etch.
//!
//! This module provides utilities for embedding CSS, JavaScript, and other assets
//! directly into HTML documentation, enabling standalone single-file output.
//!
//! See [`lib`] for the implementation details.

mod lib;

// Re-export all public items from lib
pub use lib::*;
