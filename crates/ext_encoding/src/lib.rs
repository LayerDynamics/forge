//! ext_encoding - TextEncoder/TextDecoder support for Forge runtime
//!
//! This extension provides the standard Web TextEncoder and TextDecoder APIs
//! that are normally provided by deno_web but not included in a minimal JsRuntime.
//!
//! The implementation is pure JavaScript - no Rust ops are required since
//! UTF-8 encoding/decoding can be done efficiently in JS.

use deno_core::{Extension, OpState};

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

/// Create the encoding extension
pub fn encoding_extension() -> Extension {
    runtime_encoding::ext()
}

/// Initialize encoding state (no state needed - pure JS)
pub fn init_encoding_state(_state: &mut OpState) {
    // No state needed - TextEncoder/TextDecoder are pure JS
}
