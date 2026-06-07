//! Rule `cli-command`: every `forge` subcommand is documented in `crates/forge.md`.
//!
//! Caught the missing `forge smelt` entry. The command list is sourced from the
//! CLI's own clap command model in `forge_cli/src/main.rs` (a single in-repo
//! source of truth), so adding a subcommand there surfaces the documentation
//! requirement automatically.
//!
//! Before the Phase 5 clap migration this was sourced from a hand-written
//! `forge <a|b|c>` usage banner; clap derive replaced that banner with the
//! `#[derive(Subcommand)] enum Commands { .. }` block, so this rule now parses
//! the enum variants and applies clap's default PascalCase -> kebab-case rename
//! to recover the exact subcommand names the CLI exposes.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let main_rs = ws.root.join("crates/forge_cli/src/main.rs");
    let main_src = match read_optional(&main_rs) {
        Some(s) => s,
        None => {
            return vec![Finding::new(
                "cli-command",
                format!(
                    "cannot read {} to determine the subcommand list",
                    main_rs.display()
                ),
            )]
        }
    };

    let commands = match parse_subcommands(&main_src) {
        Some(c) if !c.is_empty() => c,
        _ => {
            return vec![Finding::new(
                "cli-command",
                "could not find the `#[derive(Subcommand)] enum Commands` block in forge_cli/src/main.rs to source the command list",
            )]
        }
    };

    let forge_md = ws.docs_dir().join("crates/forge.md");
    let doc = match read_optional(&forge_md) {
        Some(d) => d,
        None => {
            return vec![Finding::new(
                "cli-command",
                "site/src/content/docs/crates/forge.md is missing (cannot document CLI commands)",
            )]
        }
    };

    let mut findings = Vec::new();
    for cmd in commands {
        // Documented as a `forge <cmd>` reference (heading or code/example).
        if !doc.contains(&format!("forge {cmd}")) {
            findings.push(Finding::new(
                "cli-command",
                format!(
                    "`forge {cmd}` is a real subcommand but is not documented in crates/forge.md"
                ),
            ));
        }
    }
    findings
}

/// Extract the top-level variant names from the clap `enum Commands { .. }`
/// block and convert each to the kebab-case name clap exposes on the CLI.
fn parse_subcommands(src: &str) -> Option<Vec<String>> {
    let body = enum_body(src, "Commands")?;

    let mut variants = Vec::new();
    let mut depth = 0i32;
    for line in body.lines() {
        let trimmed = line.trim();
        // A variant name is a PascalCase identifier sitting at the enum's top
        // level (depth 0). Struct-variant fields live at depth >= 1, and
        // attributes/doc-comments (`#[..]`, `///`) don't start with an
        // uppercase identifier, so both are skipped naturally.
        if depth == 0 {
            if let Some(name) = variant_name(trimmed) {
                variants.push(to_kebab(&name));
            }
        }
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        if depth < 0 {
            depth = 0;
        }
    }

    if variants.is_empty() {
        None
    } else {
        Some(variants)
    }
}

/// Return the brace-delimited body of `enum <name>`, matching braces so nested
/// struct-variant `{ .. }` blocks don't terminate the scan early.
fn enum_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("enum {name}");
    let start = src.find(&needle)?;
    let open = src[start..].find('{')? + start;

    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut body_start = open + 1;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                if depth == 1 {
                    body_start = i + 1;
                }
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[body_start..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// If `line` begins a variant declaration, return its identifier. Accepts the
/// PascalCase name when it is followed by variant syntax — `{` (struct
/// variant), `(` (tuple variant), `,`, or end of line (unit variant).
fn variant_name(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let end = line
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
        .map(|(idx, _)| idx)
        .unwrap_or(line.len());
    let ident = &line[..end];
    let rest = line[end..].trim_start();
    if rest.is_empty() || rest.starts_with('{') || rest.starts_with('(') || rest.starts_with(',') {
        Some(ident.to_string())
    } else {
        None
    }
}

/// Replicate clap's default subcommand rename (PascalCase -> kebab-case).
fn to_kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
