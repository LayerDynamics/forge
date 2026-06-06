//! Materialize the compiled artifact.
//!
//! Given a discovered [`ModuleGraph`], transpile every TypeScript module, write
//! the resulting `.js` tree to the output directory (mirroring the source
//! layout), and copy any verbatim relative assets (`.js`, `.json`, …) alongside.
//! The returned [`SmeltOutput`] is the descriptor of the finished ingot — what
//! `forge bundle` ships and what a future Depth-2 step would embed.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::parse::{is_ts_family, DepKind, ModuleGraph};
use crate::transpile::transpile_module;
use crate::{SmeltError, SmeltResult};

/// Descriptor of a completed smelt run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmeltOutput {
    /// Directory the compiled tree was written to.
    pub out_dir: PathBuf,
    /// Path to the compiled entry module (e.g. `<out_dir>/main.js`).
    pub entry: PathBuf,
    /// Compiled JS module paths written, in graph-discovery order.
    pub modules: Vec<PathBuf>,
    /// Verbatim (non-TypeScript) asset files copied alongside the JS.
    pub assets: Vec<PathBuf>,
    /// Total bytes of compiled JavaScript written.
    pub bytes_written: u64,
}

/// Compile every module in `graph`, write the `.js` tree to `out_dir`, and copy
/// verbatim relative assets. Returns the [`SmeltOutput`].
pub fn materialize(graph: &ModuleGraph, out_dir: &Path) -> SmeltResult<SmeltOutput> {
    std::fs::create_dir_all(out_dir).map_err(|e| SmeltError::write(out_dir, e))?;

    let mut modules = Vec::new();
    let mut bytes_written: u64 = 0;
    let mut entry: Option<PathBuf> = None;

    for module in &graph.modules {
        let js = transpile_module(module)?;

        let out_rel = with_js_ext(&module.rel_path);
        let out_path = out_dir.join(&out_rel);
        write_file(&out_path, js.as_bytes())?;

        bytes_written += js.len() as u64;
        if module.path == graph.entry {
            entry = Some(out_path.clone());
        }
        modules.push(out_path);
    }

    // Copy verbatim relative assets (non-TS targets such as hand-written `.js`
    // or imported `.json`), de-duplicated across all modules.
    let mut assets = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for module in &graph.modules {
        for dep in &module.deps {
            let DepKind::Relative(target) = &dep.kind else {
                continue;
            };
            if is_ts_family(target) || !seen.insert(target.clone()) {
                continue;
            }
            let rel = target.strip_prefix(&graph.src_root).map_err(|_| {
                SmeltError::invalid_input(format!(
                    "asset '{}' is outside the source root and cannot be smelted",
                    target.display()
                ))
            })?;
            let dest = out_dir.join(rel);
            copy_file(target, &dest)?;
            assets.push(dest);
        }
    }

    let entry = entry
        .ok_or_else(|| SmeltError::invalid_input("entry module was not present in the graph"))?;

    Ok(SmeltOutput {
        out_dir: out_dir.to_path_buf(),
        entry,
        modules,
        assets,
        bytes_written,
    })
}

/// Replace a path's extension with `.js` (e.g. `lib/util.ts` → `lib/util.js`).
fn with_js_ext(rel: &Path) -> PathBuf {
    rel.with_extension("js")
}

/// Write `bytes` to `path`, creating parent directories.
fn write_file(path: &Path, bytes: &[u8]) -> SmeltResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| SmeltError::write(parent, e))?;
    }
    std::fs::write(path, bytes).map_err(|e| SmeltError::write(path, e))
}

/// Copy `src` to `dest`, creating parent directories.
fn copy_file(src: &Path, dest: &Path) -> SmeltResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| SmeltError::write(parent, e))?;
    }
    std::fs::copy(src, dest).map_err(|e| SmeltError::write(dest, e))?;
    Ok(())
}
