//! Rule `count`: numbers in the docs match what the workspace actually contains.
//!
//! Two mechanisms, by design:
//! 1. **Markers** (authoritative, post Phase 3): `<!-- forge:count:KEY -->N<!-- /forge:count -->`
//!    regions anywhere in the docs are asserted equal to the derived value for KEY.
//!    The generator writes these, so once they exist the check is exact and noise-free.
//! 2. **Interim freeform scan** (until markers land): the three narrative pages the
//!    `Site.md` audit flagged — architecture/internals/roadmap — are scanned for the
//!    specific count phrases ("N crates", "N extensions", "N runtime modules") and any
//!    captured integer that disagrees with the derived value is flagged. Scoped tightly
//!    to those phrases/files so legitimate prose is not caught.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;
use regex::Regex;
use std::collections::HashMap;

/// Derive the authoritative counts from the workspace inventory.
fn derived_counts(ws: &Workspace) -> HashMap<&'static str, usize> {
    let mut m = HashMap::new();
    m.insert("ext_crates", ws.extension_crates().len());
    m.insert("total_crates", ws.crates.len());
    m.insert("runtime_modules", ws.sdk_modules.len());
    m
}

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let counts = derived_counts(ws);
    let mut findings = Vec::new();
    findings.extend(check_markers(ws, &counts));
    findings.extend(check_freeform(ws, &counts));
    findings
}

/// Assert every `<!-- forge:count:KEY -->N<!-- /forge:count -->` region across the
/// docs tree equals the derived value for KEY.
fn check_markers(ws: &Workspace, counts: &HashMap<&'static str, usize>) -> Vec<Finding> {
    let marker_re =
        Regex::new(r"<!--\s*forge:count:([a-z_]+)\s*-->\s*(\d+)\s*<!--\s*/forge:count\s*-->")
            .expect("valid marker regex");

    let mut findings = Vec::new();
    let docs_dir = ws.docs_dir();
    for entry in walk_markdown(&docs_dir) {
        let content = match read_optional(&entry) {
            Some(c) => c,
            None => continue,
        };
        for cap in marker_re.captures_iter(&content) {
            let key = cap[1].to_string();
            let found: usize = cap[2].parse().unwrap_or(usize::MAX);
            match counts.get(key.as_str()) {
                Some(&expected) if found != expected => findings.push(Finding::new(
                    "count",
                    format!(
                        "{}: count marker `{}` says {} but the workspace has {}",
                        rel(ws, &entry),
                        key,
                        found,
                        expected
                    ),
                )),
                None => findings.push(Finding::new(
                    "count",
                    format!(
                        "{}: count marker `{}` is not a known derived count (known: ext_crates, total_crates, runtime_modules)",
                        rel(ws, &entry),
                        key
                    ),
                )),
                _ => {}
            }
        }
    }
    findings
}

/// Interim, marker-free scan of the three audited narrative pages.
fn check_freeform(ws: &Workspace, counts: &HashMap<&'static str, usize>) -> Vec<Finding> {
    // (filename, count-key, phrase regex with the integer in capture group 1).
    // `\d+\+?` tolerates an optional trailing `+` ("30+ crates") but still flags
    // the integer when it disagrees with the true current count.
    let specs: &[(&str, &str, &str)] = &[
        ("architecture.md", "total_crates", r"(\d+)\+?\s+crates\b"),
        (
            "architecture.md",
            "ext_crates",
            r"(\d+)\+?\s+(?:runtime\s+)?extension\s+crates?\b",
        ),
        (
            "architecture.md",
            "runtime_modules",
            r"(\d+)\+?\s+runtime\s+modules?\b",
        ),
        ("internals.md", "ext_crates", r"(\d+)\+?\s+extensions?\b"),
        (
            "roadmap.md",
            "ext_crates",
            r"(\d+)\+?\s+implemented\s+extension\s+modules?\b",
        ),
    ];

    let mut findings = Vec::new();
    for (file, key, pattern) in specs {
        let path = ws.docs_dir().join(file);
        let content = match read_optional(&path) {
            Some(c) => c,
            None => continue,
        };
        // If the page already uses markers for this content, skip the freeform
        // scan for it — markers are authoritative and avoid double-reporting.
        if content.contains(&format!("forge:count:{key}")) {
            continue;
        }
        let re = Regex::new(pattern).expect("valid freeform count regex");
        let expected = counts[key];
        for cap in re.captures_iter(&content) {
            let found: usize = cap[1].parse().unwrap_or(usize::MAX);
            if found != expected {
                findings.push(Finding::new(
                    "count",
                    format!(
                        "{}: text says \"{}\" but the workspace has {} {} (add a <!-- forge:count:{} --> marker to make this exact)",
                        file,
                        cap[0].trim(),
                        expected,
                        key,
                        key
                    ),
                ));
            }
        }
    }
    findings
}

/// Recursively collect `.md` files under `dir`.
fn walk_markdown(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
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
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Workspace-relative path as a string with forward slashes on every platform.
/// Findings are compared against a checked-in baseline; using the OS-native
/// separator (`\` on Windows) would make the same drift produce a different
/// message on Windows and spuriously fail the gate there.
fn rel(ws: &Workspace, path: &std::path::Path) -> String {
    path.strip_prefix(&ws.root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
