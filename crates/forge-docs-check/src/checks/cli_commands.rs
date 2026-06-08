//! Rule `cli-command`: every `forge` subcommand is documented in `crates/forge.md`.
//!
//! Caught the missing `forge smelt` entry. The command list is sourced directly
//! from the clap command model ([`forge_cli::cli`]) — the same model the binary
//! parses with — so adding a subcommand surfaces the documentation requirement
//! automatically, with no source-text parsing or name-mangling to keep in sync.
//!
//! This is the lightweight *presence* guard (each subcommand is mentioned
//! somewhere on the page, prose or generated). The richer
//! [`crate::clidoc`] rule (`cli-doc`) regenerates the full `<!-- forge:cli -->`
//! reference — synopsis, args, options — from the same model.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::Finding;

pub fn check(ws: &Workspace) -> Vec<Finding> {
    let commands = subcommand_names();
    if commands.is_empty() {
        return vec![Finding::new(
            "cli-command",
            "forge_cli::cli() exposes no subcommands — the CLI model could not be introspected",
        )];
    }

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
        // Documented as a `forge <cmd>` reference (heading, prose, or the
        // generated CLI reference block).
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

/// The user-invocable top-level subcommand names from the clap model, skipping
/// clap's auto-generated `help` command and any hidden subcommands.
fn subcommand_names() -> Vec<String> {
    forge_cli::cli()
        .get_subcommands()
        .filter(|s| s.get_name() != "help" && !s.is_hide_set())
        .map(|s| s.get_name().to_string())
        .collect()
}
