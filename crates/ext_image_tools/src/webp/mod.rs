//! WebP image operations module (for app asset optimization)

// The implementation lives in `webp/webp.rs` and is re-exported here; the
// same-name inner module is intentional.
#[allow(clippy::module_inception)]
mod webp;

pub use webp::*;
