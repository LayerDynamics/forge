//! TypeScript → JavaScript transpile with relative-specifier rewriting.
//!
//! Each module is transpiled with the shared `forge-weld` transpile path. Before
//! transpiling, the module's relative import specifiers that point at compiled
//! TypeScript modules are rewritten to their `.js` output form (`./util.ts` and
//! `./util` → `./util.js`). `runtime:*`, bare, URL, and verbatim-asset
//! specifiers are left untouched. Without this rewrite the emitted `.js` would
//! still import `./util.ts`, which does not exist in the compiled tree — the
//! silent-breakage failure mode this module exists to prevent.

use std::ops::Range;

use forge_weld::{transpile_ts_with, TranspileSettings};

use crate::parse::{is_ts_family, DepKind, ModuleNode};
use crate::{SmeltError, SmeltResult};

/// Transpile one module to JavaScript with its relative specifiers rewritten.
pub fn transpile_module(module: &ModuleNode) -> SmeltResult<String> {
    let rewritten = rewrite_specifiers(module);

    // The specifier is only used for diagnostics / source-map naming; normalize
    // to forward slashes so it is a valid `file://` URL on every platform.
    let rel = module.rel_path.to_string_lossy().replace('\\', "/");
    let specifier = format!("file:///{rel}");

    let out = transpile_ts_with(&rewritten, &specifier, &TranspileSettings::default())
        .map_err(|e| SmeltError::transpile(&module.path, e.to_string()))?;
    Ok(out.code)
}

/// Apply the relative-specifier rewrites to a module's source text, returning
/// the edited TypeScript ready to transpile.
///
/// Edits are applied from the highest start offset down so earlier byte ranges
/// stay valid as later ones change length.
pub fn rewrite_specifiers(module: &ModuleNode) -> String {
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();

    for dep in &module.deps {
        // Only relative specifiers that resolve to a compiled TS module are
        // rewritten; verbatim assets and external specifiers stay as written.
        if let DepKind::Relative(target) = &dep.kind {
            if is_ts_family(target) {
                let new_spec = rewrite_to_js(&dep.specifier, target);
                if new_spec != dep.specifier {
                    edits.push((dep.value_range.clone(), new_spec));
                }
            }
        }
        }
    }

    if edits.is_empty() {
        return module.source.clone();
    }

    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut source = module.source.clone();
    for (range, replacement) in edits {
        source.replace_range(range, &replacement);
    }
    source
}

/// Map a relative TypeScript specifier to its compiled `.js` form.
fn rewrite_to_js(spec: &str) -> String {
    for ext in [".ts", ".tsx", ".mts", ".cts", ".jsx"] {
        if let Some(stem) = spec.strip_suffix(ext) {
            return format!("{stem}.js");
        }
    }
    if spec.ends_with(".js") || spec.ends_with(".mjs") || spec.ends_with(".cjs") {
        return spec.to_string();
    }
    // Extensionless relative import resolves to a compiled `.js` output.
    format!("{spec}.js")
}

#[cfg(test)]
mod tests {
    use super::rewrite_to_js;

    #[test]
    fn rewrites_ts_family_to_js() {
        assert_eq!(rewrite_to_js("./util.ts"), "./util.js");
        assert_eq!(rewrite_to_js("../lib/util.tsx"), "../lib/util.js");
        assert_eq!(rewrite_to_js("./util.mts"), "./util.js");
        assert_eq!(rewrite_to_js("./util.cts"), "./util.js");
    }

    #[test]
    fn extensionless_gets_js() {
        assert_eq!(rewrite_to_js("./util"), "./util.js");
        assert_eq!(rewrite_to_js("../a/b/c"), "../a/b/c.js");
    }

    #[test]
    fn already_js_unchanged() {
        assert_eq!(rewrite_to_js("./util.js"), "./util.js");
        assert_eq!(rewrite_to_js("./util.mjs"), "./util.mjs");
    }
}
