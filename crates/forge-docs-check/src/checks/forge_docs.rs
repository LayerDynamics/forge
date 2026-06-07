//! Rule `forge-docs-list`: regression guard for `forge_cli/src/docs.rs`.
//!
//! Originally this caught the `Site.md` bug where the generator's hardcoded
//! `EXTENSIONS` array had fallen behind the crates on disk. Phase 2 replaced
//! that array with filesystem discovery (`discover_extensions`), so in the
//! normal case there is no array and this rule is inert (returns no findings).
//!
//! It is kept as a guard: if anyone ever re-introduces a static
//! `EXTENSIONS: &[(&str, &str)] = &[ … ]` list in `docs.rs`, this rule fires
//! again unless that list covers every `crates/ext_*` crate — preventing a
//! silent relapse to the stale-list failure mode.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;
use regex::Regex;

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let docs_rs = ws.root.join("crates/forge_cli/src/docs.rs");
    let src = match read_optional(&docs_rs) {
        Some(s) => s,
        // If discovery has already replaced the array (Phase 2), there is nothing
        // to guard and no finding to emit.
        None => return Vec::new(),
    };

    let block = match extract_extensions_block(&src) {
        Some(b) => b,
        // No static EXTENSIONS array → assume discovery is in place (Phase 2 done).
        None => return Vec::new(),
    };

    // Each entry looks like `("fs", "runtime:fs"),` — capture the short name.
    let entry_re = Regex::new(r#"\(\s*"([a-z_]+)"\s*,"#).expect("valid entry regex");
    let listed: Vec<String> = entry_re
        .captures_iter(&block)
        .map(|c| c[1].to_string())
        .collect();

    let mut findings = Vec::new();
    for krate in ws.extension_crates() {
        // `ext_image_tools` -> `image_tools`.
        let short = krate
            .dir_name
            .strip_prefix("ext_")
            .unwrap_or(&krate.dir_name);
        if !listed.iter().any(|n| n == short) {
            findings.push(Finding::new(
                "forge-docs-list",
                format!(
                    "`forge docs` EXTENSIONS array (forge_cli/src/docs.rs) is missing `{short}` (crate `{}` exists). It will be skipped by `forge docs --all-extensions`.",
                    krate.dir_name
                ),
            ));
        }
    }
    findings
}

/// Return the text of the `const EXTENSIONS ... = &[ ... ];` array body.
fn extract_extensions_block(src: &str) -> Option<String> {
    let start = src.find("EXTENSIONS")?;
    // Skip to the assignment first, so the `&[` inside the type annotation
    // `const EXTENSIONS: &[(&str, &str)] = &[ ... ]` is not mistaken for the
    // array literal.
    let eq = src[start..].find('=')? + start;
    let bracket = src[eq..].find("&[")? + eq;
    // Find the matching closing `]` from `&[`.
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut i = bracket + 1; // points at '['
    let open = i;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(src[open + 1..i].to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}
