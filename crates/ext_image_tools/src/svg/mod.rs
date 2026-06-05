//! SVG image operations module

// The implementation lives in `svg/svg.rs` and is re-exported here; the
// same-name inner module is intentional.
#[allow(clippy::module_inception)]
mod svg;

pub use svg::*;
