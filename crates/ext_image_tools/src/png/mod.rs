//! PNG image operations module

// The implementation lives in `png/png.rs` and is re-exported here; the
// same-name inner module is intentional.
#[allow(clippy::module_inception)]
mod png;

pub use png::*;
