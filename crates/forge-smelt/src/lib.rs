//! # forge-smelt — ahead-of-time TypeScript → JavaScript compiler for Forge apps
//!
//! ## Why this crate exists (Phase-0 decision)
//!
//! A Forge app's logic lives in `src/main.ts` (plus its local `.ts` module
//! graph) and is executed by the embedded Deno runtime. Today that TypeScript is
//! shipped and run as **loose source**:
//!
//! - `forge bundle` ([`forge_cli`]) embeds only the `web/` frontend assets into
//!   the `forge-runtime` binary (`build_embedded_binary`) and copies the app's
//!   `src/` directory into the package as raw `.ts` files.
//! - `forge-runtime` then loads `src/main.ts` from disk and transpiles it (and
//!   every imported module) **on every launch** (`main.rs`, `ForgeModuleLoader`).
//!
//! Nothing in the workspace compiles the app's TypeScript ahead of time:
//! `ext_bundler` only handles icons/manifests, and `forge-weld`'s transpile is a
//! single-module helper. `forge-smelt` fills that gap — it "smelts" the raw TS
//! ore into a finished JavaScript ingot: parse the entry's module graph,
//! transpile each module to JS, and rewrite relative import specifiers so the
//! emitted `.js` tree is self-consistent and loadable with no further transpile.
//!
//! ## Scope
//!
//! **Depth 1 (this crate): transpile-in-place.** Produce a compiled `.js` tree
//! mirroring `src/`, with relative `./x.ts` import specifiers rewritten to
//! `./x.js` and `runtime:*` / bare / URL specifiers left untouched (the runtime
//! and import maps resolve those). `forge bundle` ships this compiled tree and
//! `forge-runtime` prefers a compiled `src/main.js` when present — so bundled
//! apps stop shipping loose `.ts` and stop re-transpiling at launch, while dev
//! mode keeps loading `.ts` (HMR intact).
//!
//! **Depth 2 (deferred): embed-in-binary.** Linking the compiled JS (and a V8
//! snapshot) directly into a single self-contained executable is a larger change
//! that requires a `forge-runtime` module-loader rewrite. It is intentionally
//! out of scope here and noted as a follow-up; the [`binary`] module produces the
//! materialized artifact that a future Depth-2 step would embed.
//!
//! ## Pipeline
//!
//! ```text
//! app/src/main.ts ──▶ parse ──▶ transpile (+ specifier rewrite) ──▶ binary
//!   (module graph)   (graph)     (TS→JS via forge-weld)            (write .js tree)
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use forge_smelt::smelt;
//! let out = smelt("examples/react-app", "examples/react-app/dist/src")?;
//! println!("compiled entry: {}", out.entry.display());
//! # Ok::<(), forge_smelt::SmeltError>(())
//! ```

pub mod binary;
pub mod build;
pub mod compile;
pub mod parse;
pub mod transpile;

/// Common re-exports for ergonomic glob import (`use forge_smelt::prelude::*`).
/// Lives in `src/mod.rs`, wired in explicitly so the file participates in the
/// crate rather than sitting unused.
#[path = "mod.rs"]
pub mod prelude;

use std::path::Path;

pub use binary::SmeltOutput;
pub use build::{embed, EmbedManifest};
pub use compile::{smelt, smelt_entry};
pub use parse::{ModuleGraph, ModuleNode};

/// The compiled bootstrap shim (`ts/init.ts` -> `init.js`), produced by the
/// build script. A smelted app's stable entry module: it imports the app's
/// compiled `main.js`. [`build::embed`] writes this next to the module tree.
pub const BOOTSTRAP_SHIM: &str = include_str!(concat!(env!("OUT_DIR"), "/init.js"));

/// Error codes for smelt operations (9100-9106).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SmeltErrorCode {
    /// Entry point not found (9100).
    EntryNotFound = 9100,
    /// Failed to read a source file (9101).
    Io = 9101,
    /// Failed to parse a TypeScript module (9102).
    Parse = 9102,
    /// Failed to transpile a module (9103).
    Transpile = 9103,
    /// A relative import could not be resolved to a file (9104).
    UnresolvedImport = 9104,
    /// Failed to write a compiled artifact (9105).
    Write = 9105,
    /// Invalid input/configuration (9106).
    InvalidInput = 9106,
}

impl std::fmt::Display for SmeltErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}

/// Errors produced while smelting an app's TypeScript.
#[derive(Debug, thiserror::Error)]
pub enum SmeltError {
    #[error("[{code}] Entry point not found: {path}")]
    EntryNotFound { code: SmeltErrorCode, path: String },

    #[error("[{code}] I/O error for {path}: {source}")]
    Io {
        code: SmeltErrorCode,
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("[{code}] Failed to parse {path}: {message}")]
    Parse {
        code: SmeltErrorCode,
        path: String,
        message: String,
    },

    #[error("[{code}] Failed to transpile {path}: {message}")]
    Transpile {
        code: SmeltErrorCode,
        path: String,
        message: String,
    },

    #[error("[{code}] Unresolved import '{specifier}' from {referrer}")]
    UnresolvedImport {
        code: SmeltErrorCode,
        specifier: String,
        referrer: String,
    },

    #[error("[{code}] Failed to write {path}: {source}")]
    Write {
        code: SmeltErrorCode,
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("[{code}] Invalid input: {message}")]
    InvalidInput {
        code: SmeltErrorCode,
        message: String,
    },
}

impl SmeltError {
    pub fn entry_not_found(path: impl AsRef<Path>) -> Self {
        Self::EntryNotFound {
            code: SmeltErrorCode::EntryNotFound,
            path: path.as_ref().display().to_string(),
        }
    }

    pub fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            code: SmeltErrorCode::Io,
            path: path.as_ref().display().to_string(),
            source,
        }
    }

    pub fn parse(path: impl AsRef<Path>, message: impl Into<String>) -> Self {
        Self::Parse {
            code: SmeltErrorCode::Parse,
            path: path.as_ref().display().to_string(),
            message: message.into(),
        }
    }

    pub fn transpile(path: impl AsRef<Path>, message: impl Into<String>) -> Self {
        Self::Transpile {
            code: SmeltErrorCode::Transpile,
            path: path.as_ref().display().to_string(),
            message: message.into(),
        }
    }

    pub fn unresolved_import(specifier: impl Into<String>, referrer: impl AsRef<Path>) -> Self {
        Self::UnresolvedImport {
            code: SmeltErrorCode::UnresolvedImport,
            specifier: specifier.into(),
            referrer: referrer.as_ref().display().to_string(),
        }
    }

    pub fn write(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Write {
            code: SmeltErrorCode::Write,
            path: path.as_ref().display().to_string(),
            source,
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: SmeltErrorCode::InvalidInput,
            message: message.into(),
        }
    }
}

/// Result alias for smelt operations.
pub type SmeltResult<T> = Result<T, SmeltError>;
