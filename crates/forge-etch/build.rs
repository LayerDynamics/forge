//! Build script for forge-etch
//!
//! This build script handles any compile-time code generation needed
//! for the documentation generator. Currently minimal, but can be
//! extended for embedding templates or generating parsers.

fn main() {
    // Rebuild if templates change
    println!("cargo:rerun-if-changed=src/html/templates/");
    println!("cargo:rerun-if-changed=build.rs");
}
