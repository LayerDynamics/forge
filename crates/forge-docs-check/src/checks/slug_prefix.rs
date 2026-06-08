//! Rule `slug-prefix`: every docs page is mounted under `/docs/`.
//!
//! The site serves a marketing landing page at `/` (`site/src/pages/index.astro`)
//! and the Starlight docs under `/docs/`. Starlight derives each page's URL from
//! its `slug:` frontmatter, so a page whose slug lacks the `docs/` prefix would
//! render at the site root (e.g. `/crates/forge/`) — and every authored
//! `](/docs/crates/forge)` link to it would 404.
//!
//! This rule asserts every content page's slug is under `docs/`, so a generator
//! (e.g. [`crate::cratepage`]) or a hand edit that emits a root-level slug fails
//! CI instead of silently breaking site navigation.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;
use std::path::{Path, PathBuf};

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let mut findings = Vec::new();
    for page in markdown_pages(&ws.docs_dir()) {
        let src = match read_optional(&page) {
            Some(s) => s,
            None => continue,
        };
        let slug = match frontmatter_slug(&src) {
            Some(s) => s,
            None => continue, // no explicit slug -> path-derived; not this rule's concern
        };
        if slug != "docs" && !slug.starts_with("docs/") {
            let rel = page
                .strip_prefix(&ws.root)
                .unwrap_or(&page)
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            findings.push(Finding::new(
                "slug-prefix",
                format!(
                    "{rel}: slug `{slug}` is not under `docs/` — docs are served at /docs/, so this page (and links to it) would 404; use `slug: docs/{slug}`"
                ),
            ));
        }
    }
    findings
}

/// Extract the value of the frontmatter `slug:` key (the first `slug:` line
/// inside the leading `---` block).
fn frontmatter_slug(src: &str) -> Option<String> {
    let body = src.strip_prefix("---")?;
    let end = body.find("\n---")?;
    for line in body[..end].lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("slug:") {
            return Some(rest.trim().trim_matches(['"', '\'']).to_string());
        }
    }
    None
}

/// Recursively collect `.md`/`.mdx` files under `dir`, sorted for determinism.
fn markdown_pages(dir: &Path) -> Vec<PathBuf> {
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
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("md") | Some("mdx")
            ) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::frontmatter_slug;

    #[test]
    fn extracts_quoted_and_bare_slugs() {
        assert_eq!(
            frontmatter_slug("---\ntitle: x\nslug: docs/crates/forge\n---\nbody"),
            Some("docs/crates/forge".to_string())
        );
        assert_eq!(
            frontmatter_slug("---\nslug: \"crates/forge\"\n---\n"),
            Some("crates/forge".to_string())
        );
        assert_eq!(frontmatter_slug("# no frontmatter\n"), None);
    }
}
