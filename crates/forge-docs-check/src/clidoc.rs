//! Marker-hybrid CLI reference (`<!-- forge:cli -->`).
//!
//! The `forge` CLI surface is defined once as a clap command model in
//! `forge_cli` (the `Cli`/`Commands` derive). This module introspects that
//! model via [`forge_cli::cli`] and renders an authoritative command reference
//! — synopsis, arguments, options, nested subcommands — into the
//! `<!-- forge:cli -->` region of `site/src/content/docs/crates/forge.md`.
//!
//! The `cli-doc` rule fails CI when that region is stale; `make docs-cli`
//! regenerates it. Because both the published reference and the binary's
//! argument parser are derived from the *same* clap model, a new subcommand,
//! argument, or flag updates the docs automatically and can never silently
//! drift from what the CLI actually accepts. Authored prose elsewhere on the
//! page (the narrative `## Commands` section) is outside the markers and is
//! never touched.

use crate::checks::read_optional;
use crate::discovery::Workspace;
use crate::{markers, Finding};
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

pub const BLOCK_OPEN: &str = "<!-- forge:cli -->";
pub const BLOCK_CLOSE: &str = "<!-- /forge:cli -->";

/// The docs page that hosts the generated CLI reference.
fn page_path(ws: &Workspace) -> PathBuf {
    ws.docs_dir().join("crates").join("forge.md")
}

/// Render the body between the markers from the live clap command model.
pub fn render_block_body() -> String {
    let cli = forge_cli::cli();
    let mut out = String::new();
    out.push_str(
        "<!-- generated from the forge_cli clap model — run `make docs-cli` to refresh -->\n",
    );
    for sub in visible_subcommands(&cli) {
        render_command(&mut out, sub, "forge");
    }
    out.trim_end().to_string()
}

/// Rule `cli-doc`: the generated CLI reference in `forge.md` matches the clap
/// model. Only fires if the page has opted in with an opening marker.
pub fn check(ws: &Workspace) -> Vec<Finding> {
    let page = match read_optional(&page_path(ws)) {
        Some(p) => p,
        None => return Vec::new(),
    };
    if !page.contains(BLOCK_OPEN) {
        return Vec::new(); // not opted in
    }
    let expected = render_block_body();
    match markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
        Some((_, _, body)) if body == expected => Vec::new(),
        Some(_) => vec![Finding::new(
            "cli-doc",
            "the <!-- forge:cli --> CLI reference in crates/forge.md is stale; run `make docs-cli` to regenerate it from the clap model",
        )],
        None => vec![Finding::new(
            "cli-doc",
            "crates/forge.md has an opening <!-- forge:cli --> with no closing marker",
        )],
    }
}

/// Regenerate the `<!-- forge:cli -->` region in place. Returns the rewritten
/// path (empty if the page is absent, not opted in, or already current).
pub fn write_all(ws: &Workspace) -> std::io::Result<Vec<PathBuf>> {
    let path = page_path(ws);
    let page = match read_optional(&path) {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };
    if !page.contains(BLOCK_OPEN) {
        return Ok(Vec::new());
    }
    let expected = render_block_body();
    if let Some((_, _, body)) = markers::find_region(&page, BLOCK_OPEN, BLOCK_CLOSE) {
        if body == expected {
            return Ok(Vec::new());
        }
        if let Some(updated) = markers::replace_region(&page, BLOCK_OPEN, BLOCK_CLOSE, &expected) {
            std::fs::write(&path, updated)?;
            return Ok(vec![path]);
        }
    }
    Ok(Vec::new())
}

/// Render one subcommand (and any nested subcommands) into the reference.
fn render_command(out: &mut String, cmd: &Command, parent: &str) {
    let full = format!("{parent} {}", cmd.get_name());

    let about = cmd
        .get_about()
        .map(|s| s.to_string())
        .unwrap_or_default()
        .trim()
        .to_string();
    out.push('\n');
    if about.is_empty() {
        out.push_str(&format!("**`{full}`**\n\n"));
    } else {
        out.push_str(&format!("**`{full}`** — {about}\n\n"));
    }

    let positionals: Vec<&Arg> = cmd.get_positionals().collect();
    let options: Vec<&Arg> = cmd
        .get_arguments()
        .filter(|a| !a.is_positional() && a.get_id() != "help" && a.get_id() != "version")
        .collect();
    let subs = visible_subcommands(cmd);

    // Synopsis line: `forge <name> [OPTIONS] <COMMAND> <POS>...`.
    let mut synopsis = full.clone();
    if !options.is_empty() {
        synopsis.push_str(" [OPTIONS]");
    }
    if !subs.is_empty() {
        synopsis.push_str(" <COMMAND>");
    }
    for p in &positionals {
        let token = if p.is_required_set() {
            format!("<{}>", value_name(p))
        } else {
            format!("[{}]", value_name(p))
        };
        synopsis.push(' ');
        synopsis.push_str(&token);
        if is_multiple(p) {
            synopsis.push_str("...");
        }
    }
    out.push_str("```text\n");
    out.push_str(&synopsis);
    out.push_str("\n```\n");

    if !positionals.is_empty() {
        out.push_str("\nArguments:\n");
        for p in &positionals {
            out.push_str(&format!(
                "- `<{}>`{}\n",
                value_name(p),
                help_suffix(p.get_help().map(|s| s.to_string()))
            ));
        }
    }

    if !options.is_empty() {
        out.push_str("\nOptions:\n");
        for o in &options {
            out.push_str(&format!(
                "- `{}`{}\n",
                option_flag(o),
                help_suffix(o.get_help().map(|s| s.to_string()))
            ));
        }
    }

    for s in subs {
        render_command(out, s, &full);
    }
}

/// Subcommands a user can invoke: skips the auto-generated `help` command and
/// any hidden subcommands, sorted by name for deterministic output.
fn visible_subcommands(cmd: &Command) -> Vec<&Command> {
    let mut subs: Vec<&Command> = cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "help" && !s.is_hide_set())
        .collect();
    subs.sort_by_key(|s| s.get_name().to_string());
    subs
}

/// The display value name for an argument (`<APP_DIR>`), matching clap's
/// default of upper-casing the arg id when no explicit value name is set.
fn value_name(arg: &Arg) -> String {
    if let Some(names) = arg.get_value_names() {
        if !names.is_empty() {
            return names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
    arg.get_id().as_str().to_uppercase()
}

/// The flag spelling for an option: `--out, -o <OUT>` or `--embed`.
fn option_flag(arg: &Arg) -> String {
    let mut parts = Vec::new();
    if let Some(long) = arg.get_long() {
        parts.push(format!("--{long}"));
    }
    if let Some(short) = arg.get_short() {
        parts.push(format!("-{short}"));
    }
    let mut spelled = parts.join(", ");
    if takes_value(arg) {
        spelled.push_str(&format!(" <{}>", value_name(arg)));
    }
    spelled
}

/// Whether the argument consumes a value (vs. a boolean/counting flag).
fn takes_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Set | ArgAction::Append)
}

/// Whether the argument accepts multiple values (e.g. a trailing var-arg).
fn is_multiple(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Append)
}

/// Format an optional help string as a ` — help` suffix (empty when absent).
fn help_suffix(help: Option<String>) -> String {
    match help {
        Some(h) if !h.trim().is_empty() => format!(" — {}", h.trim()),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_lists_every_subcommand() {
        let body = render_block_body();
        for cmd in ["dev", "build", "bundle", "smelt", "sign", "icon", "docs"] {
            assert!(
                body.contains(&format!("**`forge {cmd}`**")),
                "reference must document `forge {cmd}`:\n{body}"
            );
        }
        // The auto-generated `help` subcommand must not leak into the docs.
        assert!(
            !body.contains("**`forge help`**"),
            "help must be filtered out"
        );
    }

    #[test]
    fn reference_documents_flags_and_positionals() {
        let body = render_block_body();
        // smelt: `--out, -o <OUT>`, `--embed`, and the <APP_DIR> positional.
        assert!(
            body.contains("--out, -o <OUT>"),
            "smelt --out with alias:\n{body}"
        );
        assert!(body.contains("`--embed`"), "smelt --embed flag:\n{body}");
        assert!(body.contains("<APP_DIR>"), "app_dir positional:\n{body}");
        // sign: `--identity, -i <IDENTITY>` and the <ARTIFACT> positional.
        assert!(
            body.contains("--identity, -i <IDENTITY>"),
            "sign identity:\n{body}"
        );
        assert!(body.contains("<ARTIFACT>"), "sign artifact:\n{body}");
    }

    #[test]
    fn reference_renders_nested_icon_subcommands() {
        let body = render_block_body();
        assert!(
            body.contains("**`forge icon create`**"),
            "icon create:\n{body}"
        );
        assert!(
            body.contains("**`forge icon validate`**"),
            "icon validate:\n{body}"
        );
    }
}
