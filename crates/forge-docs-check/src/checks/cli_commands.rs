//! Rule `cli-command`: every `forge` subcommand is documented in `crates/forge.md`.
//!
//! Caught the missing `forge smelt` entry. The command list is sourced from the
//! CLI's own usage banner in `forge_cli/src/main.rs` (a single in-repo source),
//! so adding a subcommand there surfaces the requirement automatically. After the
//! Phase 5 clap migration this source switches to the clap command model.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;
use regex::Regex;

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
                "could not find the `forge <a|b|c>` usage banner in forge_cli/src/main.rs to source the command list",
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

/// Extract the pipe-separated command list from the first `forge <a|b|c>` usage
/// banner found in the CLI source.
fn parse_subcommands(src: &str) -> Option<Vec<String>> {
    let re = Regex::new(r"forge <([a-z][a-z|]+)>").expect("valid usage regex");
    let caps = re.captures(src)?;
    let list = caps
        .get(1)?
        .as_str()
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    Some(list)
}
