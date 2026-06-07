//! Marker-hybrid API blocks.
//!
//! The design decision behind this (recorded in
//! `docs/plans/2026-06-06-19-docs-autogeneration-pipeline.md`): the rich API
//! pages are 400–600 lines of bespoke hand-authored prose that cannot live in
//! doc-comments or be reproduced by a templated generator. So instead of
//! generating whole pages, a page opts in by embedding a single marker region
//! under its `## API Reference` heading:
//!
//!   - an opening `<!-- forge:api -->` marker,
//!   - a generated fenced `typescript` list of the module's public signatures
//!     (e.g. `info(): OsInfo` / `pathSep(): string`),
//!   - a closing `<!-- /forge:api -->` marker.
//!
//! Everything outside the markers — the bespoke `### info()` prose, examples,
//! tables — is authored and never touched.
//!
//! The block holds the current public signatures of the module's
//! `sdk/runtime.<mod>.ts` (excluding hook plumbing). [`check`] fails when a block
//! is stale; [`write_all`] regenerates every block in place from the SDK.

use crate::checks::{read_optional, HOOK_PLUMBING};
use crate::discovery::Workspace;
use crate::Finding;
use regex::Regex;

pub const BLOCK_OPEN: &str = "<!-- forge:api -->";
pub const BLOCK_CLOSE: &str = "<!-- /forge:api -->";

/// Extract the public function signatures from an SDK module's source, in
/// declaration order, excluding hook/handler plumbing. A signature is the text
/// from the function name up to the body brace, whitespace-collapsed
/// (e.g. `info(): OsInfo`).
pub fn public_signatures(sdk_src: &str) -> Vec<String> {
    let name_re = Regex::new(r"export\s+(?:async\s+)?function\s+([A-Za-z0-9_]+)")
        .expect("valid export-fn regex");
    let mut sigs = Vec::new();
    for cap in name_re.captures_iter(sdk_src) {
        let name = &cap[1];
        if HOOK_PLUMBING.contains(&name) {
            continue;
        }
        let name_start = cap.get(1).expect("name group").start();
        // Signature runs from the name up to the first `{` (the body opener).
        if let Some(brace_rel) = sdk_src[name_start..].find('{') {
            let raw = &sdk_src[name_start..name_start + brace_rel];
            let sig = raw.split_whitespace().collect::<Vec<_>>().join(" ");
            let sig = sig.trim().to_string();
            if !sig.is_empty() {
                sigs.push(sig);
            }
        }
    }
    sigs
}

/// Render the body that belongs *between* the markers (a fenced TypeScript
/// signature list with a generated-by notice).
pub fn render_block_body(module: &str, signatures: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "<!-- generated from sdk/runtime.{module}.ts — edit signatures in the SDK, run `make docs-api` to refresh -->\n"
    ));
    out.push_str("```typescript\n");
    for sig in signatures {
        out.push_str(sig);
        out.push('\n');
    }
    out.push_str("```");
    out
}

/// Locate a marker region in `page`. Returns `(open_start, close_end, body)`
/// where `body` is the exact text strictly between the open and close markers
/// (trimmed of the single surrounding newlines).
fn find_block(page: &str) -> Option<(usize, usize, String)> {
    let open = page.find(BLOCK_OPEN)?;
    let after_open = open + BLOCK_OPEN.len();
    let close_rel = page[after_open..].find(BLOCK_CLOSE)?;
    let close = after_open + close_rel;
    let close_end = close + BLOCK_CLOSE.len();
    let body = page[after_open..close].trim_matches('\n').to_string();
    Some((open, close_end, body))
}

/// Rule `api-block`: every page that opts in (contains `<!-- forge:api -->`) has
/// a block whose body matches the current SDK signatures.
pub fn check(ws: &Workspace) -> Vec<Finding> {
    let api_dir = ws.docs_dir().join("api");
    let mut findings = Vec::new();
    for module in &ws.sdk_modules {
        let page_path = api_dir.join(format!("{}.md", module.api_page_stem()));
        let page = match read_optional(&page_path) {
            Some(p) => p,
            None => continue,
        };
        if !page.contains(BLOCK_OPEN) {
            continue; // not opted in
        }
        let sdk = match read_optional(&module.path) {
            Some(s) => s,
            None => continue,
        };
        let expected = render_block_body(&module.name, &public_signatures(&sdk));
        match find_block(&page) {
            Some((_, _, body)) if body == expected => {}
            Some((_, _, _)) => findings.push(Finding::new(
                "api-block",
                format!(
                    "runtime:{}: the <!-- forge:api --> block in {}.md is stale; run `make docs-api` to refresh it",
                    module.name,
                    module.api_page_stem()
                ),
            )),
            None => findings.push(Finding::new(
                "api-block",
                format!(
                    "runtime:{}: {}.md has an opening <!-- forge:api --> with no closing <!-- /forge:api -->",
                    module.name,
                    module.api_page_stem()
                ),
            )),
        }
    }
    findings
}

/// Regenerate every opted-in block in place from the SDK. Returns the list of
/// page paths that were rewritten (only pages whose block actually changed).
pub fn write_all(ws: &Workspace) -> std::io::Result<Vec<std::path::PathBuf>> {
    let api_dir = ws.docs_dir().join("api");
    let mut written = Vec::new();
    for module in &ws.sdk_modules {
        let page_path = api_dir.join(format!("{}.md", module.api_page_stem()));
        let page = match read_optional(&page_path) {
            Some(p) => p,
            None => continue,
        };
        if !page.contains(BLOCK_OPEN) {
            continue;
        }
        let sdk = match read_optional(&module.path) {
            Some(s) => s,
            None => continue,
        };
        let expected = render_block_body(&module.name, &public_signatures(&sdk));
        if let Some((open, close_end, body)) = find_block(&page) {
            if body == expected {
                continue;
            }
            let mut updated = String::with_capacity(page.len());
            updated.push_str(&page[..open]);
            updated.push_str(BLOCK_OPEN);
            updated.push('\n');
            updated.push_str(&expected);
            updated.push('\n');
            updated.push_str(BLOCK_CLOSE);
            updated.push_str(&page[close_end..]);
            std::fs::write(&page_path, updated)?;
            written.push(page_path);
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_exclude_hook_plumbing() {
        let sdk = r#"
export function info(): OsInfo { return ops.info(); }
export function pathSep(): string { return ops.pathSep(); }
export function onBefore<T extends OpName>(x: T): void {}
export async function invokeHandler(name: string): Promise<unknown> {}
"#;
        assert_eq!(
            public_signatures(sdk),
            vec![
                "info(): OsInfo".to_string(),
                "pathSep(): string".to_string()
            ]
        );
    }

    #[test]
    fn find_block_extracts_body() {
        let page = "intro\n<!-- forge:api -->\nBODY\nLINE2\n<!-- /forge:api -->\noutro\n";
        let (_, _, body) = find_block(page).expect("block present");
        assert_eq!(body, "BODY\nLINE2");
    }

    #[test]
    fn render_block_body_is_deterministic() {
        let sigs = vec!["info(): OsInfo".to_string()];
        let a = render_block_body("os_compat", &sigs);
        let b = render_block_body("os_compat", &sigs);
        assert_eq!(a, b);
        assert!(a.contains("```typescript"));
        assert!(a.contains("info(): OsInfo"));
    }
}
