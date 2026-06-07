//! Rule `api-drift`: keep `api/runtime-<mod>.md` in sync with `sdk/runtime.<mod>.ts`.
//!
//! Bidirectional, because the two directions catch different failures:
//! - **forward** (export not documented) caught `runtime:dock` missing
//!   `info`/`nextMenuEvent`/`onMenuItemClick` after the M5 landing;
//! - **reverse** (documented method no longer exported) caught `runtime:shell`
//!   still documenting `setCwd()` after it was renamed to `chdir()`.
//!
//! Only modules that already have an API page are checked here. Whether every
//! SDK module *should* have a page is a generation/policy concern (Phase 2),
//! not drift — flagging intentionally-internal modules would make CI noisy.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;
use regex::Regex;

/// Generic hook/handler plumbing emitted into every extensibility-enabled SDK
/// module. These are not part of a module's documented surface, so neither
/// direction should require them.
const HOOK_PLUMBING: &[&str] = &[
    "invokeHandler",
    "hasHandler",
    "listHandlers",
    "onAfter",
    "onBefore",
    "onError",
    "registerHandler",
    "removeAllHooks",
    "removeHandler",
];

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let api_dir = ws.docs_dir().join("api");
    // `export function foo`, `export async function foo`, `export async function* foo`.
    let export_fn_re =
        Regex::new(r"(?m)^\s*export\s+(?:async\s+)?function\s*\*?\s*([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid export-fn regex");
    // Re-export lists: `export { a, b as c };` (but NOT `export type { ... }`,
    // which re-exports types, not callable surface). Modules like runtime:timers
    // declare `function foo()` then `export { foo }` at the bottom.
    let export_list_re =
        Regex::new(r"(?m)^\s*export\s*\{([^}]*)\}").expect("valid export-list regex");
    // Method headings: `### foo(` / `#### foo(` with NO space before `(`, so concept
    // headings like `### Architecture (` arch `)` are not mistaken for functions.
    let heading_re =
        Regex::new(r"(?m)^#{3,4}\s+([A-Za-z_][A-Za-z0-9_]*)\(").expect("valid heading regex");

    let mut findings = Vec::new();
    for module in &ws.sdk_modules {
        let page = api_dir.join(format!("{}.md", module.api_page_stem()));
        let doc = match read_optional(&page) {
            Some(d) => d,
            // No page → not this rule's concern (see module docs).
            None => continue,
        };
        let sdk = match read_optional(&module.path) {
            Some(s) => s,
            None => continue,
        };

        // `declared` = functions declared with `export function ...`. These are a
        // module's primary documented surface, so the FORWARD check requires each
        // to be documented.
        let mut declared: Vec<String> = export_fn_re
            .captures_iter(&sdk)
            .map(|c| c[1].to_string())
            .filter(|name| !HOOK_PLUMBING.contains(&name.as_str()))
            .collect();
        declared.sort();
        declared.dedup();

        // `all_exports` additionally includes names from `export { a, b as c }`
        // re-export lists — both the source (`a`) and the alias (`c`). These are
        // aliases / re-exports of already-declared functions, so they do NOT
        // independently require documentation, but they ARE valid export names,
        // so the REVERSE check accepts a documented heading that matches any of
        // them (e.g. `setTimeout`, `webviewNew`, `delete_`).
        let mut all_exports = declared.clone();
        for cap in export_list_re.captures_iter(&sdk) {
            // Strip `// line comments` per line first, so a comment line
            // (`// primary names`) does not swallow the name on the next line.
            let cleaned: String = cap[1]
                .lines()
                .map(|line| match line.find("//") {
                    Some(idx) => &line[..idx],
                    None => line,
                })
                .collect::<Vec<_>>()
                .join("\n");
            for item in cleaned.split(',') {
                // Each `X as Y` contributes both X and Y as valid export names.
                for part in item.split(" as ") {
                    let name = part.trim().trim_end_matches(',').trim();
                    if !name.is_empty()
                        && name
                            .chars()
                            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                        && !HOOK_PLUMBING.contains(&name)
                    {
                        all_exports.push(name.to_string());
                    }
                }
            }
        }
        all_exports.sort();
        all_exports.dedup();

        // Forward: every declared public function is mentioned somewhere in the page.
        for name in &declared {
            if !mentions_identifier(&doc, name) {
                findings.push(Finding::new(
                    "api-drift",
                    format!(
                        "runtime:{}: SDK exports `{}` but {}.md does not document it",
                        module.name,
                        name,
                        module.api_page_stem()
                    ),
                ));
            }
        }

        // Reverse: every documented method heading still exists as some export
        // (declared, re-exported, or alias).
        for cap in heading_re.captures_iter(&doc) {
            let documented = cap[1].to_string();
            if !all_exports.contains(&documented) && !HOOK_PLUMBING.contains(&documented.as_str()) {
                findings.push(Finding::new(
                    "api-drift",
                    format!(
                        "runtime:{}: {}.md documents `{}()` but it is no longer an SDK export (renamed or removed?)",
                        module.name,
                        module.api_page_stem(),
                        documented
                    ),
                ));
            }
        }
    }
    findings
}

/// True if `name` appears in `text` as a whole identifier (not as a substring of
/// a longer identifier). Avoids `setIcon` matching inside `setIconBadge`.
fn mentions_identifier(text: &str, name: &str) -> bool {
    let bytes = text.as_bytes();
    let name_bytes = name.as_bytes();
    let mut from = 0;
    while let Some(rel) = text[from..].find(name) {
        let start = from + rel;
        let end = start + name_bytes.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}
