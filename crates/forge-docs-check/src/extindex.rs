//! Rule `ext-index`: every `crates/ext_*` extension is listed in the
//! `architecture.md` "Extension Crates" overview table.
//!
//! This is the index/module-list guard (Phase 2.5). It is a **completeness
//! check**, not a generator: the table's `Purpose` column is hand-authored prose
//! and only 13/37 crates carry a `Cargo.toml` description, so generating that
//! column would either fabricate text or replace good labels with inconsistent
//! doc-first-lines. Instead the rule ensures the authored table can never go
//! silently incomplete — which it had (it listed 27 of 37 extensions, missing
//! `console`, `dock`, `encoding`, `image_tools`, `svelte`, `web_inspector`,
//! `codesign`, …). Adding a new extension now requires adding its row, the same
//! way `crate-page` requires its dedicated page.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;

/// The overview page whose extension table must list every extension crate.
const INDEX_PAGE: &str = "architecture.md";

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let page = match read_optional(&ws.docs_dir().join(INDEX_PAGE)) {
        Some(p) => p,
        None => return Vec::new(),
    };
    let mut findings = Vec::new();
    for krate in ws.extension_crates() {
        // The table references each extension by its crate name in a code span,
        // e.g. `| `ext_console` | … |`. Require that exact token to be present.
        let needle = format!("`{}`", krate.dir_name);
        if !page.contains(&needle) {
            findings.push(Finding::new(
                "ext-index",
                format!(
                    "extension crate `{}` is not listed in {}'s extension overview table",
                    krate.dir_name, INDEX_PAGE
                ),
            ));
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn ws_with(arch: &str, ext_crates: &[&str]) -> (tempfile::TempDir, Workspace) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("sdk")).unwrap();
        fs::create_dir_all(root.join("site/src/content/docs")).unwrap();
        fs::write(root.join("site/src/content/docs/architecture.md"), arch).unwrap();
        let mut members = String::new();
        for c in ext_crates {
            let dir = root.join("crates").join(c);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("Cargo.toml"),
                format!("[package]\nname = \"{c}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
            )
            .unwrap();
            members.push_str(&format!("  \"crates/{c}\",\n"));
        }
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nmembers = [\n{members}]\nresolver = \"2\"\n"),
        )
        .unwrap();
        let ws = Workspace::discover_at(root).unwrap();
        (tmp, ws)
    }

    #[test]
    fn flags_extension_missing_from_table() {
        let arch = "| `ext_fs` | runtime:fs | files |\n";
        let (_t, ws) = ws_with(arch, &["ext_fs", "ext_console"]);
        let findings = check(&ws);
        assert!(
            findings.iter().any(|f| f.message.contains("ext_console")),
            "missing ext_console must be flagged: {:?}",
            findings.iter().map(|f| &f.message).collect::<Vec<_>>()
        );
        assert!(!findings.iter().any(|f| f.message.contains("ext_fs")));
    }

    #[test]
    fn passes_when_all_listed() {
        let arch = "| `ext_fs` | x |\n| `ext_console` | y |\n";
        let (_t, ws) = ws_with(arch, &["ext_fs", "ext_console"]);
        assert!(check(&ws).is_empty());
    }
}
