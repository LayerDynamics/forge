//! Build-time embedding helpers (the `build::` facade).
//!
//! This is the build-script-facing half of forge-smelt. Where [`crate::smelt`]
//! is the CLI/runtime entry point that emits a compiled `.js` tree, [`embed`]
//! goes one step further: it also writes the stable bootstrap module (`init.js`,
//! transpiled from `ts/init.ts` at build time and embedded as [`crate::BOOTSTRAP_SHIM`])
//! and returns an [`EmbedManifest`] describing the full artifact a consuming
//! crate's `build.rs` can bake into a standalone binary (the Depth-2 path).
//!
//! Example (in a consumer's `build.rs`):
//!
//! ```no_run
//! // Compile the app under ./app into $OUT_DIR/app and embed the result.
//! let manifest = forge_smelt::build::embed_in_out_dir("app", "app");
//! println!("cargo:rustc-env=FORGE_APP_ENTRY={}", manifest.bootstrap.display());
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::binary::SmeltOutput;
use crate::compile::smelt;
use crate::{SmeltError, SmeltResult, BOOTSTRAP_SHIM};

/// Descriptor of an embeddable, smelted app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedManifest {
    /// Directory the compiled tree + bootstrap were written to.
    pub out_dir: PathBuf,
    /// Stable bootstrap entry the runtime loads (`<out_dir>/init.js`).
    pub bootstrap: PathBuf,
    /// The app's compiled entry the bootstrap imports (`<out_dir>/main.js`).
    pub app_entry: PathBuf,
    /// All emitted files (bootstrap first, then compiled modules, then assets).
    pub files: Vec<PathBuf>,
    /// Total bytes written (compiled JS + bootstrap).
    pub bytes_written: u64,
}

/// Smelt `app_dir` into `out_dir` and write the bootstrap shim alongside the
/// compiled module tree, returning an [`EmbedManifest`] for embedding the app.
pub fn embed(app_dir: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> SmeltResult<EmbedManifest> {
    let out_dir = out_dir.as_ref();
    let SmeltOutput {
        entry: app_entry,
        modules,
        assets,
        bytes_written,
        ..
    } = smelt(app_dir, out_dir)?;

    // Write the stable bootstrap entry that imports the compiled app entry.
    let bootstrap = out_dir.join("init.js");
    std::fs::write(&bootstrap, BOOTSTRAP_SHIM.as_bytes())
        .map_err(|e| SmeltError::write(&bootstrap, e))?;

    let mut files = Vec::with_capacity(modules.len() + assets.len() + 1);
    files.push(bootstrap.clone());
    files.extend(modules);
    files.extend(assets);

    Ok(EmbedManifest {
        out_dir: out_dir.to_path_buf(),
        bootstrap,
        app_entry,
        files,
        bytes_written: bytes_written + BOOTSTRAP_SHIM.len() as u64,
    })
}

/// Convenience wrapper for use inside a consumer's `build.rs`: smelts `app_dir`
/// into `$OUT_DIR/<sub_dir>`, emits `cargo:rerun-if-changed` for the app source,
/// and returns the [`EmbedManifest`]. Panics with a clear message on failure
/// (the conventional build-script behavior).
pub fn embed_in_out_dir(app_dir: impl AsRef<Path>, sub_dir: &str) -> EmbedManifest {
    let app_dir = app_dir.as_ref();
    println!("cargo:rerun-if-changed={}/src", app_dir.display());

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is only set inside a build script");
    let dest = Path::new(&out_dir).join(sub_dir);

    embed(app_dir, &dest).unwrap_or_else(|e| panic!("forge-smelt embed failed: {e}"))
}
