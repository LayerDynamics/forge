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
use crate::markers;
use crate::Finding;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub const BLOCK_OPEN: &str = "<!-- forge:api -->";
pub const BLOCK_CLOSE: &str = "<!-- /forge:api -->";

/// Extract the public function signatures from an SDK module's source, in
/// declaration order, excluding hook/handler plumbing. A signature is the text
/// from the function name up to the body brace, whitespace-collapsed
/// (e.g. `info(): OsInfo`).
pub fn public_signatures(sdk_src: &str) -> Vec<String> {
    // 1. Map every declared function name -> the signature tail after the name,
    //    i.e. "(params): ret". Covers both `export function foo` and bare
    //    `function foo` (which modules like runtime:timers/webview declare and
    //    then re-export via `export { foo }`).
    let decl_re =
        Regex::new(r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_]+)")
            .expect("valid decl regex");
    let mut decl_tail: HashMap<String, String> = HashMap::new();
    for cap in decl_re.captures_iter(sdk_src) {
        let name = cap[1].to_string();
        let name_start = cap.get(1).expect("name group").start();
        if let Some(brace_rel) = sdk_src[name_start..].find('{') {
            let collapsed = sdk_src[name_start..name_start + brace_rel]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let tail = collapsed
                .strip_prefix(&name)
                .unwrap_or(&collapsed)
                .to_string();
            decl_tail.entry(name).or_insert(tail);
        }
    }

    // 2. Collect the public surface as (exported_name, source_name) pairs:
    //    `export function foo` (source == export), and `export { a, b as c }`
    //    re-export lists (source is the name before `as`, export is after).
    let mut public: Vec<(String, String)> = Vec::new();
    let exp_fn = Regex::new(r"(?m)^\s*export\s+(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_]+)")
        .expect("valid export-fn regex");
    for cap in exp_fn.captures_iter(sdk_src) {
        let n = cap[1].to_string();
        public.push((n.clone(), n));
    }
    let exp_list = Regex::new(r"(?m)^\s*export\s*\{([^}]*)\}").expect("valid export-list regex");
    for cap in exp_list.captures_iter(sdk_src) {
        let cleaned: String = cap[1]
            .lines()
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        for item in cleaned.split(',') {
            let parts: Vec<&str> = item
                .split(" as ")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.is_empty() {
                continue;
            }
            let source = parts[0].trim_end_matches(',').trim().to_string();
            let exported = parts
                .last()
                .unwrap()
                .trim_end_matches(',')
                .trim()
                .to_string();
            let ident = |s: &str| s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if source.is_empty() || !ident(&source) || !ident(&exported) {
                continue;
            }
            public.push((exported, source));
        }
    }

    // 3. Render `exportedName(params): ret`, excluding hook plumbing, first
    //    occurrence wins, only for names that have a known declaration.
    let mut seen = HashSet::new();
    let mut sigs = Vec::new();
    for (exported, source) in public {
        if HOOK_PLUMBING.contains(&exported.as_str()) || !seen.insert(exported.clone()) {
            continue;
        }
        if let Some(tail) = decl_tail.get(&source) {
            sigs.push(format!("{exported}{tail}"));
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
        match markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
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
        if let Some((_, _, body)) = markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
            if body == expected {
                continue;
            }
            if let Some(updated) =
                markers::replace_region(&page, BLOCK_OPEN, BLOCK_CLOSE, &expected)
            {
                std::fs::write(&page_path, updated)?;
                written.push(page_path);
            }
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
    fn signatures_resolve_reexports_and_aliases() {
        // Declared as bare `function` then re-exported (timers/webview pattern),
        // including an `as` alias.
        let sdk = r#"
function setTimeout(cb: () => void, delay?: number): number { return 0; }
async function execute(cmd: string): Promise<string> { return ""; }
export { setTimeout };
export { execute as exec };
"#;
        assert_eq!(
            public_signatures(sdk),
            vec![
                "setTimeout(cb: () => void, delay?: number): number".to_string(),
                // alias `exec` rendered with execute's parameters
                "exec(cmd: string): Promise<string>".to_string(),
            ]
        );
    }

    #[test]
    fn public_signatures_is_line_ending_agnostic() {
        // Same source, LF vs CRLF, must yield identical signatures — otherwise a
        // Windows checkout would generate a different block than the gate expects.
        let lf = "export function info(): OsInfo { return x; }\n\
                  export function pathSep(): string { return y; }\n";
        let crlf = lf.replace('\n', "\r\n");
        assert_eq!(public_signatures(lf), public_signatures(&crlf));
        assert!(!public_signatures(&crlf).is_empty());
    }

    #[test]
    fn whole_block_round_trips_through_crlf() {
        // Render an LF block, simulate a CRLF on-disk copy, and confirm the
        // extracted body still equals the freshly-rendered (LF) expected body.
        let sigs = vec![
            "info(): OsInfo".to_string(),
            "pathSep(): string".to_string(),
        ];
        let expected = render_block_body("os_compat", &sigs);
        let on_disk = format!(
            "# page\r\n\r\n{BLOCK_OPEN}\r\n{}\r\n{BLOCK_CLOSE}\r\n\r\n## next\r\n",
            expected.replace('\n', "\r\n")
        );
        let (_, _, body) =
            markers::find_region(&on_disk, BLOCK_OPEN, BLOCK_CLOSE).expect("block present");
        assert_eq!(body, expected, "CRLF on-disk block must match LF expected");
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
