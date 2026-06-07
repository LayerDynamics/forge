//! Marker-hybrid example blocks (`<!-- forge:example -->`).
//!
//! Each example app under `examples/<name>/` imports a set of `runtime:*`
//! modules in its `src/`. The matching docs page (`docs/examples/<name>.md`)
//! can opt in with a `<!-- forge:example -->` block listing those modules; the
//! `example-block` rule fails CI when the block is stale and `make docs-examples`
//! regenerates it from the app source. Prose outside the markers is untouched.
//!
//! This extends drift protection to a real, easily-missed category: an example
//! app gains/loses a `runtime:*` dependency but its doc page is not updated.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::{markers, Finding};
use regex::Regex;
use std::path::{Path, PathBuf};

pub const BLOCK_OPEN: &str = "<!-- forge:example -->";
pub const BLOCK_CLOSE: &str = "<!-- /forge:example -->";

/// Discover example apps: immediate subdirectories of `examples/` that contain a
/// `manifest.app.toml`. Returns `(name, dir)` sorted by name.
pub fn example_apps(ws: &Workspace) -> Vec<(String, PathBuf)> {
    let examples_dir = ws.root.join("examples");
    let mut apps = Vec::new();
    let entries = match std::fs::read_dir(&examples_dir) {
        Ok(e) => e,
        Err(_) => return apps,
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        if dir.is_dir() && dir.join("manifest.app.toml").exists() {
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                apps.push((name.to_string(), dir.clone()));
            }
        }
    }
    apps.sort_by(|a, b| a.0.cmp(&b.0));
    apps
}

/// The sorted, unique `runtime:*` modules imported anywhere under `<app>/src`.
pub fn runtime_modules(app_dir: &Path) -> Vec<String> {
    let re = Regex::new(r"runtime:([a-z_]+)").expect("valid runtime-module regex");
    let mut mods: Vec<String> = Vec::new();
    for ts in ts_sources(&app_dir.join("src")) {
        if let Some(src) = read_optional(&ts) {
            for cap in re.captures_iter(&src) {
                mods.push(format!("runtime:{}", &cap[1]));
            }
        }
    }
    mods.sort();
    mods.dedup();
    mods
}

/// Render the body between the markers.
pub fn render_block_body(name: &str, modules: &[String]) -> String {
    let list = if modules.is_empty() {
        "_none_".to_string()
    } else {
        modules
            .iter()
            .map(|m| format!("`{m}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "<!-- generated from examples/{name}/src — run `make docs-examples` to refresh -->\n**Runtime modules used:** {list}",
    )
}

/// The docs page path for an example app.
fn page_path(ws: &Workspace, name: &str) -> PathBuf {
    ws.docs_dir().join("examples").join(format!("{name}.md"))
}

/// Rule `example-block`: every opted-in example page's block matches the modules
/// its app actually imports.
pub fn check(ws: &Workspace) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (name, dir) in example_apps(ws) {
        let page = match read_optional(&page_path(ws, &name)) {
            Some(p) => p,
            None => continue,
        };
        if !page.contains(BLOCK_OPEN) {
            continue; // not opted in
        }
        let expected = render_block_body(&name, &runtime_modules(&dir));
        match markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
            Some((_, _, body)) if body == expected => {}
            Some(_) => findings.push(Finding::new(
                "example-block",
                format!(
                    "examples/{name}: the <!-- forge:example --> block in {name}.md is stale; run `make docs-examples` to refresh it"
                ),
            )),
            None => findings.push(Finding::new(
                "example-block",
                format!("examples/{name}: {name}.md has an opening <!-- forge:example --> with no closing marker"),
            )),
        }
    }
    findings
}

/// Regenerate every opted-in example block in place. Returns rewritten paths.
pub fn write_all(ws: &Workspace) -> std::io::Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    for (name, dir) in example_apps(ws) {
        let path = page_path(ws, &name);
        let page = match read_optional(&path) {
            Some(p) => p,
            None => continue,
        };
        if !page.contains(BLOCK_OPEN) {
            continue;
        }
        let expected = render_block_body(&name, &runtime_modules(&dir));
        if let Some((_, _, body)) = markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
            if body == expected {
                continue;
            }
            if let Some(updated) =
                markers::replace_region(&page, BLOCK_OPEN, BLOCK_CLOSE, &expected)
            {
                std::fs::write(&path, updated)?;
                written.push(path);
            }
        }
    }
    Ok(written)
}

/// Recursively collect `.ts` files under `dir`.
fn ts_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = match std::fs::read_dir(&d) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("ts") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_block_body_lists_modules() {
        let body = render_block_body(
            "react-app",
            &["runtime:ipc".to_string(), "runtime:window".to_string()],
        );
        assert!(body.contains("**Runtime modules used:** `runtime:ipc`, `runtime:window`"));
        assert!(body.contains("examples/react-app/src"));
    }

    #[test]
    fn render_block_body_handles_no_modules() {
        assert!(render_block_body("x", &[]).contains("_none_"));
    }
}
