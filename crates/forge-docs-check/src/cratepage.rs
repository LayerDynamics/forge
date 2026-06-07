//! Crate-page generator (gap-fill).
//!
//! Generates a Starlight page under `site/src/content/docs/crates/<stem>.md` for
//! any workspace crate that does not have one yet, from that crate's source:
//! the `//!` module doc becomes the page body and `Cargo.toml`'s `description`
//! (or the doc's first line) becomes the frontmatter description.
//!
//! It is deliberately **gap-fill only** — it never overwrites an existing page.
//! The rich, hand-authored crate pages (e.g. `ext-fs.md`) are left untouched;
//! this only closes the "crate has no page" drift the `crate-page` rule reports.
//! Rustdoc intra-doc links (`[`x`](module::path)`) are converted to plain code
//! text so they don't render as broken links on the web.

use crate::checks::read_optional;
use crate::discovery::{CrateInfo, Workspace};
use regex::Regex;

/// Extract the leading `//!` module doc block from Rust source, stripped of the
/// `//!` prefix. Returns `None` if there is no module doc.
pub fn extract_module_doc(src: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut started = false;
    for line in src.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            started = true;
            lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        } else if started {
            break; // the contiguous //! block has ended
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n").trim().to_string())
}

/// Convert rustdoc intra-doc links to plain text, preserving real web links.
/// `[`op_console_push`](console::op_console_push)` -> `` `op_console_push` ``;
/// `[Guide](https://…)` and `[x](/docs/…)` and `[x](#anchor)` are kept.
pub fn strip_intra_doc_links(md: &str) -> String {
    let re = Regex::new(r"\[([^\]]+)\]\(([^)]*)\)").expect("valid md-link regex");
    re.replace_all(md, |caps: &regex::Captures| {
        let text = &caps[1];
        let target = &caps[2];
        let is_web = target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with('/')
            || target.starts_with('#');
        if is_web {
            caps[0].to_string()
        } else {
            text.to_string()
        }
    })
    .into_owned()
}

/// Parse `[package].description` from a crate's `Cargo.toml`.
fn cargo_description(cargo_toml: &str) -> Option<String> {
    let value: toml::Value = toml::from_str(cargo_toml).ok()?;
    value
        .get("package")?
        .get("description")?
        .as_str()
        .map(|s| s.to_string())
}

/// First sentence of the doc body, cleaned of markdown heading markers and
/// backticks — used as a fallback description when `Cargo.toml` has none.
fn description_from_doc(doc: &str) -> Option<String> {
    // First non-empty line that is not a markdown heading.
    let first = doc
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))?;
    let sentence = first.split(". ").next().unwrap_or(first);
    let cleaned = sentence
        .replace('`', "")
        .trim()
        .trim_end_matches('.')
        .to_string();
    (!cleaned.is_empty()).then_some(cleaned)
}

/// Read a crate's primary Rust source (`src/lib.rs`, falling back to `src/main.rs`).
fn crate_source(krate: &CrateInfo) -> Option<String> {
    read_optional(&krate.path.join("src/lib.rs"))
        .or_else(|| read_optional(&krate.path.join("src/main.rs")))
}

/// Render a Starlight crate page (frontmatter + the module-doc body).
pub fn render_crate_page(krate: &CrateInfo, description: &str, body: &str) -> String {
    let stem = krate.crate_page_stem();
    format!(
        "---\ntitle: \"{}\"\ndescription: \"{}\"\nslug: crates/{}\n---\n\n{}\n",
        krate.package_name,
        description.replace('"', "\\\""),
        stem,
        body.trim()
    )
}

/// Generate a page for every crate that lacks one. Returns the written paths.
/// Never overwrites an existing page.
pub fn write_missing(ws: &Workspace) -> std::io::Result<Vec<std::path::PathBuf>> {
    let crates_docs = ws.docs_dir().join("crates");
    let mut written = Vec::new();
    for krate in &ws.crates {
        let stem = krate.crate_page_stem();
        let page_path = crates_docs.join(format!("{stem}.md"));
        if page_path.exists() {
            continue; // gap-fill only — never clobber an authored page
        }
        let src = match crate_source(krate) {
            Some(s) => s,
            None => continue,
        };
        let doc = match extract_module_doc(&src) {
            Some(d) => d,
            None => continue,
        };
        let body = strip_intra_doc_links(&doc);
        let description = read_optional(&krate.path.join("Cargo.toml"))
            .and_then(|c| cargo_description(&c))
            .or_else(|| description_from_doc(&body))
            .unwrap_or_else(|| format!("The {} crate.", krate.package_name));

        std::fs::create_dir_all(&crates_docs)?;
        std::fs::write(&page_path, render_crate_page(krate, &description, &body))?;
        written.push(page_path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_module_doc_collects_contiguous_block() {
        let src =
            "#![allow(dead_code)]\n//! Line one.\n//!\n//! Line two.\nuse std::fs;\n//! not part\n";
        assert_eq!(
            extract_module_doc(src).as_deref(),
            Some("Line one.\n\nLine two.")
        );
    }

    #[test]
    fn strip_intra_doc_links_keeps_web_links_only() {
        let md = "see [`op_push`](console::op_push) and [Guide](https://x.io) and [api](/docs/api)";
        assert_eq!(
            strip_intra_doc_links(md),
            "see `op_push` and [Guide](https://x.io) and [api](/docs/api)"
        );
    }

    #[test]
    fn description_falls_back_to_first_doc_line() {
        let body = "# Title\n\nConsole capture extension for Forge (`runtime:console`). More.";
        assert_eq!(
            description_from_doc(body).as_deref(),
            Some("Console capture extension for Forge (runtime:console)")
        );
    }
}
